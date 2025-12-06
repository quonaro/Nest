//! Argument extraction from parsed CLI arguments.
//!
//! This module handles extracting command arguments from clap's ArgMatches
//! and converting them into a format suitable for script execution.

use super::ast::Parameter;
use super::cli::CliGenerator;
use clap::ArgMatches;
use std::collections::HashMap;

/// Extracts arguments from parsed CLI matches.
///
/// This is a utility struct with static methods for argument extraction.
pub struct ArgumentExtractor;

impl ArgumentExtractor {
    /// Extracts arguments from clap matches for a regular command.
    ///
    /// This function processes all parameters and extracts their values from
    /// the parsed CLI arguments. For parameters without provided values,
    /// it uses default values if available.
    ///
    /// # Arguments
    ///
    /// * `matches` - The parsed CLI arguments from clap
    /// * `parameters` - List of parameters to extract
    /// * `generator` - CLI generator for parameter ID resolution
    ///
    /// # Returns
    ///
    /// Returns a HashMap of parameter names to their string values.
    /// Boolean parameters are converted to "true" or "false".
    pub fn extract_from_matches(
        matches: &ArgMatches,
        parameters: &[Parameter],
        generator: &CliGenerator,
    ) -> HashMap<String, String> {
        let mut args = HashMap::new();

        for param in parameters {
            if param.param_type == "bool" {
                let value = Self::extract_bool_flag(matches, param, generator);
                args.insert(param.name.clone(), value.to_string());
            } else {
                if let Some(value) = Self::extract_value_arg(matches, param, generator) {
                    args.insert(param.name.clone(), value);
                } else if let Some(default) = &param.default {
                    if let Some(default_str) = generator.value_to_string(default) {
                        args.insert(param.name.clone(), default_str);
                    }
                }
            }
        }

        args
    }

    /// Extracts arguments from clap matches for a default subcommand.
    ///
    /// This is similar to `extract_from_matches`, but handles the special case
    /// where arguments are passed to a parent group command but should be
    /// applied to the default subcommand.
    ///
    /// # Arguments
    ///
    /// * `matches` - The parsed CLI arguments from clap (for the parent group)
    /// * `parameters` - List of parameters from the default subcommand
    /// * `generator` - CLI generator for parameter ID resolution
    ///
    /// # Returns
    ///
    /// Returns a HashMap of parameter names to their string values.
    pub fn extract_for_default_command(
        matches: &ArgMatches,
        parameters: &[Parameter],
        generator: &CliGenerator,
    ) -> HashMap<String, String> {
        let mut args = HashMap::new();

        for param in parameters {
            if param.param_type == "bool" {
                let value = Self::extract_bool_flag_for_default(matches, param, generator);
                args.insert(param.name.clone(), value.to_string());
            } else {
                if let Some(value) = Self::extract_value_arg_for_default(matches, param, generator)
                {
                    args.insert(param.name.clone(), value);
                } else if let Some(default) = &param.default {
                    if let Some(default_str) = generator.value_to_string(default) {
                        args.insert(param.name.clone(), default_str);
                    }
                }
            }
        }

        args
    }

    fn extract_bool_flag(
        matches: &ArgMatches,
        param: &Parameter,
        generator: &CliGenerator,
    ) -> bool {
        let param_id = generator.get_param_id(&param.name);
        
        // Check if flag is present
        if matches.contains_id(param_id) {
            // If value is provided, parse it
            if let Some(value_str) = matches.get_one::<String>(param_id) {
                value_str == "true"
            } else {
                // Flag present without value means true
                true
            }
        } else {
            // Check alias
            param
                .alias
                .as_ref()
                .and_then(|a| a.chars().next())
                .map(|c| {
                    let alias_str = c.to_string();
                    if matches.contains_id(&alias_str) {
                        if let Some(value_str) = matches.get_one::<String>(&alias_str) {
                            value_str == "true"
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        }
    }

    fn extract_bool_flag_for_default(
        matches: &ArgMatches,
        param: &Parameter,
        generator: &CliGenerator,
    ) -> bool {
        let param_id = generator.get_param_id(&param.name);
        
        // Check if flag is present
        if matches.contains_id(param_id) {
            // If value is provided, parse it
            if let Some(value_str) = matches.get_one::<String>(param_id) {
                value_str == "true"
            } else {
                // Flag present without value means true
                true
            }
        } else {
            // Check alias
            param
                .alias
                .as_ref()
                .and_then(|a| a.chars().next())
                .map(|c| {
                    let alias_str = c.to_string();
                    if matches.contains_id(&alias_str) {
                        if let Some(value_str) = matches.get_one::<String>(&alias_str) {
                            value_str == "true"
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
        }
    }

    fn extract_value_arg(
        matches: &ArgMatches,
        param: &Parameter,
        _generator: &CliGenerator,
    ) -> Option<String> {
        matches
            .get_one::<String>(&param.name)
            .cloned()
            .or_else(|| {
                param.alias.as_ref().and_then(|alias| {
                    alias
                        .chars()
                        .next()
                        .and_then(|c| matches.get_one::<String>(&c.to_string()).cloned())
                })
            })
    }

    fn extract_value_arg_for_default(
        matches: &ArgMatches,
        param: &Parameter,
        generator: &CliGenerator,
    ) -> Option<String> {
        let param_id = generator.get_param_id(&param.name);
        matches
            .get_one::<String>(param_id)
            .cloned()
            .or_else(|| {
                param.alias.as_ref().and_then(|alias| {
                    alias
                        .chars()
                        .next()
                        .and_then(|c| matches.get_one::<String>(&c.to_string()).cloned())
                })
            })
    }
}

