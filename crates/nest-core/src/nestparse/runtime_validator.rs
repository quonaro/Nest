//! Runtime validator for command arguments.
//!
//! This module handles the validation of command arguments against
//! regex patterns and other rules defined in directives.

use super::template::{TemplateContext, TemplateProcessor};
use regex::Regex;
use std::collections::HashMap;

/// Validator for runtime parameters.
pub struct RuntimeValidator;

impl RuntimeValidator {
    /// Validates command parameters according to validation directives.
    ///
    /// Supports format: "param_name matches /regex/"
    ///
    /// # Arguments
    ///
    /// * `validate_directives` - List of validation rules
    /// * `args` - Arguments to validate
    /// * `env_vars` - Environment variables
    /// * `tpl_context` - Template context for variable substitution
    /// * `command_path` - Command path for error messages
    /// * `parent_args` - Parent arguments
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all validations pass,
    /// `Err(message)` if validation fails.
    pub fn validate(
        validate_directives: &[(String, String)],
        args: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
        tpl_context: &TemplateContext,
        command_path: &[String],
        parent_args: &HashMap<String, String>,
    ) -> Result<(), String> {
        for (param_name, rule) in validate_directives {
            // Process templates in the pattern part (allows dynamic rules)
            let processed_pattern =
                TemplateProcessor::process(rule, args, tpl_context, parent_args, None);
            let pattern_part = processed_pattern.trim();

            // Determine the target value (either from args or environment)
            let target_value_result = if let Some(env_name) = param_name.strip_prefix('$') {
                // Check in Nest-defined session env vars first, then system env,
                // and finally in Nest variables (since root-level 'env' directives are stored as variables)
                env_vars
                    .get(env_name)
                    .cloned()
                    .or_else(|| std::env::var(env_name).ok())
                    .or_else(|| {
                        tpl_context
                            .parent_variables
                            .iter()
                            .find(|v| v.name == env_name)
                            .map(|v| v.value.to_string_unquoted())
                    })
                    .or_else(|| {
                        tpl_context
                            .global_variables
                            .iter()
                            .find(|v| v.name == env_name)
                            .map(|v| v.value.to_string_unquoted())
                    })
            } else {
                args.get(param_name).cloned()
            };

            let target_value = target_value_result.ok_or_else(|| {
                format!(
                    "Validation error: target '{}' not found in arguments or environment",
                    param_name
                )
            })?;

            // Check if it's a membership validation (in ["a", "b"])
            if let Some(list_part) = pattern_part.strip_prefix("in ") {
                let list_part = list_part.trim();
                if (list_part.starts_with('[') && list_part.ends_with(']'))
                    || (list_part.starts_with('(') && list_part.ends_with(')'))
                {
                    let content = &list_part[1..list_part.len() - 1];
                    let allowed_values: Vec<String> = content
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if (s.starts_with('"') && s.ends_with('"'))
                                || (s.starts_with('\'') && s.ends_with('\''))
                            {
                                s[1..s.len() - 1].to_string()
                            } else {
                                s.to_string()
                            }
                        })
                        .filter(|s| !s.is_empty())
                        .collect();

                    if !allowed_values.contains(&target_value) {
                        let command_str = command_path.join(" ");
                        return Err(format!(
                            "❌ Validation error in command 'nest {}':\n   Target '{}' with value '{}' is not in allowed list: [{}]",
                            command_str,
                            param_name,
                            target_value,
                            allowed_values.join(", ")
                        ));
                    }
                    continue; // Validation passed for this directive
                }
            }

            // Extract regex pattern from /pattern/ or /pattern/flags
            let pattern = if pattern_part.starts_with('/') && pattern_part.len() > 1 {
                // Find closing /
                let mut end_pos = None;
                let mut escaped = false;
                for (i, ch) in pattern_part[1..].char_indices() {
                    if escaped {
                        escaped = false;
                        continue;
                    }
                    if ch == '\\' {
                        escaped = true;
                        continue;
                    }
                    if ch == '/' {
                        end_pos = Some(i + 1);
                        break;
                    }
                }

                if let Some(end) = end_pos {
                    // Extract pattern (without leading /)
                    let pattern_str = &pattern_part[1..end - 1];
                    // Unescape the pattern
                    let unescaped = pattern_str.replace("\\/", "/");

                    // Check for flags after closing /
                    let flags = pattern_part[end..].trim();
                    let regex = if flags.is_empty() {
                        Regex::new(&unescaped)
                    } else {
                        // Parse flags (e.g., "i" for case-insensitive)
                        let mut regex_builder = regex::RegexBuilder::new(&unescaped);
                        if flags.contains('i') {
                            regex_builder.case_insensitive(true);
                        }
                        regex_builder.build()
                    };

                    match regex {
                        Ok(re) => re,
                        Err(e) => {
                            return Err(format!(
                                "Invalid regex pattern in validation rule for '{}': '{}'. Error: {}",
                                param_name, pattern_part, e
                            ));
                        }
                    }
                } else {
                    // No closing slash found - treat whole string as regex
                    match Regex::new(pattern_part) {
                        Ok(re) => re,
                        Err(e) => {
                            return Err(format!(
                                "Invalid regex pattern: {}. Error: {}",
                                pattern_part, e
                            ))
                        }
                    }
                }
            } else {
                // simple pattern, try to compile as is
                match Regex::new(pattern_part) {
                    Ok(re) => re,
                    Err(e) => {
                        return Err(format!(
                            "Invalid regex pattern: {}. Error: {}",
                            pattern_part, e
                        ))
                    }
                }
            };

            // Validate
            if !pattern.is_match(&target_value) {
                let command_str = command_path.join(" ");
                let target_type = if param_name.starts_with('$') {
                    "Environment variable"
                } else {
                    "Parameter"
                };
                return Err(format!(
                    "❌ Validation error in command 'nest {}':\n   {} '{}' with value '{}' does not match pattern '{}'",
                    command_str, target_type, param_name, target_value, pattern_part
                ));
            }
        }

        Ok(())
    }
}
