use super::ast::Directive;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct EnvironmentManager;

impl EnvironmentManager {
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
            env_vars.insert(key, value);
        }
    }
}

