//! Environment variable management for command execution.
//!
//! This module handles extracting and loading environment variables from
//! directives, including support for .env files, direct assignments, and
//! system environment variables with fallback values.

use super::ast::Directive;
use std::collections::HashMap;
use std::env;
use std::fs;

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
    /// - Resolves environment variables with fallback (${VAR:-default} or $VAR)
    ///   Variables are resolved first from already loaded env_vars, then from system environment
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
    /// - `> env: NODE_ENV=${NODE_ENV:-development}` - Variable with fallback (checks env_vars first, then system)
    /// - `> env: PGPASSWORD=$DB_PASSWORD` - Use variable from already loaded .env file
    pub fn extract_env_vars(directives: &[Directive]) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        for directive in directives {
            match directive {
                Directive::Env(name, value, _) => {
                    env_vars.insert(name.clone(), value.clone());
                }
                Directive::EnvFile(path, _) => {
                    Self::load_from_file(path, &mut env_vars);
                }
                _ => {}
            }
        }

        env_vars
    }

    /// Exports all Nest variables and constants to the environment HashMap.
    pub fn export_all_vars(
        env_vars: &mut HashMap<String, String>,
        variables: &[super::ast::Variable],
        constants: &[super::ast::Constant],
    ) {
        for var in variables {
            env_vars.insert(var.name.clone(), var.value.to_string_unquoted());
        }
        for constant in constants {
            env_vars.insert(constant.name.clone(), constant.value.to_string_unquoted());
        }
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

    /// Resolves environment variable references in a value string.
    ///
    /// Supports syntax:
    /// - `${VAR:-default}` - Use variable VAR from env_vars or system, otherwise use default
    /// - `${VAR}` - Use variable VAR from env_vars or system, otherwise empty string
    /// - `$VAR` - Use variable VAR from env_vars or system, otherwise empty string
    ///
    /// # Arguments
    ///
    /// * `value` - The value string that may contain environment variable references
    /// * `env_vars` - Already loaded environment variables (from .env files, etc.)
    ///
    /// # Returns
    ///
    /// Returns the resolved value with all environment variable references expanded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use nest_core::nestparse::env::EnvironmentManager;
    ///
    /// let mut env_vars = HashMap::new();
    /// env_vars.insert("NODE_ENV".to_string(), "production".to_string());
    ///
    /// // If NODE_ENV is set to "production" in env_vars or system:
    /// let res1 = EnvironmentManager::resolve_env_value("${NODE_ENV:-development}", &env_vars);
    /// assert_eq!(res1, "production");
    ///
    /// // If NODE_ENV is not set:
    /// let res2 = EnvironmentManager::resolve_env_value("${OTHER_VAR:-defaultValue}", &env_vars);
    /// assert_eq!(res2, "defaultValue");
    /// ```
    /// Resolves environment variable references in a value string.
    ///
    /// This function is intentionally public so it can be reused by other
    /// modules (e.g. condition evaluation) to provide a consistent behaviour
    /// for `$VAR` / `${VAR:-default}` style expansions.
    pub fn resolve_env_value(value: &str, env_vars: &HashMap<String, String>) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                if chars.peek() == Some(&'{') {
                    // Found ${ - start of environment variable reference with braces
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

                    // Resolve the variable: first check env_vars, then system environment
                    let resolved = if !var_name.is_empty() {
                        env_vars
                            .get(&var_name)
                            .cloned()
                            .or_else(|| env::var(&var_name).ok())
                            .unwrap_or_else(|| fallback.unwrap_or_default())
                    } else {
                        // Invalid syntax, return as-is
                        format!("${{{}}}", var_name)
                    };

                    result.push_str(&resolved);
                } else {
                    // Found $ - start of environment variable reference without braces
                    // Extract variable name (alphanumeric and underscore only)
                    let mut var_name = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            var_name.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    // Resolve the variable: first check env_vars, then system environment
                    if !var_name.is_empty() {
                        let resolved = env_vars
                            .get(&var_name)
                            .cloned()
                            .or_else(|| env::var(&var_name).ok())
                            .unwrap_or_default();
                        result.push_str(&resolved);
                    } else {
                        // Just a $ without variable name, keep it as-is
                        result.push(ch);
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}
