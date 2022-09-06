use std::{collections::HashMap, path::Path};

use regex as re;
use serde_json as sj;

use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
// FassocRules
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub enum FindCommandError {
    CannotConvertPath,
    NoMappingFound,
    NoMatchFound,
}

impl std::fmt::Display for FindCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindCommandError::CannotConvertPath => {
                write!(f, "Could not convert the path of the file into a string.",)
            }
            FindCommandError::NoMappingFound => write!(f, "No mapping could map the file to a matcher."),
            FindCommandError::NoMatchFound => write!(f, "No matcher could match the file to a command."),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FassocRules {
    pub mappings: HashMap<String, Vec<String>>,
    pub matchers: HashMap<String, Matcher>,
    pub commands: HashMap<String, Command>,
}

impl FassocRules {
    pub fn find_suitable_command(&self, file_path: &Path) -> Result<&Command, FindCommandError> {
        let file_name_str: String = file_path.file_name().and_then(|n| n.to_str()).map_or_else(
            || Err(FindCommandError::CannotConvertPath),
            |s| Ok(String::from(s)),
        )?;

        // File extension is allowed to be None, as it could stil be handled
        // by the fallback catch-all mapping "*"
        let file_ext_str = file_path
            .extension()
            .and_then(|ext| ext.to_str().map(|s| String::from(s)));

        let fallback_mapping = self.mappings.get(&String::from("*"));
        let extension_mapping = file_ext_str.to_owned().and_then(|s| self.mappings.get(&s));

        // Use the mapping derived from the file extension, if found, otherwise
        // use the fallback mapping, if found.
        let final_mapping = match extension_mapping.or(fallback_mapping) {
            Some(mapping) => mapping,
            None => {
                // Neither the extension mapping nor the fallback mapping found.
                return Err(FindCommandError::NoMappingFound);
            }
        };

        // File content is stored, so that it doesn't have to be read multiple
        // times. Reading is avoided unless needed, for performance reasons.
        let mut file_content: &mut Option<String> = &mut None;

        let ensure_contents_read = |content: &mut Option<String>| {
            if content.is_none() {
                *content = std::fs::read_to_string(file_path).map_or_else(
                    |e| {
                        log::error!(
                            "Failed to read the contents of file \"{}\" because: {}",
                            file_path.to_str().unwrap_or("CANNOT_GET_FILE"),
                            e
                        );
                        None
                    },
                    |s| Some(s),
                );
            }
        };

        let mapping_name = file_ext_str.to_owned().unwrap_or_else(|| String::from("*"));

        for (index, matcher_name) in final_mapping.iter().enumerate() {
            log::debug!("Trying matcher #{} - {}", index, matcher_name);

            let matcher: &Matcher = match self.matchers.get(matcher_name) {
                Some(matcher) => matcher,
                None => {
                    match self.commands.get(matcher_name) {
                        Some(command) => {
                            log::debug!(
                                "Mapping \"{}\" referred to \"{}\" which isn't a valid matcher, but it is a valid command, mapping directly to command instead.", 
                                mapping_name,
                                matcher_name
                            );

                            return Ok(command);
                        },

                        None => {
                            log::warn!(
                                "Ignored a matcher with mame \"{}\" from mapping \"{}\" because it doesn't point to anything that exists.", 
                                matcher_name, 
                                mapping_name
                            );

                            continue;
                        }
                    }

                }
            };

            let matcher_command: &Command = match self.commands.get(&matcher.command) {
                Some(command) => command,
                None => {
                    log::warn!("The command pointed to by matcher \"{}\" does not exist, ignoring this matcher.", matcher_name);
                    continue;
                }
            };

            let mut is_match: bool = true;

            // If matcher has file name RegEx, match the RegEx against the file.
            is_match &= matcher.regexf.as_ref().map_or(true, |_| {
                matcher.rmatch_file_name(file_name_str.to_owned())
                    .unwrap_or_else(|error| {
                        log::error!(
                            "Encountered RegEx error when evaluating mapped matchers: {:?}",
                            error
                        );
                        false
                    })
            });

            // If match already failed, continue.
            if !is_match {
                continue;
            }

            // If matcher has file content RegEx, match the RegEx against the file.
            is_match &= matcher.regexc.as_ref().map_or(true, |_| {
                ensure_contents_read(&mut file_content);

                file_content.as_ref().map_or(false, |content| {
                    matcher.rmatch_file_content(content).unwrap_or_else(|error| {
                        log::error!(
                            "Encountered RegEx error when evaluating mapped matchers: {:?}",
                            error,
                        );
                        false
                    })
                })
            });

            // If the matcher is still valid after validation, return it.
            if is_match {
                log::debug!(
                    "Matcher #{} - {} - matched this file.",
                    index,
                    matcher_name
                );


                return Ok(matcher_command);
            }
        }

        return Err(FindCommandError::NoMatchFound);
    }
}

// ----------------------------------------------------------------------------
// Matcher
// ----------------------------------------------------------------------------
#[derive(Debug)]
pub enum MatcherError {
    RegexCompileError(re::Error),
    NoRegexError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Matcher {
    pub command: String,
    pub regexf: Option<String>,
    pub regexc: Option<String>,
}

impl Matcher {
    fn rmatch_file(regstr: Option<String>, content: &String) -> Result<bool, MatcherError> {
        regstr.map_or(Err(MatcherError::NoRegexError), |regstr| {
            re::Regex::new(regstr.as_str()).map_or_else(
                |error| Err(MatcherError::RegexCompileError(error)),
                |regex| Ok(regex.is_match(content.as_str())),
            )
        })
    }

    pub fn rmatch_file_name(&self, file_name: String) -> Result<bool, MatcherError> {
        Matcher::rmatch_file(self.regexf.to_owned(), &file_name)
    }

    pub fn rmatch_file_content(&self, file_content: &String) -> Result<bool, MatcherError> {
        Matcher::rmatch_file(self.regexc.to_owned(), file_content)
    }
}

// ----------------------------------------------------------------------------
// Command
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub path: String,
    pub arguments: Option<String>,
    pub cwd: Option<String>,
    pub process_attributes: Option<SecurityAttributes>,
    pub thread_attributes: Option<SecurityAttributes>,
    pub inherit_handles: Option<bool>,
    pub creation_flags: Option<Vec<sj::Value>>,
    // pub environment: Option<Vec<sj::Value>>, -- This will be implemented later.
    pub extras: Option<Extras>,
}

impl Command {}

impl Clone for Command {
    fn clone(&self) -> Self {
        Command {
            path: self.path.clone(),
            arguments: self.arguments.clone(),
            cwd: self.cwd.clone(),
            process_attributes: self.process_attributes.clone(),
            thread_attributes: self.thread_attributes.clone(),
            inherit_handles: self.inherit_handles.clone(),
            creation_flags: self.creation_flags.clone(),
            extras: self.extras.clone(),
        }
    }
}

// ----------------------------------------------------------------------------
// ProcessAttributes
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct SecurityAttributes {
    pub security_descriptor: Option<isize>,
    pub inherit_handle: Option<bool>,
}

// impl SecurityAttributes {}

impl Clone for SecurityAttributes {
    fn clone(&self) -> Self {
        SecurityAttributes {
            security_descriptor: self.security_descriptor.clone(),
            inherit_handle: self.inherit_handle.clone(),
        }
    }
}

// ----------------------------------------------------------------------------
// Extras
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Extras {
    pub desktop: Option<String>,
    pub title: Option<String>,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub x_size: Option<u32>,
    pub y_size: Option<u32>,
    pub x_count_chars: Option<u32>,
    pub y_count_chars: Option<u32>,
    pub fill_attribute: Option<Vec<sj::Value>>,
    pub flags: Option<Vec<sj::Value>>,
    pub show_window: Option<Vec<sj::Value>>,
}

// impl Extras {}

impl Clone for Extras {
    fn clone(&self) -> Self {
        Extras {
            desktop: self.desktop.clone(),
            title: self.title.clone(),
            x: self.x.clone(),
            y: self.y.clone(),
            x_size: self.x_size.clone(),
            y_size: self.y_size.clone(),
            x_count_chars: self.x_count_chars.clone(),
            y_count_chars: self.y_count_chars.clone(),
            fill_attribute: self.fill_attribute.clone(),
            flags: self.flags.clone(),
            show_window: self.show_window.clone(),
        }
    }
}
