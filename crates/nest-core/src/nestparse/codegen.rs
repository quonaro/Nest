//! Codegen module for converting AST back to Nestfile string.
//!
//! This module is responsible for reconstructing valid Nestfile syntax from
//! parsed Command structures. This is primarily used by the filtered include system.

use super::ast::{Command, Directive, ParamKind, Value};
use crate::constants::INDENT_SIZE;

/// Converts a Command AST back to a Nestfile string representation.
///
/// # Arguments
///
/// * `command` - The command to convert
/// * `indent` - The current indentation level (number of spaces)
///
/// # Returns
///
/// A string containing the formatted command definition.
pub fn to_nestfile_string(command: &Command, indent: usize) -> String {
    let mut result = String::new();
    let indent_str = " ".repeat(indent);

    // 1. Print variables
    for var in &command.local_variables {
        result.push_str(&format!("{}@var {} = {}\n", indent_str, var.name, var.value));
    }

    // 2. Print constants
    for constant in &command.local_constants {
        result.push_str(&format!("{}@const {} = {}\n", indent_str, constant.name, constant.value));
    }

    // 3. Command signature
    result.push_str(&indent_str);
    result.push_str(&command.name);

    if !command.parameters.is_empty() {
        result.push('(');
        let params: Vec<String> = command.parameters.iter().map(|p| {
            match &p.kind {
                ParamKind::Normal => {
                    let mut s = if p.is_named { "!".to_string() } else { String::new() };
                    s.push_str(&p.name);
                    if let Some(alias) = &p.alias {
                        s.push_str(&format!("|{}", alias));
                    }
                    s.push_str(&format!(": {}", p.param_type));
                    if let Some(default) = &p.default {
                        s.push_str(" = ");
                        s.push_str(&value_to_string(default));
                    }
                    s
                }
                ParamKind::Wildcard { name, count } => {
                    let mut s = String::from("*");
                    if let Some(name) = name {
                        s.push_str(name);
                    }
                    if let Some(count) = count {
                        s.push_str(&format!("[{}]", count));
                    }
                    s
                }
            }
        }).collect();
        result.push_str(&params.join(", "));
        result.push(')');
    }

    result.push_str(":\n");

    // 4. Print directives
    // We increase indent for directives
    let inner_indent = indent + (INDENT_SIZE as usize);
    let inner_indent_str = " ".repeat(inner_indent);

    for directive in &command.directives {
        match directive {
            Directive::Desc(s) => {
                result.push_str(&format!("{}> desc: {}\n", inner_indent_str, s));
            }
            Directive::Cwd(s) => {
                result.push_str(&format!("{}> cwd: {}\n", inner_indent_str, s));
            }
            Directive::Env(s) => {
                result.push_str(&format!("{}> env: {}\n", inner_indent_str, s));
            }
            Directive::Depends(deps, parallel) => {
                let deps_str: Vec<String> = deps.iter().map(|dep| {
                    if dep.args.is_empty() {
                        dep.command_path.clone()
                    } else {
                        let args_str: Vec<String> = dep.args.iter()
                            .map(|(k, v)| format!("{}=\"{}\"", k, v))
                            .collect();
                        format!("{}({})", dep.command_path, args_str.join(", "))
                    }
                }).collect();
                let suffix = if *parallel { "[parallel]" } else { "" };
                result.push_str(&format!("{}> depends{}: {}\n", inner_indent_str, suffix, deps_str.join(", ")));
            }
            Directive::Privileged(val) => {
                result.push_str(&format!("{}> privileged: {}\n", inner_indent_str, val));
            }
            Directive::Logs(path, fmt) => {
                result.push_str(&format!("{}> logs:{} {}\n", inner_indent_str, fmt, path));
            }
            Directive::RequireConfirm(msg) => {
                if msg.is_empty() {
                    result.push_str(&format!("{}> require_confirm:\n", inner_indent_str));
                } else {
                    result.push_str(&format!("{}> require_confirm: {}\n", inner_indent_str, msg));
                }
            }
            Directive::If(cond) => {
                result.push_str(&format!("{}> if: {}\n", inner_indent_str, cond));
            }
            Directive::Elif(cond) => {
                result.push_str(&format!("{}> elif: {}\n", inner_indent_str, cond));
            }
            Directive::Else => {
                result.push_str(&format!("{}> else\n", inner_indent_str));
            }
            Directive::Validate(rule) => {
                result.push_str(&format!("{}> validate: {}\n", inner_indent_str, rule));
            }
            // Script directives
            Directive::Script(s) => format_script_directive(&mut result, &inner_indent_str, "script", s),
            Directive::ScriptHide(s) => format_script_directive(&mut result, &inner_indent_str, "script[hide]", s),
            Directive::Before(s) => format_script_directive(&mut result, &inner_indent_str, "before", s),
            Directive::BeforeHide(s) => format_script_directive(&mut result, &inner_indent_str, "before[hide]", s),
            Directive::After(s) => format_script_directive(&mut result, &inner_indent_str, "after", s),
            Directive::AfterHide(s) => format_script_directive(&mut result, &inner_indent_str, "after[hide]", s),
            Directive::Fallback(s) => format_script_directive(&mut result, &inner_indent_str, "fallback", s),
            Directive::FallbackHide(s) => format_script_directive(&mut result, &inner_indent_str, "fallback[hide]", s),
            Directive::Finaly(s) => format_script_directive(&mut result, &inner_indent_str, "finaly", s),
            Directive::FinalyHide(s) => format_script_directive(&mut result, &inner_indent_str, "finaly[hide]", s),
            Directive::Watch(inputs) => {
                let formatted: Vec<String> = inputs.iter().map(|s| format!("\"{}\"", s)).collect();
                result.push_str(&format!("{}> watch: {}\n", inner_indent_str, formatted.join(", ")));
            }
        }
    }

    // 5. Print children
    for child in &command.children {
        result.push_str(&to_nestfile_string(child, inner_indent));
    }

    result
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s), // Always quote strings for safety
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|s| format!("\"{}\"", s)).collect();
            format!("[{}]", items.join(", "))
        }
    }
}

fn format_script_directive(result: &mut String, indent_str: &str, name: &str, content: &str) {
    if content.contains('\n') {
        result.push_str(&format!("{}> {}: |\n", indent_str, name));
        for line in content.lines() {
            result.push_str(&format!("{}    {}\n", indent_str, line));
        }
    } else {
        result.push_str(&format!("{}> {}: {}\n", indent_str, name, content));
    }
}
