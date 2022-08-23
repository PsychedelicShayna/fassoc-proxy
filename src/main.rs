#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] 
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

use serde_json as sj;
use std::path::Path;
use std::io::{Read, Write};
use regex::Regex;
use std::env;
use std::fs;

pub mod winutil;

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

fn read_rules_json(path: String) -> Result<sj::Value, String> {
    let cfg_raw_json: String = match std::fs::read_to_string(path.to_owned()) {
        Ok(raw_json) => raw_json,

        Err(error) => return 
            Err(format!("Failed to read proxy rules file at \"{}\", because {}", path.to_owned(), error))
    };

    let parsed_json: sj::Value = match serde_json::from_str(cfg_raw_json.as_str()) {
        Ok(value) => value,
        Err(error) => return Err(format!("Failed to parse the proxy rules JSON: {:?}", error))
    };

    if !parsed_json.is_array() {
        return Err(String::from("The parsed proxy rules JSON data was not an array!"));
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

    log::debug!("Received command line arguments: {:?}", cli_args);
 
    if cli_args.len() < 2 {
        log::error!("Exiting because the argument length {} < 2", cli_args.len());
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

    let proxy_rules_path: String = if cli_args.len() >= 3 {
        cli_args[2].clone()
    } else {
        match env::var("FASSOC_PROXY_RULES") {
            Ok(val) => val,
            Err(_) => {
                log::error!("No argument or environment variable was given that points to the proxy rules file.");
                panic!()
            }
        }
    };

    let proxy_rules_json = match read_rules_json(proxy_rules_path) {
        Ok(val) => val,
        Err(err) => {
            log::error!("read_rules_json eeturned err: {}", err);
            panic!();
        }
    };

    for (index, proxy_rule) in proxy_rules_json.as_array().unwrap().iter().enumerate() {
        // Get the comment string from the current proxy rule.
        let mut command: String = match proxy_rule["cmd"].as_str() {
            Some(val) => String::from(val),
            None => {
                log::error!("Proxy rule #{} is missing the \"cmd\" key, or it is not a string.", index);
                panic!();
            }
        };

        // Get arguments string from proxy rule. This should be one contiguous
        // string arguments, as opposed to a more traditional array approach.
        let mut arguments: String = match proxy_rule["args"].as_str() {
            Some(val) => val.to_owned(),
            None => {
                log::error!("Proxy rule #{} is missing the \"args\" key, or it is not a string.", index);
                panic!();
            }
        };

        // Get working directory override from proxy rule, if the key it exists.
        // If the key doesn't exist, get None instead, as this key is optional.
        let mut wd_override: Option<String> = match proxy_rule["wd"].as_str() {
            Some(val) => Some(String::from(val)),
            None => None
        };

        // Get the RegEx pattern strings from the proxy rule.
        let patterns: Vec<String> = match proxy_rule["regex"].as_array() {
            Some(val) => val.to_owned().iter().map(|pattern| {
                match pattern.as_str() {
                    Some(pattern) => String::from(pattern),
                    None => { 
                        log::error!("One of the RegEx patterns of proxy rule #{} is not a string!", index);
                        panic!()
                    }
                }
            }).collect(),
            None => { 
                log::error!("Proxy rule #{} is missing the \"RegEx\" key, or it is not an array.", index);
                panic!()
            }
        };

        // Skip the current proxy rule if the target file's name doesn't match 
        // any of the proxy rule's RegEx patterns.
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

        // Replace any valid palceholder strings in the command string and the
        // argument strings with the appropriate values from the command line.
        for (index, cli_arg) in cli_args.iter().enumerate() {
            let placeholder_lftm = format!("~~${}", index);
            let placeholder: &str = placeholder_lftm.as_str();

            if command.contains(placeholder) {
                command = command.replace(placeholder, cli_arg);
            }

            if arguments.contains(placeholder) {
                arguments = arguments.replace(placeholder, cli_arg);
            }
            
            wd_override = match wd_override {
                Some(val) => Some(val.replace(placeholder, cli_arg)),
                None => None
            }
        }

        log::debug!(
            "Creating process, path: \"{}\", args: \"{}\", cwd \"{}\"",
            command,
            arguments,
            match wd_override.to_owned() {
                Some(val) => val,
                None => String::from("None")
            }
        );

        winutil::create_process(command, arguments, wd_override);

        if cfg!(debug_assertions) {
            write!(std::io::stdout(), "{}", "Press enter to close the debug message console...").unwrap();
            std::io::stdin().read(&mut [0u8]).unwrap();
        }
    }
}
