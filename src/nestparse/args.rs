//! Argument extraction from parsed CLI arguments.
//!
//! This module handles extracting command arguments from clap's ArgMatches
//! and converting them into a format suitable for script execution.

use super::ast::{ParamKind, Parameter};
use super::cli::CliGenerator;
use super::type_validator;
use crate::constants::BOOL_TRUE;
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
    /// * `command_path` - Path to the command (for validation errors)
    ///
    /// # Returns
    ///
    /// Returns `Ok(HashMap)` with validated parameter names to their string values,
    /// or `Err(Vec<String>)` with validation error messages.
    pub fn extract_from_matches(
        matches: &ArgMatches,
        parameters: &[Parameter],
        generator: &CliGenerator,
        command_path: &[String],
    ) -> Result<HashMap<String, String>, Vec<String>> {
        let mut args = HashMap::new();
        let mut custom_errors: Vec<String> = Vec::new();

        for param in parameters {
            match &param.kind {
                ParamKind::Normal => {
                    if param.param_type == "bool" {
                        let value = if param.is_named {
                            Self::extract_bool_flag(matches, param, generator)
                        } else {
                            // Positional bool arguments
                            Self::extract_bool_positional(matches, param, generator)
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
                ParamKind::Wildcard { name: _, count } => {
                    // Wildcard parameters collect multiple positional values.
                    let id = &param.name;
                    let collected: Vec<String> = matches
                        .get_many::<String>(id)
                        .map(|vals| vals.cloned().collect())
                        .unwrap_or_else(Vec::new);

                    if let Some(expected) = count {
                        if collected.len() != *expected {
                            let command_str = command_path.join(" ");
                            custom_errors.push(format!(
                                "❌ Wildcard parameter '{}' in command 'nest {}' expects exactly {} argument(s), but got {}.",
                                param.name,
                                command_str,
                                expected,
                                collected.len()
                            ));
                            continue;
                        }
                    }

                    let joined = collected.join(" ");
                    args.insert(param.name.clone(), joined);
                }
            }
        }

        // Validate all arguments against their types
        match type_validator::validate_all_arguments(&args, parameters, command_path) {
            Ok(validated) => {
                if custom_errors.is_empty() {
                    Ok(validated)
                } else {
                    Err(custom_errors)
                }
            }
            Err(mut type_errors) => {
                type_errors.extend(custom_errors);
                Err(type_errors)
            }
        }
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
    /// * `command_path` - Path to the command (for validation errors)
    ///
    /// # Returns
    ///
    /// Returns `Ok(HashMap)` with validated parameter names to their string values,
    /// or `Err(Vec<String>)` with validation error messages.
    pub fn extract_for_default_command(
        matches: &ArgMatches,
        parameters: &[Parameter],
        generator: &CliGenerator,
        command_path: &[String],
    ) -> Result<HashMap<String, String>, Vec<String>> {
        let mut args = HashMap::new();
        let mut custom_errors: Vec<String> = Vec::new();

        for param in parameters {
            match &param.kind {
                ParamKind::Normal => {
                    if param.param_type == "bool" {
                        let value = if param.is_named {
                            Self::extract_bool_flag_for_default(matches, param, generator)
                        } else {
                            Self::extract_bool_positional(matches, param, generator)
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
                ParamKind::Wildcard { name: _, count } => {
                    let id = &param.name;
                    let collected: Vec<String> = matches
                        .get_many::<String>(id)
                        .map(|vals| vals.cloned().collect())
                        .unwrap_or_else(Vec::new);

                    if let Some(expected) = count {
                        if collected.len() != *expected {
                            let command_str = command_path.join(" ");
                            custom_errors.push(format!(
                                "❌ Wildcard parameter '{}' in command 'nest {}' expects exactly {} argument(s), but got {}.",
                                param.name,
                                command_str,
                                expected,
                                collected.len()
                            ));
                            continue;
                        }
                    }

                    let joined = collected.join(" ");
                    args.insert(param.name.clone(), joined);
                }
            }
        }

        // Validate all arguments against their types
        match type_validator::validate_all_arguments(&args, parameters, command_path) {
            Ok(validated) => {
                if custom_errors.is_empty() {
                    Ok(validated)
                } else {
                    Err(custom_errors)
                }
            }
            Err(mut type_errors) => {
                type_errors.extend(custom_errors);
                Err(type_errors)
            }
        }
    }

    fn extract_bool_flag(
        matches: &ArgMatches,
        param: &Parameter,
        generator: &CliGenerator,
    ) -> bool {
        // Use parameter name directly as ID (same as used in parameter_to_arg_with_id)
        let param_id = &param.name;

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
            // If not found, check default value
            if let Some(default) = &param.default {
                if let Some(default_str) = generator.value_to_string(default) {
                    default_str == BOOL_TRUE
                } else {
                    false
                }
            } else {
                false
            }
        }
    }

    fn extract_bool_flag_for_default(
        matches: &ArgMatches,
        param: &Parameter,
        generator: &CliGenerator,
    ) -> bool {
        // Use parameter name directly as ID (same as used in parameter_to_arg_with_id)
        let param_id = &param.name;

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
            // If not found, check default value
            if let Some(default) = &param.default {
                if let Some(default_str) = generator.value_to_string(default) {
                    default_str == BOOL_TRUE
                } else {
                    false
                }
            } else {
                false
            }
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

    fn extract_value_arg_positional(matches: &ArgMatches, param: &Parameter) -> Option<String> {
        // Positional arguments are accessible by their name
        matches.get_one::<String>(&param.name).cloned()
    }

    fn extract_bool_positional(
        matches: &ArgMatches,
        param: &Parameter,
        generator: &CliGenerator,
    ) -> bool {
        // For positional bool arguments, check if value exists
        if let Some(value_str) = matches.get_one::<String>(&param.name) {
            value_str == BOOL_TRUE
        } else {
            // If not provided, check default value
            if let Some(default) = &param.default {
                if let Some(default_str) = generator.value_to_string(default) {
                    default_str == BOOL_TRUE
                } else {
                    false
                }
            } else {
                false
            }
        }
    }

    fn extract_value_arg_for_default_named(matches: &ArgMatches, param_id: &str) -> Option<String> {
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
