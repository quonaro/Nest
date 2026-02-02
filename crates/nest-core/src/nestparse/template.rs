//! Template processing for variable substitution in scripts.
//!
//! This module handles replacing placeholders in scripts with actual values.
//! Supports parameter placeholders ({{param}}), variables ({{VAR}}), constants ({{CONST}}),
//! and special variables ({{now}}, {{user}}).
//! Also supports modifiers:
//! - {{var|sep:","}} - for arrays: replace default separator (space) with a custom one
//! - {{var|copy}} - for boolean values: copy the argument format (flag -> "--param", true -> "true", false -> "")

use super::ast::{Constant, Value, Variable};
use crate::constants::{
    DEFAULT_USER, ENV_VAR_USER, TEMPLATE_VAR_ERROR, TEMPLATE_VAR_NOW, TEMPLATE_VAR_USER,
};
use chrono::Utc;
use std::collections::HashMap;
use std::env;

/// Type alias for dynamic value evaluator function
pub type ValueEvaluator<'a> = dyn Fn(&str) -> Result<String, String> + 'a;

/// Context for template processing containing variables and constants.
#[derive(Default)]
pub struct TemplateContext<'a> {
    pub global_variables: &'a [Variable],
    pub global_constants: &'a [Constant],
    pub local_variables: &'a [Variable],
    pub local_constants: &'a [Constant],
    pub parent_variables: &'a [Variable],
    pub parent_constants: &'a [Constant],
}

/// Trait for resolving function calls in templates.
pub trait FunctionResolver {
    fn resolve(
        &self,
        func_name: &str,
        args: &HashMap<String, String>,
    ) -> Result<Option<String>, String>;
}

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
    /// Supported modifiers:
    /// - `{{var|sep:","}}` - For arrays: replace spaces with comma (e.g., "redis celery backend" -> "redis,celery,backend")
    /// - `{{var|rep:" "=>","}}` - Explicit replacement: replace space with comma
    /// - `{{var|copy}}` - For boolean values: copy the argument format (flag -> "--param", true -> "true", false -> "")
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
    /// * `context` - Template processing context (variables and constants)
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
    /// use nest_core::nestparse::template::{TemplateProcessor, TemplateContext};
    /// let mut args = HashMap::new();
    /// args.insert("name".to_string(), "world".to_string());
    /// let script = "echo Hello {{name}}!";
    /// let processed = TemplateProcessor::process(script, &args, &TemplateContext::default(), &std::collections::HashMap::new());
    /// assert_eq!(processed, "echo Hello world!");
    /// ```
    pub fn process(
        script: &str,
        args: &HashMap<String, String>,
        context: &TemplateContext,
        parent_args: &HashMap<String, String>,
        evaluator: Option<&ValueEvaluator>,
    ) -> String {
        let mut processed = script.to_string();

        // Build variable map with priority: local > parent > global > special
        let mut var_map: HashMap<String, String> = HashMap::new();

        // 1. Add global constants first (lowest priority for constants)
        for constant in context.global_constants {
            var_map.insert(
                constant.name.clone(),
                Self::resolve_value(&constant.value, evaluator),
            );
        }

        // 2. Add global variables (can override global constants)
        for variable in context.global_variables {
            var_map.insert(
                variable.name.clone(),
                Self::resolve_value(&variable.value, evaluator),
            );
        }

        // 3. Add parent constants (override global constants/variables)
        for constant in context.parent_constants {
            var_map.insert(
                constant.name.clone(),
                Self::resolve_value(&constant.value, evaluator),
            );
        }

        // 4. Add parent variables (override parent constants and global variables)
        for variable in context.parent_variables {
            var_map.insert(
                variable.name.clone(),
                Self::resolve_value(&variable.value, evaluator),
            );
        }

        // 5. Add local constants (override parent and global constants/variables)
        for constant in context.local_constants {
            var_map.insert(
                constant.name.clone(),
                Self::resolve_value(&constant.value, evaluator),
            );
        }

        // 6. Add local variables (highest priority for variables, override everything)
        for variable in context.local_variables {
            var_map.insert(
                variable.name.clone(),
                Self::resolve_value(&variable.value, evaluator),
            );
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
                                let modified_value = if modifier_part == "copy" {
                                    // copy modifier for boolean values: {{param|copy}}
                                    // If value is "true" (regardless of how passed), output "--param" (flag format)
                                    // If value is "false", output empty string
                                    // If value starts with "--", it's already in flag format -> output as is
                                    if value == "true" {
                                        // Convert parameter name to flag format (--param)
                                        format!("--{}", var_name)
                                    } else if value.starts_with("--") {
                                        // Already in flag format
                                        value.clone()
                                    } else if value == "false" {
                                        String::new()
                                    } else {
                                        value.clone()
                                    }
                                } else if let Some(sep_value) = modifier_part.strip_prefix("sep:") {
                                    // sep modifier: {{var|sep:","}}
                                    let sep = Self::parse_modifier_value(sep_value);
                                    value.replace(" ", &sep)
                                } else if let Some(rep_value) = modifier_part.strip_prefix("rep:") {
                                    // rep modifier: {{var|rep:"from"=>"to"}}
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
                                result.push_str(trimmed);
                                result.push_str("}}");
                            }
                        } else {
                            // Simple placeholder: {{var}}
                            if let Some(value) = combined_map.get(trimmed) {
                                // For boolean flags (values starting with "--"), convert to "true" for normal substitution
                                let output_value = if value.starts_with("--") {
                                    "true"
                                } else {
                                    value
                                };
                                result.push_str(output_value);
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
    pub fn parse_rep_modifier(modifier: &str) -> Option<(String, String)> {
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

    /// Processes function calls in templates like {{ func() }} or {{ func(arg="value") }}.
    ///
    /// # Arguments
    ///
    /// * `script` - The script containing template function calls
    /// * `resolver` - Function resolver implementation
    pub fn process_function_calls<R: FunctionResolver>(
        script: &str,
        resolver: &R,
    ) -> Result<String, String> {
        let mut result = String::with_capacity(script.len());
        let mut chars = script.chars().peekable();
        let mut buffer = String::new();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Check if it's {{ (double curly brace)
                if let Some(&'{') = chars.peek() {
                    chars.next(); // consume second '{'
                    buffer.clear();

                    // Collect content until }}
                    let mut found_close = false;
                    while let Some(ch) = chars.next() {
                        if ch == '}' {
                            if let Some(&'}') = chars.peek() {
                                chars.next(); // consume second '}'
                                found_close = true;
                                break;
                            } else {
                                buffer.push(ch);
                            }
                        } else {
                            buffer.push(ch);
                        }
                    }

                    if found_close {
                        let template_expr = buffer.trim();

                        // Check if it's a function call (contains parentheses)
                        if template_expr.contains('(') && template_expr.contains(')') {
                            // Try to parse as function call
                            if let Some((func_name, func_args)) =
                                super::executor::CommandExecutor::parse_command_call(template_expr)
                            {
                                // Resolve and execute function via resolver
                                let return_value = resolver.resolve(&func_name, &func_args)?;

                                // Replace template with return value
                                let replacement = return_value.unwrap_or_else(String::new);
                                result.push_str(&replacement);
                                continue;
                            }
                        }

                        // Not a function call or function not found - keep original template
                        result.push_str("{{");
                        result.push_str(template_expr);
                        result.push_str("}}");
                        continue;
                    }
                }
            }

            result.push(ch);
        }

        Ok(result)
    }

    /// Resolves a value, evaluating it if it's dynamic.
    fn resolve_value(value: &Value, evaluator: Option<&ValueEvaluator>) -> String {
        match value {
            Value::Dynamic(cmd) => evaluator
                .as_ref()
                .map(|e| e(cmd).unwrap_or_else(|err| format!("<error: {}>", err)))
                .unwrap_or_else(|| value.to_string_unquoted()),
            _ => value.to_string_unquoted(),
        }
    }
}
