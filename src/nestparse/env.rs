//! Environment variable management for command execution.
//!
//! This module handles extracting and loading environment variables from
//! directives, including support for .env files, direct assignments, and
//! system environment variables with fallback values.

use super::ast::Directive;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

/// Manages environment variables for command execution.
///
/// This is a utility struct with static methods for environment variable handling.
pub struct EnvironmentManager;

impl EnvironmentManager {
    /// Extracts environment variables from directives.
    ///
    /// This function processes `Directive::Env` directives and:
    /// - Loads variables from .env files (if the value starts with '.' and the file exists)
    /// - Parses direct assignments (if the value contains '=')
    /// - Resolves system environment variables with fallback (${VAR:-default})
    ///
    /// # Arguments
    ///
    /// * `directives` - List of directives to process
    ///
    /// # Returns
    ///
    /// Returns a HashMap of environment variable names to values.
    ///
    /// # Example
    ///
    /// Directives like:
    /// - `> env: .env.local` - Loads from file
    /// - `> env: NODE_ENV=production` - Direct assignment
    /// - `> env: NODE_ENV=${NODE_ENV:-development}` - System variable with fallback
    pub fn extract_env_vars(directives: &[Directive]) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        for directive in directives {
            if let Directive::Env(env_value) = directive {
                if Self::is_env_file(env_value) {
                    Self::load_from_file(env_value, &mut env_vars);
                } else if Self::is_direct_assignment(env_value) {
                    Self::parse_direct_assignment(env_value, &mut env_vars);
                }
            }
        }

        env_vars
    }

    fn is_env_file(value: &str) -> bool {
        value.starts_with('.') && Path::new(value).exists()
    }

    fn is_direct_assignment(value: &str) -> bool {
        value.contains('=')
    }

    fn load_from_file(file_path: &str, env_vars: &mut HashMap<String, String>) {
        if let Ok(content) = fs::read_to_string(file_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = Self::parse_env_line(line) {
                    env_vars.insert(key, value);
                }
            }
        }
    }

    fn parse_env_line(line: &str) -> Option<(String, String)> {
        line.find('=').map(|pos| {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            (key, value)
        })
    }

    fn parse_direct_assignment(env_value: &str, env_vars: &mut HashMap<String, String>) {
        if let Some((key, value)) = Self::parse_env_line(env_value) {
            // Resolve system environment variables with fallback syntax: ${VAR:-default}
            let resolved_value = Self::resolve_env_value(&value);
            env_vars.insert(key, resolved_value);
        }
    }

    /// Resolves environment variable references in a value string.
    ///
    /// Supports syntax:
    /// - `${VAR:-default}` - Use system variable VAR if exists, otherwise use default
    /// - `${VAR}` - Use system variable VAR if exists, otherwise empty string
    ///
    /// # Arguments
    ///
    /// * `value` - The value string that may contain environment variable references
    ///
    /// # Returns
    ///
    /// Returns the resolved value with all environment variable references expanded.
    ///
    /// # Example
    ///
    /// ```
    /// // If NODE_ENV is set to "production" in system:
    /// resolve_env_value("${NODE_ENV:-development}") -> "production"
    ///
    /// // If NODE_ENV is not set:
    /// resolve_env_value("${NODE_ENV:-development}") -> "development"
    ///
    /// // Simple variable without fallback:
    /// resolve_env_value("${HOME}") -> "/home/user" (if set) or "" (if not set)
    /// ```
    fn resolve_env_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                // Found ${ - start of environment variable reference
                chars.next(); // Skip '{'
                
                let mut var_name = String::new();
                let mut fallback = None;
                let mut in_fallback = false;
                
                while let Some(ch) = chars.next() {
                    match ch {
                        '}' if !in_fallback => {
                            // End of variable reference
                            break;
                        }
                        ':' if !in_fallback && chars.peek() == Some(&'-') => {
                            // Found :- start of fallback
                            chars.next(); // Skip '-'
                            in_fallback = true;
                        }
                        '}' if in_fallback => {
                            // End of fallback
                            break;
                        }
                        _ if in_fallback => {
                            fallback.get_or_insert_with(String::new).push(ch);
                        }
                        _ => {
                            var_name.push(ch);
                        }
                    }
                }
                
                // Resolve the variable
                let resolved = if !var_name.is_empty() {
                    env::var(&var_name).unwrap_or_else(|_| {
                        fallback.unwrap_or_default()
                    })
                } else {
                    // Invalid syntax, return as-is
                    format!("${{{}}}", var_name)
                };
                
                result.push_str(&resolved);
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}

