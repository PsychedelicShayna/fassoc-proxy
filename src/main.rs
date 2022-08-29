#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

use regex::Regex;
use serde_json as sj;
use std::{collections::HashMap, env, io::Read, path::Path};

mod logging;
use logging::MAIN_LOGGER;

mod winutil;
use winutil::procspawn::create_process;
use winutil::structs::CreationExtras;

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

fn rule_matches_file(rule_name: &str, rule: &sj::Value, file_name: &str) -> bool {
    let regex_jv = match rule.get("match") {
        Some(jv) => jv,
        None => {
            log::debug!(
                "Could not find a \"match\" key inside of the rule \"{}\"",
                rule_name
            );

            return true;
        }
    };

    let matcher = |regex_str: &str| -> bool {
        let is_match = match Regex::new(regex_str) {
            Ok(regex) => regex.is_match(file_name),

            Err(error) => {
                log::error!(
                    "The RegEx pattern \"{}\" belonging to rule \"{}\" failed to compile - {}",
                    regex_str,
                    rule_name,
                    error
                );

                return false;
            }
        };

        log::debug!(
            "Evaluating rule \"{}\"'s RegEx pattern \"{}\" on \"{}\" gave: {} ",
            rule_name,
            regex_str,
            file_name,
            is_match
        );

        is_match
    };

    match regex_jv.as_str() {
        Some(regex_str) => matcher(regex_str),
        None => match regex_jv.as_array() {
            Some(regex_arr) => regex_arr
                .iter()
                .any(|elem| elem.as_str().map_or(false, |regex_str| matcher(regex_str))),

            None => {
                log::error!(
                    "The \"match\" key inside of the rule \"{}\" was not a string or an array!",
                    rule_name
                );

                false
            }
        },
    }
}

fn find_matching_rule(
    allowed_rule_keys: &Vec<String>,
    rules_data: &HashMap<String, sj::Value>,
    target_file_name: &str,
) -> Option<String> {
    let first_matching_rule = allowed_rule_keys.iter().find(|e| {
        let rule_name: &str = e.to_owned();

        // Get the rule with name rule_name from rules HashMap.
        let rule: &sj::Value = match rules_data.get(rule_name) {
            Some(rule) => rule,
            None => return false,
        };

        let is_match = rule_matches_file(rule_name, rule, target_file_name);

        log::debug!(
            "Rule \"{}\" for file \"{}\" evaluated to {}",
            rule_name,
            target_file_name,
            is_match,
        );

        is_match
    });

    first_matching_rule.map(|s| String::from(s))
}

fn main() {
    log::set_logger(&MAIN_LOGGER).unwrap();

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

    let target_file_path = Path::new(&cli_args[1]);

    let target_file_name = match target_file_path.to_str() {
        Some(file_name) => file_name,
        None => {
            log::error!("Could not get the name of the target file from the file path!");
            panic!()
        }
    };

    let target_file_ext: Option<&str> = match target_file_path.extension() {
        Some(val) => val.to_str(),
        None => None,
    };

    let target_folder = match target_file_path.parent() {
        Some(path) => match path.to_str() {
            Some(path_str) => path_str,
            None => {
                log::error!(
                    "Failed to convert the target file's parent directory path into a string."
                );
                panic!();
            }
        },
        None => {
            log::error!("Failed to establish the target file's parent directory.");
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

    let rules: HashMap<String, sj::Value> = match sj::from_value(proxy_rules["rules"].to_owned()) {
        Ok(rules) => rules,
        Err(error) => {
            log::error!(
                "The \"rules\" key is missing from proxy rules, or it is not an object ({})",
                error
            );
            panic!()
        }
    };

    let mappings: HashMap<String, Vec<String>> = match sj::from_value(
        proxy_rules["mappings"].to_owned(),
    ) {
        Ok(mappings) => mappings,
        Err(error) => {
            log::error!(
                "The \"mappings\" key is missing from proxy rules, or could not be deserialized - {} ",
                error
            );
            panic!();
        }
    };

    // The fallback "*" catch all mappings, for when a file has no extension,
    // or there is no match anywehre else.
    let rule_names_fallback: Option<&Vec<String>> = mappings.get("*");

    // The mappings resolved using the target file's extension.
    let rule_names_from_ext: Option<&Vec<String>> = match target_file_ext {
        Some(ext) => mappings.get(ext),
        None => None,
    };

    let rule_name_from_ext: Option<String> = match rule_names_from_ext {
        Some(rule_names) => find_matching_rule(rule_names, &rules, target_file_name),
        None => None,
    };

    let final_rule_name: String = match rule_name_from_ext {
        Some(rule_name) => rule_name,
        None => {
            if rule_names_fallback.is_none() {
                log::warn!("Could not map the extension ({:?}) of file {} to a rule, and no fallbak mapping was defined.", target_file_ext, target_file_name);
                panic!();
            }

            let rule_from_fallback: String = match find_matching_rule(
                rule_names_fallback.unwrap(),
                &rules,
                &target_file_name,
            ) {
                Some(rule_name) => rule_name,
                None => {
                    log::warn!("Could not find a mapping for the file \"{}\" using its extension, and no fallback is defined.", target_file_name);
                    panic!();
                }
            };

            rule_from_fallback
        }
    };

    let rule_data: &sj::Value = match rules.get(&final_rule_name) {
        Some(rule_data) => rule_data,
        None => {
            log::error!(
                "Could not find a rule with name \"{}\" - this is a strange state!",
                final_rule_name
            );
            panic!();
        }
    };

    let mut command: String = match rule_data.get("command") {
        Some(jv) => match jv.as_str() {
            Some(jstr) => String::from(jstr),
            None => {
                log::error!(
                    "The \"command\" key in rule \"{}\" is not a string!",
                    final_rule_name
                );

                panic!();
            }
        },
        None => {
            log::error!(
                "Malformed rule! Rule with the name \"{}\" is missing its command key!",
                final_rule_name
            );
            panic!();
        }
    };

    let mut arguments = String::from(rule_data["arguments"].as_str().unwrap_or(""));

    let mut cwd = String::from(rule_data["cwd"].as_str().unwrap_or(target_folder));

    let creation_extras: CreationExtras = rule_data["extras"]
        .as_object()
        .map_or(CreationExtras::default(), |obj_extras| {
            CreationExtras::from_json(obj_extras)
        });

    log::debug!("CreationExtras: {:?}", creation_extras);
    log::debug!("NativeCreationExtras: {:?}", creation_extras.as_native());

    for (index, cli_arg) in cli_args.iter().enumerate() {
        let clarg_placeholder_string = format!("~~${}", index);
        let clarg_placeholder: &str = clarg_placeholder_string.as_str();

        let tf_placeholder: &str = "~~$filedir";

        if command.contains(clarg_placeholder) {
            command = command.replace(clarg_placeholder, cli_arg.as_str());
        }

        if command.contains(tf_placeholder) {
            command = command.replace(tf_placeholder, target_folder);
        }

        if arguments.contains(clarg_placeholder) {
            arguments = arguments.replace(clarg_placeholder, cli_arg);
        }

        if arguments.contains(tf_placeholder) {
            arguments = arguments.replace(tf_placeholder, target_folder);
        }

        if cwd.contains(clarg_placeholder) {
            cwd = cwd.replace(clarg_placeholder, cli_arg);
        }

        if cwd.contains(tf_placeholder) {
            cwd = cwd.replace(tf_placeholder, target_folder);
        }
    }

    log::debug!(
        "Creating process, path: \"{}\", args: \"{}\"",
        command,
        arguments
    );

    let native_extras = creation_extras.as_native();

    create_process(command, arguments, cwd, native_extras);

    if cfg!(debug_assertions) {
        println!("Press enter to close the debug message console.");
        std::io::stdin().read(&mut [0u8]).unwrap_or(1);
    }
}
