//! Template processing for variable substitution in scripts.
//!
//! This module handles replacing placeholders in scripts with actual values.
//! Supports parameter placeholders ({{param}}) and special variables ({{now}}, {{user}}).

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
    /// - `{{now}}` - Replaced with current UTC time in RFC3339 format
    /// - `{{user}}` - Replaced with the USER environment variable (or "unknown" if not set)
    ///
    /// # Arguments
    ///
    /// * `script` - The script template containing placeholders
    /// * `args` - HashMap of parameter names to values for replacement
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
    /// let processed = TemplateProcessor::process(script, &args);
    /// assert_eq!(processed, "echo Hello world!");
    /// ```
    pub fn process(script: &str, args: &HashMap<String, String>) -> String {
        let mut processed = script.to_string();

        // Replace parameter placeholders {{param}}
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            processed = processed.replace(&placeholder, value);
        }

        // Replace special variables
        processed = processed.replace("{{now}}", &Utc::now().to_rfc3339());
        processed = processed.replace(
            "{{user}}",
            &env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
        );

        processed
    }
}

