//! Configuration file validator with detailed error reporting.
//!
//! This module validates the parsed configuration and provides
//! detailed error messages with line numbers and helpful suggestions.

use super::ast::{Command, Directive};
use super::output::colors;
use crate::constants::{RESERVED_SHORT_OPTIONS, RESERVED_WORDS};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Represents a validation error with location information.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Line number where the error was found (1-based)
    /// Note: Currently not used, reserved for future parser improvements
    #[allow(dead_code)]
    pub line: usize,
    /// Column number (1-based, approximate)
    /// Note: Currently not used, reserved for future parser improvements
    #[allow(dead_code)]
    pub column: Option<usize>,
    /// Error message
    pub message: String,
    /// Helpful suggestion to fix the error
    pub suggestion: Option<String>,
    /// Command path where error occurred
    pub command_path: Vec<String>,
}

/// Validates a list of commands and returns any errors found.
///
/// This function performs comprehensive validation including:
/// - Duplicate command names
/// - Reserved word usage
/// - Alias conflicts
/// - Invalid parameter types
/// - Missing required directives
/// - Invalid paths
///
/// # Arguments
///
/// * `commands` - List of commands to validate
/// * `file_path` - Path to the configuration file (for path validation)
///
/// # Returns
///
/// Returns `Ok(())` if validation passes,
/// `Err(errors)` with a list of validation errors if any are found.
pub fn validate_commands(
    commands: &[Command],
    file_path: &Path,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut command_names = HashMap::new();
    let mut all_aliases = HashMap::new();

    // Validate each top-level command
    for command in commands {
        validate_command_recursive(
            command,
            &[],
            &mut command_names,
            &mut all_aliases,
            &mut errors,
            file_path,
            None,
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_command_recursive(
    command: &Command,
    parent_path: &[String],
    command_names: &mut HashMap<String, Vec<String>>,
    all_aliases: &mut HashMap<String, (Vec<String>, String)>,
    errors: &mut Vec<ValidationError>,
    file_path: &Path,
    parent_source_file: Option<&std::path::Path>,
) {
    let current_path = {
        let mut path = parent_path.to_vec();
        path.push(command.name.clone());
        path
    };

    let full_name = current_path.join(" ");

    // Check for reserved words (but allow "default" as subcommand name)
    // "default" is only reserved at top level, not as a subcommand
    if RESERVED_WORDS.contains(&command.name.as_str()) && parent_path.is_empty() {
        errors.push(ValidationError {
            line: 1, // Approximate, parser doesn't track line numbers yet
            column: None,
            message: format!(
                "Command name '{}' is reserved and cannot be used as a top-level command",
                command.name
            ),
            suggestion: Some(format!(
                "Use a different name. Reserved words: {}",
                RESERVED_WORDS.join(", ")
            )),
            command_path: current_path.clone(),
        });
    }

    // Check for duplicate command names at the same level (same parent)
    // Commands with the same name but different parents are allowed
    let path_str = current_path.join(" ");
    let parent_str = if parent_path.is_empty() {
        String::new()
    } else {
        parent_path.join(" ")
    };

    // Check for duplicates only within the same parent context
    if let Some(existing_paths) = command_names.get(&command.name) {
        for existing_path in existing_paths {
            // Extract parent from existing path
            let existing_parent = if let Some((parent, _)) = existing_path.rsplit_once(' ') {
                parent
            } else {
                ""
            };

            // Only report conflict if same parent
            if existing_parent == parent_str {
                errors.push(ValidationError {
                    line: 1,
                    column: None,
                    message: format!(
                        "Duplicate command name '{}' found at the same level",
                        command.name
                    ),
                    suggestion: Some(format!(
                        "Command '{}' is already defined at path '{}'. Use a unique name.",
                        command.name, existing_path
                    )),
                    command_path: current_path.clone(),
                });
            }
        }
    }
    command_names
        .entry(command.name.clone())
        .or_default()
        .push(path_str);

    // Validate parameters
    let mut param_names = HashSet::new();
    for param in &command.parameters {
        // Check for duplicate parameter names
        if !param_names.insert(&param.name) {
            errors.push(ValidationError {
                line: 1,
                column: None,
                message: format!(
                    "Duplicate parameter name '{}' in command '{}'",
                    param.name, full_name
                ),
                suggestion: Some("Each parameter must have a unique name".to_string()),
                command_path: current_path.clone(),
            });
        }

        // Validate parameter type (wildcards always use internal type "arr")
        if !matches!(param.param_type.as_str(), "str" | "bool" | "num" | "arr") {
            errors.push(ValidationError {
                line: 1,
                column: None,
                message: format!(
                    "Invalid parameter type '{}' for parameter '{}'",
                    param.param_type, param.name
                ),
                suggestion: Some("Valid types are: str, bool, num, arr".to_string()),
                command_path: current_path.clone(),
            });
        }

        // Additional structural validation for wildcard parameters
        if let super::ast::ParamKind::Wildcard { name: _, count } = &param.kind {
            if let Some(c) = count {
                if *c == 0 {
                    errors.push(ValidationError {
                        line: 1,
                        column: None,
                        message: format!(
                            "Wildcard parameter '{}' must capture at least 1 argument (found [0])",
                            param.name
                        ),
                        suggestion: Some(
                            "Use a positive integer in the [N] specifier, e.g., *name[1] or *[2]"
                                .to_string(),
                        ),
                        command_path: current_path.clone(),
                    });
                }
            }
        }

        // Validate alias
        if let Some(alias) = &param.alias {
            // Check if alias is empty
            if alias.is_empty() {
                errors.push(ValidationError {
                    line: 1,
                    column: None,
                    message: format!("Empty alias for parameter '{}'", param.name),
                    suggestion: Some("Remove the alias or provide a single character".to_string()),
                    command_path: current_path.clone(),
                });
            } else if alias.len() != 1 {
                // Check if alias is a single character
                errors.push(ValidationError {
                    line: 1,
                    column: None,
                    message: format!(
                        "Alias '{}' for parameter '{}' must be a single character",
                        alias, param.name
                    ),
                    suggestion: Some(
                        "Use a single character alias, e.g., 'f' for 'force'".to_string(),
                    ),
                    command_path: current_path.clone(),
                });
            } else if let Some(alias_char) = alias.chars().next() {
                // Check if alias conflicts with reserved short options
                if RESERVED_SHORT_OPTIONS.contains(&alias_char) {
                    errors.push(ValidationError {
                        line: 1,
                        column: None,
                        message: format!(
                            "Alias '{}' for parameter '{}' conflicts with reserved option",
                            alias, param.name
                        ),
                        suggestion: Some(format!(
                            "Reserved short options are: {}. Use a different alias.",
                            RESERVED_SHORT_OPTIONS
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                        command_path: current_path.clone(),
                    });
                }

                // Check for alias conflicts within the same command
                // Different commands can use the same alias - that's allowed
                if let Some((existing_path, existing_param)) = all_aliases.get(alias) {
                    // Only report conflict if it's in the same command path
                    if existing_path == &current_path {
                        errors.push(ValidationError {
                            line: 1,
                            column: None,
                            message: format!(
                                "Alias '{}' conflicts with parameter '{}' in the same command '{}'",
                                alias,
                                existing_param,
                                current_path.join(" ")
                            ),
                            suggestion: Some(format!(
                                "Use a different alias for parameter '{}'",
                                param.name
                            )),
                            command_path: current_path.clone(),
                        });
                    }
                } else {
                    all_aliases.insert(alias.clone(), (current_path.clone(), param.name.clone()));
                }
            }
        }

        // Validate default value type matches parameter type
        if let Some(default) = &param.default {
            if !value_matches_type(default, &param.param_type) {
                errors.push(ValidationError {
                    line: 1,
                    column: None,
                    message: format!(
                        "Default value for parameter '{}' doesn't match type '{}'",
                        param.name, param.param_type
                    ),
                    suggestion: Some(format!(
                        "Expected a {} value, but got {:?}",
                        param.param_type, default
                    )),
                    command_path: current_path.clone(),
                });
            }
        }
    }

    // Validate directives
    let mut has_script = false;
    let mut cwd_paths = Vec::new();
    let mut env_files = Vec::new();

    for directive in &command.directives {
        match directive {
            Directive::Script(_) | Directive::ScriptHide(_) => has_script = true,
            Directive::Desc(_) => {}
            Directive::Depends(_) => {}
            Directive::Before(_) | Directive::BeforeHide(_) => {}
            Directive::After(_) | Directive::AfterHide(_) => {}
            Directive::Fallback(_) | Directive::FallbackHide(_) => {}
            Directive::Finaly(_) | Directive::FinalyHide(_) => {}
            Directive::Validate(_) => {}
            Directive::Privileged(_) => {}
            Directive::RequireConfirm(_) => {}
            Directive::Cwd(path) => {
                cwd_paths.push(path.clone());
            }
            Directive::Env(env_value) => {
                // Check if it's a file path (starts with .)
                if env_value.starts_with('.') {
                    env_files.push(env_value.clone());
                }
            }
            Directive::Logs(_, _) => {}
            Directive::If(_) => {}
            Directive::Elif(_) => {}
            Directive::Else => {}
        }
    }

    // Check for multiple cwd directives
    if cwd_paths.len() > 1 {
        errors.push(ValidationError {
            line: 1,
            column: None,
            message: format!("Multiple 'cwd' directives found in command '{}'", full_name),
            suggestion: Some("Use only one 'cwd' directive per command".to_string()),
            command_path: current_path.clone(),
        });
    }

    // Validate cwd path exists (if specified)
    // Use source_file from command if available, otherwise fall back to file_path
    if let Some(cwd) = cwd_paths.first() {
        // Determine which file path to use for validation
        let source_file_for_validation: Option<std::path::PathBuf> = command
            .source_file
            .clone()
            .or_else(|| parent_source_file.map(|p| p.to_path_buf()))
            .or_else(|| Some(file_path.to_path_buf()));

        if let Some(source_file) = source_file_for_validation {
            if let Some(parent) = source_file.parent() {
                let full_cwd = parent.join(cwd);
                if !full_cwd.exists() {
                    errors.push(ValidationError {
                        line: 1,
                        column: None,
                        message: format!(
                            "Working directory '{}' does not exist for command '{}'",
                            cwd, full_name
                        ),
                        suggestion: Some(format!(
                            "Create the directory or fix the path. Full path: {}",
                            full_cwd.display()
                        )),
                        command_path: current_path.clone(),
                    });
                }
            }
        }
    }

    // Validate .env files exist (if specified)
    // Note: This is a warning, not an error, as .env files are often optional
    // We'll check but not block execution
    for env_file in &env_files {
        if let Some(parent) = file_path.parent() {
            let full_env_path = parent.join(env_file);
            if !full_env_path.exists() {
                // This is informational - .env files might be created later
                // We could make this a warning instead of error, but for now keep as error
                // to help users catch configuration issues early
                errors.push(ValidationError {
                    line: 1,
                    column: None,
                    message: format!(
                        "Environment file '{}' does not exist for command '{}'",
                        env_file, full_name
                    ),
                    suggestion: Some(format!(
                        "Create the file or fix the path. Full path: {}. Note: This file is optional and may be created later.",
                        full_env_path.display()
                    )),
                    command_path: current_path.clone(),
                });
            }
        }
    }

    // Warn if command has no script (unless it's a group command)
    if !has_script && command.children.is_empty() {
        errors.push(ValidationError {
            line: 1,
            column: None,
            message: format!("Command '{}' has no script directive", full_name),
            suggestion: Some(
                "Add a 'script' directive or make this a group command with subcommands"
                    .to_string(),
            ),
            command_path: current_path.clone(),
        });
    }

    // Warn if group command has script (usually not needed)
    if has_script && !command.children.is_empty() {
        errors.push(ValidationError {
            line: 1,
            column: None,
            message: format!(
                "Group command '{}' has a script directive (usually not needed)",
                full_name
            ),
            suggestion: Some(
                "Group commands typically don't need scripts. Remove the script directive or make it a regular command".to_string()
            ),
            command_path: current_path.clone(),
        });
    }

    // Validate child commands
    for child in &command.children {
        // Use child's source_file if available, otherwise use parent's source_file or command's source_file
        let child_source_file: Option<&std::path::Path> = child
            .source_file
            .as_deref()
            .or_else(|| command.source_file.as_deref())
            .or(parent_source_file);

        validate_command_recursive(
            child,
            &current_path,
            command_names,
            all_aliases,
            errors,
            file_path,
            child_source_file,
        );
    }
}

fn value_matches_type(value: &super::ast::Value, param_type: &str) -> bool {
    matches!(
        (value, param_type),
        (super::ast::Value::String(_), "str")
            | (super::ast::Value::Bool(_), "bool")
            | (super::ast::Value::Number(_), "num")
            | (super::ast::Value::Array(_), "arr")
    )
}

/// Formats and prints validation errors in a user-friendly way.
pub fn print_validation_errors(errors: &[ValidationError], file_path: &Path) {
    use std::fmt::Write;

    let mut output = String::new();

    writeln!(
        output,
        "\n{}╔═══════════════════════════════════════════════════════════════╗{}",
        colors::RED,
        colors::RESET
    )
    .expect("Failed to format validation error header");
    writeln!(
        output,
        "{}║{}  {}❌ Configuration Validation Errors{}",
        colors::RED,
        colors::RESET,
        colors::BRIGHT_RED,
        colors::RESET
    )
    .expect("Failed to format validation error title");
    writeln!(
        output,
        "{}╚═══════════════════════════════════════════════════════════════╝{}\n",
        colors::RED,
        colors::RESET
    )
    .expect("Failed to format validation error footer");

    writeln!(
        output,
        "{}📄 File:{} {}",
        colors::CYAN,
        colors::RESET,
        file_path.display()
    )
    .expect("Failed to format file path in validation error");
    writeln!(
        output,
        "{}❌ Found {}{} error(s){}\n",
        colors::BRIGHT_RED,
        colors::RESET,
        errors.len(),
        colors::RESET
    )
    .expect("Failed to format error count in validation error");

    for (idx, error) in errors.iter().enumerate() {
        writeln!(
            output,
            "{}[{}]{} {}",
            colors::YELLOW,
            idx + 1,
            colors::RESET,
            error.message
        )
        .expect("Failed to format validation error message");

        if !error.command_path.is_empty() {
            writeln!(
                output,
                "   {}Command:{} {}nest {}{}",
                colors::GRAY,
                colors::RESET,
                colors::BRIGHT_BLUE,
                error.command_path.join(" "),
                colors::RESET
            )
            .expect("Failed to format command path in validation error");
        }

        if let Some(suggestion) = &error.suggestion {
            writeln!(
                output,
                "   {}💡 Suggestion:{} {}{}{}",
                colors::BRIGHT_CYAN,
                colors::RESET,
                colors::GRAY,
                suggestion,
                colors::RESET
            )
            .expect("Failed to format suggestion in validation error");
        }

        if idx < errors.len() - 1 {
            writeln!(output).expect("Failed to add newline in validation error output");
        }
    }

    writeln!(
        output,
        "\n{}ℹ{} {}Fix the errors above and try again.{}",
        colors::BRIGHT_CYAN,
        colors::RESET,
        colors::GRAY,
        colors::RESET
    )
    .expect("Failed to format validation error footer message");

    eprint!("{}", output);
}
