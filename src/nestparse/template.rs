//! Template processing for variable substitution in scripts.
//!
//! This module handles replacing placeholders in scripts with actual values.
//! Supports parameter placeholders ({{param}}), variables ({{VAR}}), constants ({{CONST}}),
//! and special variables ({{now}}, {{user}}).
//! Also supports modifiers like {{var|sep:","}} to replace default separator (space) with a custom one.

use super::ast::{Constant, Variable};
use crate::constants::{
    DEFAULT_USER, ENV_VAR_USER, TEMPLATE_VAR_ERROR, TEMPLATE_VAR_NOW, TEMPLATE_VAR_USER,
};
use chrono::Utc;
use std::collections::HashMap;
use std::env;

/// Processes templates by replacing placeholders with actual values.
///
/// This is a utility struct with static methods for template processing.
pub struct TemplateProcessor;

impl TemplateProcessor {
    /// Processes a script template by replacing placeholders with values.
    ///
    /// Supported placeholders:
    /// - `{{param}}` - Replaced with the value from `args` for key "param"
    /// - `{{VAR}}` - Replaced with variable value (can be redefined)
    /// - `{{CONST}}` - Replaced with constant value (cannot be redefined)
    /// - `{{now}}` - Replaced with current UTC time in RFC3339 format (only if not overridden)
    /// - `{{user}}` - Replaced with the USER environment variable (only if not overridden)
    /// - `{{SYSTEM_ERROR_MESSAGE}}` - Replaced with error message (available in fallback scripts)
    ///
    /// Supported modifiers for array values (arrays are space-separated by default):
    /// - `{{var|sep:","}}` - Replace spaces with comma (e.g., "redis celery backend" -> "redis,celery,backend")
    /// - `{{var|rep:" "=>","}}` - Explicit replacement: replace space with comma
    ///
    /// Priority order:
    /// 1. Parameters (from args) - highest priority
    /// 2. Parent parameters (from parent commands, nearest to farthest) - override parent variables/constants
    /// 3. Local variables (from command) - override parent and global variables
    /// 4. Local constants (from command) - override parent and global constants
    /// 5. Parent variables (from parent commands, nearest to farthest) - override global variables
    /// 6. Parent constants (from parent commands, nearest to farthest) - override global constants
    /// 7. Global variables (can be redefined, last definition wins)
    /// 8. Global constants (cannot be redefined)
    /// 9. Special variables ({{now}}, {{user}}) - lowest priority, only if not defined above
    ///
    /// # Arguments
    ///
    /// * `script` - The script template containing placeholders
    /// * `args` - HashMap of parameter names to values for replacement
    /// * `global_variables` - List of global variables
    /// * `global_constants` - List of global constants
    /// * `local_variables` - List of local variables (from command, optional)
    /// * `local_constants` - List of local constants (from command, optional)
    /// * `parent_variables` - List of parent variables (from parent commands, optional)
    /// * `parent_constants` - List of parent constants (from parent commands, optional)
    /// * `parent_args` - HashMap of parent command parameter names to values (from parent commands, optional)
    ///
    /// # Returns
    ///
    /// Returns the processed script with all placeholders replaced.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// let mut args = HashMap::new();
    /// args.insert("name".to_string(), "world".to_string());
    /// let script = "echo Hello {{name}}!";
    /// let processed = TemplateProcessor::process(script, &args, &[], &[], &[], &[], &[], &[], &std::collections::HashMap::new());
    /// assert_eq!(processed, "echo Hello world!");
    /// ```
    pub fn process(
        script: &str,
        args: &HashMap<String, String>,
        global_variables: &[Variable],
        global_constants: &[Constant],
        local_variables: &[Variable],
        local_constants: &[Constant],
        parent_variables: &[Variable],
        parent_constants: &[Constant],
        parent_args: &HashMap<String, String>,
    ) -> String {
        let mut processed = script.to_string();

        // Build variable map with priority: local > parent > global > special
        let mut var_map: HashMap<String, String> = HashMap::new();

        // 1. Add global constants first (lowest priority for constants)
        for constant in global_constants {
            var_map.insert(constant.name.clone(), constant.value.clone());
        }

        // 2. Add global variables (can override global constants)
        for variable in global_variables {
            var_map.insert(variable.name.clone(), variable.value.clone());
        }

        // 3. Add parent constants (override global constants/variables)
        for constant in parent_constants {
            var_map.insert(constant.name.clone(), constant.value.clone());
        }

        // 4. Add parent variables (override parent constants and global variables)
        for variable in parent_variables {
            var_map.insert(variable.name.clone(), variable.value.clone());
        }

        // 5. Add local constants (override parent and global constants/variables)
        for constant in local_constants {
            var_map.insert(constant.name.clone(), constant.value.clone());
        }

        // 6. Add local variables (highest priority for variables, override everything)
        for variable in local_variables {
            var_map.insert(variable.name.clone(), variable.value.clone());
        }

        // Build combined map for all replacements (args + parent_args)
        let mut all_args = args.clone();
        for (key, value) in parent_args {
            if !all_args.contains_key(key) {
                all_args.insert(key.clone(), value.clone());
            }
        }

        // Process placeholders: first modifiers, then simple ones
        // Use a single pass that handles both types correctly
        processed = Self::process_all_placeholders(&processed, &all_args, &var_map);

        // Replace shell-style $* with wildcard arguments (for compatibility)
        // This allows using $* in scripts instead of {{*}}
        if let Some(wildcard_value) = args.get("*") {
            // Replace $* (not part of a larger variable name)
            // Use regex-like replacement: $* at word boundaries or end of string
            let mut result = String::with_capacity(processed.len() + wildcard_value.len());
            let mut chars = processed.chars().peekable();

            while let Some(ch) = chars.next() {
                if ch == '$' {
                    if let Some(&'*') = chars.peek() {
                        // Found $*, replace it
                        chars.next(); // consume '*'
                        result.push_str(wildcard_value);
                        continue;
                    }
                }
                result.push(ch);
            }
            processed = result;
        }
        

        // Replace special variables (lowest priority, only if not already defined)
        // Check if "now" is already in var_map, if not, use special variable
        if !var_map.contains_key("now") {
            processed = processed.replace(TEMPLATE_VAR_NOW, &Utc::now().to_rfc3339());
        }

        // Check if "user" is already in var_map, if not, use special variable
        if !var_map.contains_key("user") {
            processed = processed.replace(
                TEMPLATE_VAR_USER,
                &env::var(ENV_VAR_USER).unwrap_or_else(|_| DEFAULT_USER.to_string()),
            );
        }

        // Replace system error message (from args if available, otherwise empty)
        // This variable is available in fallback scripts
        if let Some(error_msg) = args.get("SYSTEM_ERROR_MESSAGE") {
            processed = processed.replace(TEMPLATE_VAR_ERROR, error_msg);
        } else {
            processed = processed.replace(TEMPLATE_VAR_ERROR, "");
        }

        processed
    }

    /// Processes all placeholders in the script, handling both modifiers and simple placeholders.
    ///
    /// Supports formats:
    /// - `{{var}}` - simple replacement
    /// - `{{var|sep:","}}` - replace default separator (space) with comma
    /// - `{{var|rep:" "=>","}}` - explicit replacement: replace space with comma
    ///
    /// Returns a new string with all placeholders replaced.
    fn process_all_placeholders(
        script: &str,
        args: &HashMap<String, String>,
        var_map: &HashMap<String, String>,
    ) -> String {
        let mut combined_map = HashMap::new();
        
        // Combine args and var_map (args have priority)
        for (k, v) in var_map {
            combined_map.insert(k.clone(), v.clone());
        }
        for (k, v) in args {
            combined_map.insert(k.clone(), v.clone());
        }
        
        let mut result = String::with_capacity(script.len() * 2);
        let mut chars = script.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Check if it's {{
                if let Some(&'{') = chars.peek() {
                    chars.next(); // consume second {
                    
                    // Collect placeholder content until }}
                    let mut placeholder_content = String::new();
                    let mut found_close = false;
                    
                    while let Some(ch) = chars.next() {
                        if ch == '}' {
                            if let Some(&'}') = chars.peek() {
                                chars.next(); // consume second }
                                found_close = true;
                                break;
                            } else {
                                placeholder_content.push(ch);
                            }
                        } else {
                            placeholder_content.push(ch);
                        }
                    }
                    
                    if found_close {
                        // Parse and replace placeholder
                        let trimmed = placeholder_content.trim();
                        
                        // Check if it has modifiers
                        if let Some(pipe_pos) = trimmed.find('|') {
                            let var_name = trimmed[..pipe_pos].trim();
                            let modifier_part = trimmed[pipe_pos + 1..].trim();
                            
                            if let Some(value) = combined_map.get(var_name) {
                                let modified_value = if modifier_part.starts_with("sep:") {
                                    // sep modifier: {{var|sep:","}}
                                    let sep_value = &modifier_part[4..];
                                    let sep = Self::parse_modifier_value(sep_value);
                                    value.replace(" ", &sep)
                                } else if modifier_part.starts_with("rep:") {
                                    // rep modifier: {{var|rep:"from"=>"to"}}
                                    let rep_value = &modifier_part[4..];
                                    if let Some((from, to)) = Self::parse_rep_modifier(rep_value) {
                                        value.replace(&from, &to)
                                    } else {
                                        // Invalid modifier, keep original value
                                        value.clone()
                                    }
                                } else {
                                    // Unknown modifier, keep original value
                                    value.clone()
                                };
                                
                                result.push_str(&modified_value);
                            } else {
                                // Variable not found, keep placeholder as is
                                result.push_str("{{");
                                result.push_str(&trimmed);
                                result.push_str("}}");
                            }
                        } else {
                            // Simple placeholder: {{var}}
                            if let Some(value) = combined_map.get(trimmed) {
                                result.push_str(value);
                            } else {
                                // Variable not found, keep placeholder as is
                                result.push_str("{{");
                                result.push_str(trimmed);
                                result.push_str("}}");
                            }
                        }
                    } else {
                        // No closing }}, keep as is
                        result.push('{');
                        result.push('{');
                        result.push_str(&placeholder_content);
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Parses modifier value, removing quotes if present.
    /// Examples: "," -> ",", "\",\"" -> ","
    fn parse_modifier_value(value: &str) -> String {
        let trimmed = value.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Parses rep modifier in format "from"=>"to".
    /// Returns Some((from, to)) if successful, None otherwise.
    fn parse_rep_modifier(modifier: &str) -> Option<(String, String)> {
        // Look for => separator
        if let Some(arrow_pos) = modifier.find("=>") {
            let from_part = modifier[..arrow_pos].trim();
            let to_part = modifier[arrow_pos + 2..].trim();
            
            let from = Self::parse_modifier_value(from_part);
            let to = Self::parse_modifier_value(to_part);
            
            Some((from, to))
        } else {
            None
        }
    }
}
