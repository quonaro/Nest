//! Directive resolution and handling logic.
//!
//! This module handles the resolution of directives from the AST,
//! including OS compatibility checks and value extraction.

use super::ast::Directive;

/// Resolver for directives and their values.
pub struct DirectiveResolver;

impl DirectiveResolver {
    /// Checks if the directive's OS requirement matches the current system.
    pub fn check_os_match(os: &Option<String>) -> bool {
        match os {
            Some(required_os) => {
                let current_os = std::env::consts::OS;
                if required_os.eq_ignore_ascii_case(current_os) {
                    return true;
                }
                if required_os.eq_ignore_ascii_case("unix") && cfg!(unix) {
                    return true;
                }
                if required_os.eq_ignore_ascii_case("bsd") && current_os.contains("bsd") {
                    return true;
                }
                false
            }
            None => true,
        }
    }

    /// Gets directive value for a given name, checking OS compatibility.
    pub fn get_directive_value(directives: &[Directive], name: &str) -> Option<String> {
        let mut best_match: Option<String> = None;
        let mut best_score = 0;

        for d in directives {
            let (val, os, target_name) = match d {
                Directive::Desc(s) => (Some(s.clone()), &None, "desc"),
                Directive::Cwd(s) => (Some(s.clone()), &None, "cwd"),
                Directive::Env(k, v, _) => (Some(format!("{}={}", k, v)), &None, "env"),
                Directive::EnvFile(s, _) => (Some(s.clone()), &None, "env"),

                Directive::Script(s, os, _) => (Some(s.clone()), os, "script"),
                Directive::Before(s, os, _) => (Some(s.clone()), os, "before"),
                Directive::After(s, os, _) => (Some(s.clone()), os, "after"),
                Directive::Fallback(s, os, _) => (Some(s.clone()), os, "fallback"),
                Directive::Finally(s, os, _) => (Some(s.clone()), os, "finally"),
                Directive::Validate(_, _) => (None, &None, "validate"), // validation handled separately
                _ => (None, &None, ""),
            };

            if target_name == name && Self::check_os_match(os) {
                let score = if os.is_some() { 2 } else { 1 };
                if score > best_score {
                    best_score = score;
                    best_match = val;
                }
            }
        }
        best_match
    }

    /// Gets directive value and checks if output should be hidden.
    /// Returns (value, hide_output) tuple.
    pub fn get_directive_value_with_hide(
        directives: &[Directive],
        name: &str,
    ) -> Option<(String, bool)> {
        let mut best_match: Option<(String, bool)> = None;
        let mut best_score = 0;

        for d in directives {
            let (val, os, hide, target_name) = match d {
                Directive::Env(k, v, hide) => (Some(format!("{}={}", k, v)), &None, *hide, "env"),
                Directive::EnvFile(s, hide) => (Some(s.clone()), &None, *hide, "env"),

                Directive::Script(s, os, hide) => (Some(s.clone()), os, *hide, "script"),
                Directive::Before(s, os, hide) => (Some(s.clone()), os, *hide, "before"),
                Directive::After(s, os, hide) => (Some(s.clone()), os, *hide, "after"),
                Directive::Fallback(s, os, hide) => (Some(s.clone()), os, *hide, "fallback"),
                Directive::Finally(s, os, hide) => (Some(s.clone()), os, *hide, "finally"),
                _ => (None, &None, false, ""),
            };

            if target_name == name && Self::check_os_match(os) {
                let score = if os.is_some() { 2 } else { 1 };
                if score > best_score {
                    best_score = score;
                    // Unwrap is safe because we checked val is Some in match arms if target_name matches
                    if let Some(v) = val {
                        best_match = Some((v, hide));
                    }
                }
            }
        }
        best_match
    }

    pub fn get_depends_directive(directives: &[Directive]) -> (Vec<super::ast::Dependency>, bool) {
        directives
            .iter()
            .find_map(|d| match d {
                Directive::Depends(deps, parallel) => Some((deps.clone(), *parallel)),
                _ => None,
            })
            .unwrap_or((Vec::new(), false))
    }

    pub fn get_privileged_directive(directives: &[Directive]) -> bool {
        directives
            .iter()
            .find_map(|d| match d {
                Directive::Privileged(value) => Some(*value),
                _ => None,
            })
            .unwrap_or(false)
    }

    pub fn get_require_confirm_directive(directives: &[Directive]) -> Option<String> {
        directives.iter().find_map(|d| match d {
            Directive::RequireConfirm(message) => Some(message.clone()),
            _ => None,
        })
    }

    pub fn get_logs_directive(directives: &[Directive]) -> Option<(String, String)> {
        directives.iter().find_map(|d| match d {
            Directive::Logs(path, format) => Some((path.clone(), format.clone())),
            _ => None,
        })
    }

    pub fn get_validate_directives(directives: &[Directive]) -> Vec<(String, String)> {
        directives
            .iter()
            .filter_map(|d| match d {
                Directive::Validate(target, rule) => Some((target.clone(), rule.clone())),
                _ => None,
            })
            .collect()
    }
}
