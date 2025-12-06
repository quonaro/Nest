//! Argument extraction from parsed CLI arguments.
//!
//! This module handles extracting command arguments from clap's ArgMatches
//! and converting them into a format suitable for script execution.

use crate::constants::BOOL_TRUE;
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
                let value = if param.is_named {
                    Self::extract_bool_flag(matches, param, generator)
                } else {
                    // Positional bool arguments
                    Self::extract_bool_positional(matches, param)
                };
                args.insert(param.name.clone(), value.to_string());
            } else {
                let value = if param.is_named {
                    // For named arguments, use param_id from generator
                    let param_id = generator.get_param_id(&param.name);
                    Self::extract_value_arg_named(matches, param, param_id)
                } else {
                    // Positional arguments are accessible by name
                    Self::extract_value_arg_positional(matches, param)
                };
                
                if let Some(value) = value {
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
                let value = if param.is_named {
                    Self::extract_bool_flag_for_default(matches, param, generator)
                } else {
                    Self::extract_bool_positional(matches, param)
                };
                args.insert(param.name.clone(), value.to_string());
            } else {
                let value = if param.is_named {
                    // For named arguments, use param_id from generator
                    let param_id = generator.get_param_id(&param.name);
                    Self::extract_value_arg_for_default_named(matches, param_id)
                } else {
                    // Positional arguments are accessible by name
                    Self::extract_value_arg_for_default_positional(matches, param)
                };
                
                if let Some(value) = value {
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
        
        // Check if flag is present by param_id (works for both --flag and -f)
        if matches.contains_id(param_id) {
            // If value is provided, parse it
            if let Some(value_str) = matches.get_one::<String>(param_id) {
                value_str == BOOL_TRUE
            } else {
                // Flag present without value means true
                true
            }
        } else {
            // If not found, use default value
            false
        }
    }

    fn extract_bool_flag_for_default(
        matches: &ArgMatches,
        param: &Parameter,
        generator: &CliGenerator,
    ) -> bool {
        let param_id = generator.get_param_id(&param.name);
        
        // Check if flag is present by param_id (works for both --flag and -f)
        if matches.contains_id(param_id) {
            // If value is provided, parse it
            if let Some(value_str) = matches.get_one::<String>(param_id) {
                value_str == BOOL_TRUE
            } else {
                // Flag present without value means true
                true
            }
        } else {
            // If not found, use default value
            false
        }
    }

    fn extract_value_arg_named(
        matches: &ArgMatches,
        _param: &Parameter,
        param_id: &str,
    ) -> Option<String> {
        // For named arguments, clap uses the param_id (parameter name) as the ID
        // The alias is only used for the short option, but the ID remains the parameter name
        matches.get_one::<String>(param_id).cloned()
    }

    fn extract_value_arg_positional(
        matches: &ArgMatches,
        param: &Parameter,
    ) -> Option<String> {
        // Positional arguments are accessible by their name
        matches.get_one::<String>(&param.name).cloned()
    }

    fn extract_bool_positional(
        matches: &ArgMatches,
        param: &Parameter,
    ) -> bool {
        // For positional bool arguments, check if value exists
        if let Some(value_str) = matches.get_one::<String>(&param.name) {
            value_str == BOOL_TRUE
        } else {
            // If not provided and has default, use default
            // Otherwise false
            false
        }
    }

    fn extract_value_arg_for_default_named(
        matches: &ArgMatches,
        param_id: &str,
    ) -> Option<String> {
        // For named arguments, use the param_id directly
        matches.get_one::<String>(param_id).cloned()
    }

    fn extract_value_arg_for_default_positional(
        matches: &ArgMatches,
        param: &Parameter,
    ) -> Option<String> {
        // Positional arguments are accessible by their name
        matches.get_one::<String>(&param.name).cloned()
    }
}

