//! Runtime type validation for command arguments.
//!
//! This module validates that command-line arguments match their declared types
//! before execution, providing clear error messages when types don't match.

use super::ast::Parameter;
use std::collections::HashMap;

/// Validates that a string value matches the expected parameter type.
///
/// # Arguments
///
/// * `value` - The string value from command line
/// * `param` - The parameter definition with type information
///
/// # Returns
///
/// Returns `Ok(validated_string)` if validation passes,
/// `Err(error_message)` with a user-friendly error if validation fails.
pub fn validate_argument_type(value: &str, param: &Parameter) -> Result<String, String> {
    match param.param_type.as_str() {
        "str" => Ok(value.to_string()),
        
        "bool" => {
            // Bool should already be handled by clap flags, but validate anyway
            let lower = value.to_lowercase();
            if lower == "true" || lower == "false" || lower == "1" || lower == "0" {
                Ok(lower)
            } else {
                Err(format!(
                    "Invalid boolean value '{}' for parameter '{}'. Expected 'true' or 'false'.",
                    value, param.name
                ))
            }
        },
        
        "num" => {
            // Try to parse as number
            if let Ok(_num) = value.parse::<f64>() {
                Ok(value.to_string())
            } else {
                Err(format!(
                    "Invalid number '{}' for parameter '{}'. Expected a numeric value (e.g., 42, 3.14, -10).",
                    value, param.name
                ))
            }
        },
        
        "arr" => {
            // Arrays are passed as comma-separated strings
            // Empty arrays are valid if they match the default
            let trimmed = value.trim();
            if trimmed.is_empty() {
                // Check if empty array is allowed (has default = [])
                if let Some(default) = &param.default {
                    if let super::ast::Value::Array(arr) = default {
                        if arr.is_empty() {
                            // Empty array matches empty default, it's valid
                            return Ok(value.to_string());
                        }
                    }
                }
                // Empty array without default is invalid
                return Err(format!(
                    "Empty array value for parameter '{}'. Provide comma-separated values (e.g., 'a,b,c').",
                    param.name
                ));
            }
            
            // Check if it looks like a valid array (comma-separated or single value)
            // We'll accept any non-empty string as it will be split later
            Ok(value.to_string())
        },
        
        _ => {
            Err(format!(
                "Unknown parameter type '{}' for parameter '{}'",
                param.param_type, param.name
            ))
        }
    }
}

/// Validates all arguments in a HashMap against their parameter definitions.
///
/// This function checks that all provided arguments match their declared types
/// and that all required parameters are present.
///
/// # Arguments
///
/// * `args` - HashMap of argument names to string values
/// * `parameters` - List of parameter definitions
/// * `command_path` - Path to the command (for error messages)
///
/// # Returns
///
/// Returns `Ok(validated_args)` if all validations pass,
/// `Err(errors)` with a list of validation errors if any fail.
pub fn validate_all_arguments(
    args: &HashMap<String, String>,
    parameters: &[Parameter],
    command_path: &[String],
) -> Result<HashMap<String, String>, Vec<String>> {
    let mut errors = Vec::new();
    let mut validated_args = HashMap::new();

    // Validate each provided argument
    for (arg_name, arg_value) in args {
        // Find the parameter definition
        if let Some(param) = parameters.iter().find(|p| &p.name == arg_name) {
            match validate_argument_type(arg_value, param) {
                Ok(validated) => {
                    validated_args.insert(arg_name.clone(), validated);
                }
                Err(error_msg) => {
                    let command_str = command_path.join(" ");
                    errors.push(format!(
                        "❌ Type validation error in command 'nest {}':\n   {}",
                        command_str, error_msg
                    ));
                }
            }
        }
        // If parameter not found, it might be from a different context, skip it
    }

    // Check for missing required parameters
    // Note: clap already handles required arguments, but we check here for completeness
    for param in parameters {
        if param.default.is_none() && !args.contains_key(&param.name) && !validated_args.contains_key(&param.name) {
            let command_str = command_path.join(" ");
            errors.push(format!(
                "❌ Missing required parameter '{}' for command 'nest {}'",
                param.name, command_str
            ));
        }
    }

    if errors.is_empty() {
        Ok(validated_args)
    } else {
        Err(errors)
    }
}

/// Parses an array string into a vector of strings.
///
/// Arrays are typically passed as comma-separated strings.
/// This function handles parsing and validation.
///
/// # Arguments
///
/// * `value` - The comma-separated string value
///
/// # Returns
///
/// Returns a vector of trimmed string values.
#[allow(dead_code)]
pub fn parse_array(value: &str) -> Vec<String> {
    if value.trim().is_empty() {
        return Vec::new();
    }
    
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Formats an array for display in error messages.
#[allow(dead_code)]
pub fn format_array_for_display(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", items.join(", "))
    }
}

