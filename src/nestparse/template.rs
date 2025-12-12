//! Template processing for variable substitution in scripts.
//!
//! This module handles replacing placeholders in scripts with actual values.
//! Supports parameter placeholders ({{param}}), variables ({{VAR}}), constants ({{CONST}}),
//! and special variables ({{now}}, {{user}}).

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

        // Replace parameter placeholders {{param}} (highest priority)
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            processed = processed.replace(&placeholder, value);
        }

        // Replace parent parameter placeholders {{param}} (second highest priority, only if not in args)
        for (key, value) in parent_args {
            if !args.contains_key(key) {
                let placeholder = format!("{{{{{}}}}}", key);
                processed = processed.replace(&placeholder, value);
            }
        }

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

        // Replace variable and constant placeholders {{VAR}} or {{CONST}}
        for (key, value) in &var_map {
            let placeholder = format!("{{{{{}}}}}", key);
            processed = processed.replace(&placeholder, value);
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
}
