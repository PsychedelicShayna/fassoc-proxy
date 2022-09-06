use std::{collections::HashMap, path::Path};

use regex as re;
use serde_json as sj;

use serde::{Deserialize, Serialize};

// ----------------------------------------------------------------------------
// FassocRules
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub enum FindRuleError {
    CannotConvertPath(String), // In the future, make this take a &str
    NoMappingFound,
    NoRuleFound,
}

impl std::fmt::Display for FindRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindRuleError::CannotConvertPath(s) => write!(f, "Cannot convert path: {}", s),
            FindRuleError::NoMappingFound => write!(f, "No mapping found"),
            FindRuleError::NoRuleFound => write!(f, "No rule found"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FassocRules {
    pub mappings: HashMap<String, Vec<String>>,
    pub rules: HashMap<String, Rule>,
}

impl FassocRules {
    pub fn find_suitable_rule(&self, file_path: &Path) -> Result<Rule, FindRuleError> {
        // let file_path_str: String = file_path.to_str().map_or_else(
        //     || Err(FindRuleError::CannotConvertPath(String::from(""))),
        //     |s| Ok(String::from(s)),
        // )?;

        let file_name_str: String = file_path.file_name().and_then(|n| n.to_str()).map_or_else(
            || Err(FindRuleError::CannotConvertPath(String::from(""))),
            |s| Ok(String::from(s)),
        )?;

        let file_ext_str = file_path
            .extension()
            .and_then(|ext| ext.to_str().map(|s| String::from(s)));

        let fallback_mapping = self.mappings.get(&String::from("*"));
        let extension_mapping = file_ext_str.and_then(|s| self.mappings.get(&s));

        let final_mapping = match extension_mapping.or(fallback_mapping) {
            Some(mapping) => mapping,
            None => {
                return Err(FindRuleError::NoMappingFound);
            }
        };

        let (regexf_only, regexc_only, regexf_and_c, noregex) = final_mapping.iter().fold(
            (vec![], vec![], vec![], vec![]) as (Vec<&Rule>, Vec<&Rule>, Vec<&Rule>, Vec<&Rule>),
            |mut v, s| {
                self.rules.get(s).map(|r| {
                    if r.regexf.is_some() && r.regexc.is_none() {
                        v.0.push(r);
                    } else if r.regexf.is_none() && r.regexc.is_some() {
                        v.1.push(r);
                    } else if r.regexf.is_some() && r.regexc.is_some() {
                        v.2.push(r);
                    } else {
                        v.3.push(r)
                    }
                });

                v
            },
        );

        // Return the first rule that matches and only has a filename pattern.
        for rule in regexf_only {
            if rule
                .rmatch_file_name(file_name_str.to_owned())
                .unwrap_or(false)
            {
                return Ok(rule.to_owned());
            }
        }

        // File content is stored, so that it doesn't have to be read multiple
        // times. Reading is avoided unless needed, for performance reasons.

        let mut file_content: &mut Option<String> = &mut None;

        let ensure_contents_read = |content: &mut Option<String>| {
            if content.is_none() {
                *content = std::fs::read_to_string(file_path).map_or_else(
                    |e| {
                        log::error!("Failed to read file's contents: {}", e);
                        None
                    },
                    |s| Some(s),
                );
            }
        };

        // Return the first rule that matches and has a filename AND content pattern.
        for rule in regexf_and_c {
            if rule
                .rmatch_file_name(file_name_str.to_owned())
                .unwrap_or(false)
            {
                ensure_contents_read(&mut file_content);

                if file_content.as_ref().map_or(false, |content| {
                    rule.rmatch_file_content(content).unwrap_or(false)
                }) {
                    return Ok(rule.to_owned());
                }
            }
        }

        // Return the first rule that matches and only has a content pattern.
        for rule in regexc_only {
            ensure_contents_read(&mut file_content);

            if file_content.as_ref().map_or(false, |content| {
                rule.rmatch_file_content(content).unwrap_or(false)
            }) {
                return Ok(rule.to_owned());
            }
        }

        return match noregex.first() {
            Some(rule) => Ok(rule.to_owned().to_owned()),
            None => Err(FindRuleError::NoRuleFound),
        };
    }
}

// ----------------------------------------------------------------------------
// Rule
// ----------------------------------------------------------------------------
#[derive(Debug)]
pub enum RuleRegexError {
    RegexCompileError(re::Error),
    NoRegexError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    pub command: String,
    pub arguments: Option<String>,
    pub cwd: Option<String>,
    pub regexf: Option<String>,
    pub regexc: Option<String>,
    pub process_attributes: Option<SecurityAttributes>,
    pub thread_attributes: Option<SecurityAttributes>,
    pub inherit_handles: Option<bool>,
    pub creation_flags: Option<Vec<sj::Value>>,
    // pub environment: Option<Vec<sj::Value>>, -- This will be implemented later.
    pub extras: Option<Extras>,
}

impl Rule {
    fn rmatch_file(regstr: Option<String>, content: &String) -> Result<bool, RuleRegexError> {
        regstr.map_or(Err(RuleRegexError::NoRegexError), |regstr| {
            re::Regex::new(regstr.as_str()).map_or_else(
                |error| Err(RuleRegexError::RegexCompileError(error)),
                |regex| Ok(regex.is_match(content.as_str())),
            )
        })
    }

    pub fn rmatch_file_name(&self, file_name: String) -> Result<bool, RuleRegexError> {
        Rule::rmatch_file(self.regexf.to_owned(), &file_name)
    }

    pub fn rmatch_file_content(&self, file_content: &String) -> Result<bool, RuleRegexError> {
        Rule::rmatch_file(self.regexc.to_owned(), file_content)
    }
}

impl Clone for Rule {
    fn clone(&self) -> Self {
        Rule {
            command: self.command.clone(),
            arguments: self.arguments.clone(),
            cwd: self.cwd.clone(),
            regexf: self.regexf.clone(),
            regexc: self.regexc.clone(),
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
