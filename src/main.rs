#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] 
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

use std::time::Duration;
use serde_json as sj;
use std::path::Path;
use std::io::Write;
use regex::Regex;
use std::env;
use std::fs;

struct FileLogger;
static FILE_LOGGER: FileLogger = FileLogger;

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let current_exe_path = env::current_exe().unwrap();
            let exe_dir_path = current_exe_path.parent().unwrap();
            let log_file_path = exe_dir_path.join("fassoc-proxy.log");
            
            let mut log_file = fs::OpenOptions::new().create(true)   
                                                     .append(true)
                                                     .open(log_file_path)
                                                     .unwrap();

            let log_time = chrono::Local::now().format("%d-%m-%y %H:%M:%S");

            let outmsg = format!("[{}] - {} - {}", 
                log_time.to_string(),
                record.level(),
                record.args()
            );

            println!("{}", outmsg);

            match writeln!(log_file, "{}", outmsg) {
                Ok(val) => val,
                Err(_) => ()
            }
        }
    }

    fn flush(&self) {}
}

fn read_json_config(path: String) -> Result<sj::Value, String> {
    let cfg_raw_json: String = match std::fs::read_to_string(path) {
        Ok(raw_json) => raw_json,
        Err(error) => return Err(format!("Failed to read config file: {:?}", error))
    };

    let parsed_json: sj::Value = match serde_json::from_str(cfg_raw_json.as_str()) {
        Ok(value) => value,
        Err(error) => return Err(format!("Failed to parse the JSON data: {:?}", error))
    };

    if !parsed_json.is_array() {
        return Err(String::from("The parsed JSON data was not an array!"));
    }

    return Ok(parsed_json);
}

fn main() {
    log::set_logger(&FILE_LOGGER).unwrap();

    if cfg!(debug_assertions) {
        log::set_max_level(log::LevelFilter::Debug);
    } else {
        log::set_max_level(log::LevelFilter::Info);
    }

    let cli_args: Vec<String> = env::args().collect();
    let usage = "Usage: fassoc-proxy <file-to-open> (optional: proxyrules.json)\nIf proxyrules.json not provided, environment variable FASSOC_PROXY_RULES is used to find proxyrules.json instead.";

    log::debug!("Received command line arguments: {:?}", cli_args);
 
    if cli_args.len() < 2 {
        println!("{}", usage);
        log::debug!("Printing usage and exiting due to invalid argument length, was < 2 ({:?})", cli_args.len());
        std::process::exit(1);
    }

    // The same as if the argument were to fail with an incorrect value, but
    // faster because it skips opening the file.
    if cli_args[1].eq("-h") || cli_args[1].eq("--help") {
        println!("{}", usage);
        log::debug!("Printing usage and exiting because first argument was -h/--help");
        std::process::exit(1);
    }

    let target_file = Path::new(&cli_args[1]);

    let target_folder = match target_file.parent() {
        Some(val) => val,
        None => {
            log::error!("Failed to establish parent directory of the target file.");
            panic!();
        }
    };

    let proxy_config_file_path: String = if cli_args.len() >= 3 {
        cli_args[2].clone()
    } else {
        match env::var("FASSOC_PROXY_RULES") {
            Ok(config) => config,
            Err(_) => {
                log::error!("No argument or environment variable was given that points to the config file.");
                panic!()
            }
        }
    };

    let proxy_config_json = match read_json_config(proxy_config_file_path) {
        Ok(config) => config,
        Err(err) => {
            log::error!("read_json_config returned err: {}", err);
            panic!();
        }
    };

    for (index, config_entry) in proxy_config_json.as_array().unwrap().iter().enumerate() {
        // Collect the command string from the current config entry.
        let mut command: String = match config_entry["command"].as_str() {
            Some(val) => String::from(val),
            None => {
                log::error!("Proxyrules JSON entry ({}) is missing \"command\" key.", index);
                panic!();
            }
        };

        // Collect the command argument string from the current config entry.
        let mut command_args: Vec<String> = match config_entry["arguments"].as_array() {
            Some(val) => val.to_owned().iter().map(|command_arg| {
                match command_arg.as_str() {
                    Some(command_arg) => String::from(command_arg),
                    None => { 
                        log::error!("One of the command arguments of config entry #{} is not a string!", index);
                        panic!()
                    }
                }
            }).collect(),

            None => { 
                log::error!("Config entry {:?} is missing arguments key.", index);
                panic!()
            }
        };

        // Collect the pattern strings from the current config entry.
        let patterns: Vec<String> = match config_entry["patterns"].as_array() {
            Some(val) => val.to_owned().iter().map(|pattern| {
                match pattern.as_str() {
                    Some(pattern) => String::from(pattern),
                    None => { 
                        log::error!("One of the patterns of config entry #{} is not a string!", index);
                        panic!()
                    }
                }
            }).collect(),
            None => { 
                log::error!("Config entry {:?} is missing patterns key.", index);
                panic!()
            }
        };

        // Skip the current config entry if the target file doesn't match any of the entry's patterns.
        if !patterns.iter().any(|pattern| {
            match Regex::new(pattern) {
                Ok(val) => val.is_match(target_file.to_str().unwrap()),
                Err(err) => { 
                    log::error!("Error when compiling RegEx pattern {:?} - {:?}", pattern, err);
                    panic!()
                }
            }
        }) {
            continue
        }

        // Go through the command line arguments, and check if the command and its
        // arguments contain a substitution string that should be substituted with
        // a command line argument whose index matches the number in the string.
        // If so, replace the string with the corresponding command line argument.
        for (index, cli_arg) in cli_args.iter().enumerate() {
            // The substitution string for the current index.
            let sstr: String = format!("~~$#{}", index);

            // Substitute the substitution string (lookfor) in the command string.
            if command.contains(sstr.as_str()) {
                command = command.replace(sstr.as_str(), cli_arg);
            }

            for command_arg in command_args.iter_mut()
                                           .filter(|e|e.contains(sstr.as_str()))
                                                  
            {
                *command_arg = command_arg.replace(sstr.as_str(), cli_arg)
            }
        }

        match std::process::Command::new(command.as_str()).args(command_args.to_owned())
                                                          .current_dir(target_folder)
                                                          .output()
        {
            Ok(_) => {
                log::debug!("Ran command \"{}\" with arguments ({:?}), in CWD: {}", 
                    command, command_args, target_folder.to_str().unwrap());
            },
            Err(_) => ()
        }

        std::thread::sleep(Duration::from_millis(2000));
    }
}
