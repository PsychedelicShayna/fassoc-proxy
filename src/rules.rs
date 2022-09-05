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
    NoMath,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FassocRules {
    pub mappings: HashMap<String, Vec<String>>,
    pub rules: HashMap<String, Rule>,
}

impl FassocRules {
    pub fn find_suitable_rule(&self, file_path: &Path) -> Result<Rule, FindRuleError> {
        let file_path_str: String = file_path.to_str().map_or_else(
            || Err(FindRuleError::CannotConvertPath(String::from(""))),
            |s| Ok(String::from(s)),
        )?;

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

        let regexf_only = final_mapping.iter().fold(vec![] as Vec<&Rule>, |v, s| {
            self.rules.get(s).map(|r| {
                if r.regexf.is_some() && r.regexc.is_none() {
                    v.push(r);
                }
            });

            v
        });

        let (regexf_only, regexc_only, regexf_and_c, noregex) = final_mapping.iter().fold(
            (vec![], vec![], vec![], vec![]) as (Vec<&Rule>, Vec<&Rule>, Vec<&Rule>, Vec<&Rule>),
            |v, s| {
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
            if rule.rmatch_file_name(file_name_str).unwrap_or(false) {
                return Ok(*rule);
            }
        }

        // File content is stored, so that it doesn't have to be read multiple
        // times. Reading is avoided unless needed, for performance reasons.

        let mut file_content: Option<String> = None;

        let ensure_contents_read = || {
            if file_content.is_none() {
                file_content = std::fs::read_to_string(file_path).map_or_else(
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
            if rule.rmatch_file_name(file_name_str).unwrap_or(false) {
                ensure_contents_read();

                if file_content.map_or(false, |content| {
                    rule.rmatch_file_content(content).unwrap_or(false)
                }) {
                    return Ok(*rule);
                }
            }
        }

        // Return the first rule that matches and only has a content pattern.
        for rule in regexc_only {
            ensure_contents_read();

            if file_content.map_or(false, |content| {
                rule.rmatch_file_content(content).unwrap_or(false)
            }) {
                return Ok(*rule);
            }
        }

        return match noregex.first() {
            Some(rule) => Ok(*rule.to_owned()),
            None => Err(FindRuleError::NoRuleFound),
        };
    }
}

// ----------------------------------------------------------------------------
// Rule
// ----------------------------------------------------------------------------

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

#[derive(Debug)]
pub enum RuleRegexError {
    RegexCompileError(re::Error),
    NoRegexError,
}

impl Rule {
    pub fn rmatch_file(regstr: Option<String>, content: String) -> Result<bool, RuleRegexError> {
        regstr.map_or(Err(RuleRegexError::NoRegexError), |regstr| {
            re::Regex::new(regstr.as_str()).map_or_else(
                |error| Err(RuleRegexError::RegexCompileError(error)),
                |regex| Ok(regex.is_match(content.as_str())),
            )
        })
    }

    pub fn rmatch_file_name(self, file_name: String) -> Result<bool, RuleRegexError> {
        Rule::rmatch_file(self.regexf, file_name)
    }

    pub fn rmatch_file_content(self, file_content: String) -> Result<bool, RuleRegexError> {
        Rule::rmatch_file(self.regexc, file_content)
    }

    fn has_regex(&self) -> bool {
        self.regexf.is_some() || self.regexc.is_some()
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

impl SecurityAttributes {}

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
    pub show_window: Option<u16>,
}

impl Extras {}
