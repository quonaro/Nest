//! Template processing for variable substitution in scripts.
//!
//! This module handles replacing placeholders in scripts with actual values.
//! Supports parameter placeholders ({{param}}), variables ({{VAR}}), constants ({{CONST}}),
//! and special variables ({{now}}, {{user}}).

use crate::constants::{DEFAULT_USER, ENV_VAR_USER, TEMPLATE_VAR_NOW, TEMPLATE_VAR_USER};
use super::ast::{Variable, Constant};
use std::collections::HashMap;
use std::env;
use chrono::Utc;

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
    /// - `{{now}}` - Replaced with current UTC time in RFC3339 format
    /// - `{{user}}` - Replaced with the USER environment variable (or "unknown" if not set)
    ///
    /// Priority order:
    /// 1. Parameters (from args) - highest priority
    /// 2. Local variables (from command) - override global variables
    /// 3. Local constants (from command) - override global constants
    /// 4. Global variables (can be redefined, last definition wins)
    /// 5. Global constants (cannot be redefined)
    /// 6. Special variables ({{now}}, {{user}}) - lowest priority
    ///
    /// # Arguments
    ///
    /// * `script` - The script template containing placeholders
    /// * `args` - HashMap of parameter names to values for replacement
    /// * `global_variables` - List of global variables
    /// * `global_constants` - List of global constants
    /// * `local_variables` - List of local variables (from command, optional)
    /// * `local_constants` - List of local constants (from command, optional)
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
    /// let processed = TemplateProcessor::process(script, &args, &[], &[], &[], &[]);
    /// assert_eq!(processed, "echo Hello world!");
    /// ```
    pub fn process(
        script: &str,
        args: &HashMap<String, String>,
        global_variables: &[Variable],
        global_constants: &[Constant],
        local_variables: &[Variable],
        local_constants: &[Constant],
    ) -> String {
        let mut processed = script.to_string();

        // Build variable map with priority: local > global
        let mut var_map: HashMap<String, String> = HashMap::new();
        
        // 1. Add global constants first (lowest priority for constants)
        for constant in global_constants {
            var_map.insert(constant.name.clone(), constant.value.clone());
        }
        
        // 2. Add global variables (can override global constants)
        for variable in global_variables {
            var_map.insert(variable.name.clone(), variable.value.clone());
        }
        
        // 3. Add local constants (override global constants/variables)
        for constant in local_constants {
            var_map.insert(constant.name.clone(), constant.value.clone());
        }
        
        // 4. Add local variables (highest priority for variables, override everything)
        for variable in local_variables {
            var_map.insert(variable.name.clone(), variable.value.clone());
        }

        // Replace parameter placeholders {{param}} (highest priority)
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            processed = processed.replace(&placeholder, value);
        }

        // Replace variable and constant placeholders {{VAR}} or {{CONST}}
        for (key, value) in &var_map {
            let placeholder = format!("{{{{{}}}}}", key);
            processed = processed.replace(&placeholder, value);
        }

        // Replace special variables (lowest priority, but checked after to avoid conflicts)
        processed = processed.replace(TEMPLATE_VAR_NOW, &Utc::now().to_rfc3339());
        processed = processed.replace(
            TEMPLATE_VAR_USER,
            &env::var(ENV_VAR_USER).unwrap_or_else(|_| DEFAULT_USER.to_string()),
        );

        processed
    }
}

