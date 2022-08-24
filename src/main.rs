#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

use regex::Regex;
use serde_json as sj;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub mod winutil;
use winutil::structs::*;
use winutil::*;

struct FileLogger;
static FILE_LOGGER: FileLogger = FileLogger;

struct ProxyRule {
    patterns: Vec<String>,
    command: String,
    arguments: String,
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let current_exe_path = env::current_exe().unwrap();
            let exe_dir_path = current_exe_path.parent().unwrap();
            let log_file_path = exe_dir_path.join("fassoc-proxy.log");

            let mut log_file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file_path)
                .unwrap();

            let log_time = chrono::Local::now().format("%d-%m-%y %H:%M:%S");

            let outmsg = format!(
                "[{}] - {} - {}",
                log_time.to_string(),
                record.level(),
                record.args()
            );

            println!("{}", outmsg);

            match writeln!(log_file, "{}", outmsg) {
                Ok(val) => val,
                Err(_) => (),
            }
        }
    }

    fn flush(&self) {}
}

fn read_rules_json(path: String) -> Result<sj::Value, String> {
    let cfg_raw_json: String = match std::fs::read_to_string(path.to_owned()) {
        Ok(raw_json) => raw_json,

        Err(error) => {
            return Err(format!(
                "Failed to read proxy rules file at \"{}\", because {}",
                path.to_owned(),
                error
            ))
        }
    };

    let parsed_json: sj::Value = match serde_json::from_str(cfg_raw_json.as_str()) {
        Ok(value) => value,
        Err(error) => return Err(format!("Failed to parse the proxy rules JSON: {:?}", error)),
    };

    if !parsed_json.is_object() {
        return Err(String::from(
            "The parsed proxy rules JSON data was not an object!",
        ));
    }

    return Ok(parsed_json);
}

fn search_for_rule(mapping_value: sj::Value, rules: sj::Value, file_name: &str) -> Option<String> {
    let candidate_rule_names: Vec<&str> = match mapping_value.as_array() {
        Some(rule_names) => rule_names.iter().filter_map(|e|e.as_str()).collect(),
        None => return None
    };

    
    let first_matching_rule = candidate_rule_names.iter().find(|e| {
        let rule_name: &str = e.to_owned();
        
        // Get the rule with name rule_name from rules HashMap.
        let rule = match rules.get(rule_name) {
            Some(rule) => rule,
            None => return false
        };
        
        // Try to extract the array of RegEx patterns from the rule.
        let regex_patterns = match rule["match"].as_array() {
            Some(patterns) => patterns,
            None => return false
        };

        // Return true if any of the RegEx patterns match the target file.
        regex_patterns.iter().any(|regstr| {
            let regex_pattern_str = match regstr.as_str() {
                Some(pattern) => pattern,
                None => return false
            };

            match Regex::new(regex_pattern_str) {
                Ok(regex) => regex.is_match(file_name),

                Err(error) => {
                    log::error!("The RegEx pattern \"{}\" belonging to rule \"{}\" failed to compile - {}", regex_pattern_str, rule_name, error);
                    return false;
                }
            }
        })
    });

    match first_matching_rule {
        Some(rule_name) => Some(String::from(*rule_name)),
        None => None
    }
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
    let target_file_ext: Option<&str> = match target_file.extension() {
        Some(val) => val.to_str(),
        None => None
    };

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

    let proxy_rules = match read_rules_json(proxy_rules_path) {
        Ok(val) => val,
        Err(err) => {
            log::error!("read_rules_json eeturned err: {}", err);
            panic!();
        }
    };
    
    let mappings = match proxy_rules["mapping"].as_object() {
        Some(val) => val,
        None => {
            log::error!("The \"mappings\" key is missing from proxy rules, or it is not an object.");
            panic!()
        }
    };
    
    let rules = match proxy_rules["rules"].as_object() {
        Some(val) => val,
        None => {
            log::error!("The \"rules\" key is missing from proxy rules, or it is not an object.");
            panic!()
        }       
    };
    
    // All of the rule names that are applicable to target file extension.
    let target_file_ext_mappings: Option<Vec<&str>> = match target_file_ext {
        Some(ext) => match mappings[ext].as_array() {
            Some(arr) => {
                let result: Vec<&str> = arr.iter().filter_map(|e|e.as_str()).collect();
                if result.len() == 0 { None } else { Some(result) }
            }
            None => None
        },
        None => None
    };

    // The name of the first rule whose RegEx matches the target file.
    let first_matching_rule: Option<&str> = match target_file_ext_mappings {
        Some(rule_name_array) => {
            let matching_rule_name = rule_name_array.iter().find(|e| {
                let rule_name: &str = e.to_owned();
                
                // Get the rule with name rule_name from rules HashMap.
                let rule = match rules.get(rule_name) {
                    Some(rule) => rule,
                    None => return false
                };
                
                // Try to extract the array of RegEx patterns from the rule.
                let regex_patterns = match rule["match"].as_array() {
                    Some(patterns) => patterns,
                    None => return false
                };

                // Return true if any of the RegEx patterns match the target file.
                regex_patterns.iter().any(|regstr| {
                    let regex_pattern_str = match regstr.as_str() {
                        Some(pattern) => pattern,
                        None => return false
                    };

                    match Regex::new(regex_pattern_str) {
                        Ok(regex) => {
                            return regex.is_match(target_file.to_str().unwrap());
                        },

                        Err(error) => {
                            log::error!("The RegEx pattern \"{}\" belonging to rule \"{}\" failed to compile - {}", regex_pattern_str, rule_name, error);
                            return false;
                        }
                    }
                })
            });

            match matching_rule_name {
                Some(str) => Some(*str),
                None => None
            }
        },

        None => None
    };

    //
    // let final_rule_name: Option<&str> = match(first_matching_rule) {
    //     Some(rule_name) => Some(rule_name),
    //     None => {
    //         let star_mapping = mappings.get("*");
    //
    //         if star_mapping.is_none() {
    //             None
    //         }
    //
    //     }
    // };
    //
    // 


    // Old Code ------------------------------

    for (index, proxy_rule) in proxy_rules.as_array().unwrap().iter().enumerate() {
        // Get the comment string from the current proxy rule.
        let mut command: String = match proxy_rule["cmd"].as_str() {
            Some(val) => String::from(val),
            None => {
                log::error!(
                    "Proxy rule #{} is missing the \"cmd\" key, or it is not a string.",
                    index
                );
                panic!();
            }
        };

        // Get arguments string from proxy rule. This should be one contiguous
        // string arguments, as opposed to a more traditional array approach.
        let mut arguments: String = match proxy_rule["args"].as_str() {
            Some(val) => val.to_owned(),
            None => {
                log::error!(
                    "Proxy rule #{} is missing the \"args\" key, or it is not a string.",
                    index
                );
                panic!();
            }
        };

        // Get working directory override from proxy rule, if the key it exists.
        // If the key doesn't exist, get None instead, as this key is optional.
        let mut wd_override: Option<String> = match proxy_rule["wd"].as_str() {
            Some(val) => Some(String::from(val)),
            None => None,
        };

        // Get the RegEx pattern strings from the proxy rule.
        let patterns: Vec<String> = match proxy_rule["regex"].as_array() {
            Some(val) => val
                .to_owned()
                .iter()
                .map(|pattern| match pattern.as_str() {
                    Some(pattern) => String::from(pattern),
                    None => {
                        log::error!(
                            "One of the RegEx patterns of proxy rule #{} is not a string!",
                            index
                        );
                        panic!()
                    }
                })
                .collect(),
            None => {
                log::error!(
                    "Proxy rule #{} is missing the \"RegEx\" key, or it is not an array.",
                    index
                );
                panic!()
            }
        };

        // Skip the current proxy rule if the target file's name doesn't match
        // any of the proxy rule's RegEx patterns.
        if !patterns.iter().any(|pattern| match Regex::new(pattern) {
            Ok(val) => val.is_match(target_file.to_str().unwrap()),
            Err(err) => {
                log::error!(
                    "Error when compiling RegEx pattern {:?} - {:?}",
                    pattern,
                    err
                );
                panic!()
            }
        }) {
            continue;
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
                None => None,
            }
        }

        log::debug!(
            "Creating process, path: \"{}\", args: \"{}\", cwd \"{}\"",
            command,
            arguments,
            match wd_override.to_owned() {
                Some(val) => val,
                None => String::from("None"),
            }
        );

        // winutil::create_process(command, arguments, wd_override);

        if cfg!(debug_assertions) {
            write!(
                std::io::stdout(),
                "{}",
                "Press enter to close the debug message console..."
            )
            .unwrap();
            std::io::stdin().read(&mut [0u8]).unwrap();
        }
    }
}
