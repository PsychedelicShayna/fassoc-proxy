#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, windows_subsystem = "console")]

use std::fs;

use serde_json as sj;
use std::{env, io::Read, path::Path};

mod logging;
use logging::MAIN_LOGGER;

mod rules;
use rules::{FassocRules, Rule};

use crate::winproc::create_process;

mod winproc;

#[derive(Debug)]
enum ReadRulesError {
    SjErr(sj::Error),
    IoErr(std::io::Error),
}

impl std::fmt::Display for ReadRulesError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReadRulesError::SjErr(e) => write!(f, "Serde JSON Error: {}", e),
            ReadRulesError::IoErr(e) => write!(f, "IO Error: {}", e),
        }
    }
}

fn read_fassoc_rules(path: String) -> Result<FassocRules, ReadRulesError> {
    let fassoc_rules: FassocRules = sj::from_str(
        fs::read_to_string(path)
            .map_err(|e| ReadRulesError::IoErr(e))?
            .as_str(),
    )
    .map_err(|e| ReadRulesError::SjErr(e))?;

    Ok(fassoc_rules)
}

fn subst_arg_placeholders(rules: FassocRules, args: Vec<String>) -> FassocRules {
    let mut new_rules: FassocRules = rules;

    for (index, argument) in args.iter().enumerate() {
        let placeholder = format!("~~${}", index);

        for rule in new_rules.rules.values_mut() {
            rule.command = rule.command.replace(&placeholder, argument);

            rule.arguments = rule
                .arguments
                .to_owned()
                .map(|arg| arg.replace(&placeholder, argument));

            rule.cwd = rule
                .cwd
                .to_owned()
                .map(|cwd| cwd.replace(&placeholder, argument));

            rule.extras = rule.extras.to_owned().map(|mut extras| {
                extras.title = extras.title.map(|str| str.replace(&placeholder, argument));
                extras.desktop = extras
                    .desktop
                    .map(|str| str.replace(&placeholder, argument));
                extras
            });
        }
    }

    new_rules
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

    let fassoc_rules = match read_fassoc_rules(proxy_rules_path) {
        Ok(rules) => subst_arg_placeholders(rules, cli_args.to_owned()),
        Err(error) => {
            log::error!("Failure when reading fassoc rules ({})", error);
            panic!();
        }
    };

    let suitable_rule: Rule = match fassoc_rules.find_suitable_rule(target_file_path) {
        Ok(rule) => rule,
        Err(error) => {
            log::error!(
                "Could not find a suitable rule for the file \"{}\", because: {}",
                target_file_name,
                error
            );
            panic!("");
        }
    };

    log::debug!(
        "Creating process, path: \"{}\", args: \"{}\"",
        suitable_rule.command.to_owned(),
        suitable_rule
            .arguments
            .to_owned()
            .unwrap_or(String::from("NONE"))
    );

    match create_process(&suitable_rule) {
        Ok(process_info) => {
            log::debug!("Process created, information: {:?}", process_info)
        }

        Err(error) => {
            log::error!("Error when attempting to create process: {}", error)
        }
    }

    if cfg!(debug_assertions) {
        println!("Press enter to close the debug message console.");
        std::io::stdin().read(&mut [0u8]).unwrap_or(1);
    }
}
