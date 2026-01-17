//! Display utilities for printing command structures.
//!
//! This module provides functions for displaying commands in a human-readable
//! tree format, used by `nest --show ast`.

use super::ast::{Command, Directive};

/// Prints a command and its children in a tree format.
///
/// This function recursively prints the command structure with indentation,
/// showing directives and child commands in a visual tree.
///
/// # Arguments
///
/// * `command` - The command to print
/// * `indent` - The indentation level (number of spaces)
pub fn print_command(command: &Command, indent: usize) {
    let indent_str = "  ".repeat(indent);
    println!("{}└─ {}", indent_str, command);

    // Print directives
    for directive in &command.directives {
        match directive {
            Directive::Desc(s) => {
                println!("{}    > desc: {}", indent_str, s);
            }
            Directive::Cwd(s) => {
                println!("{}    > cwd: {}", indent_str, s);
            }
            Directive::Env(s) => {
                println!("{}    > env: {}", indent_str, s);
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
                let suffix = if *parallel { " [parallel]" } else { "" };
                println!("{}    > depends{}: {}", indent_str, suffix, deps_str.join(", "));
            }
            Directive::Before(s) | Directive::BeforeHide(s) => {
                let directive_name = if matches!(directive, Directive::BeforeHide(_)) {
                    "before[hide]"
                } else {
                    "before"
                };
                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, directive_name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, directive_name, s);
                }
            }
            Directive::After(s) | Directive::AfterHide(s) => {
                let directive_name = if matches!(directive, Directive::AfterHide(_)) {
                    "after[hide]"
                } else {
                    "after"
                };
                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, directive_name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, directive_name, s);
                }
            }
            Directive::Fallback(s) | Directive::FallbackHide(s) => {
                let directive_name = if matches!(directive, Directive::FallbackHide(_)) {
                    "fallback[hide]"
                } else {
                    "fallback"
                };
                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, directive_name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, directive_name, s);
                }
            }
            Directive::Finaly(s) | Directive::FinalyHide(s) => {
                let directive_name = if matches!(directive, Directive::FinalyHide(_)) {
                    "finaly[hide]"
                } else {
                    "finaly"
                };
                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, directive_name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, directive_name, s);
                }
            }
            Directive::Validate(s) => {
                println!("{}    > validate: {}", indent_str, s);
            }
            Directive::Privileged(value) => {
                println!("{}    > privileged: {}", indent_str, value);
            }
            Directive::Script(s) | Directive::ScriptHide(s) => {
                let directive_name = if matches!(directive, Directive::ScriptHide(_)) {
                    "script[hide]"
                } else {
                    "script"
                };
                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, directive_name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, directive_name, s);
                }
            }
            Directive::Logs(path, format) => {
                println!("{}    > logs:{} {}", indent_str, format, path);
            }

            Directive::RequireConfirm(message) => {
                if message.trim().is_empty() {
                    println!("{}    > require_confirm:", indent_str);
                } else {
                    println!("{}    > require_confirm: {}", indent_str, message);
                }
            }
            Directive::Watch(inputs) => {
                let formatted: Vec<String> = inputs.iter().map(|s| format!("\"{}\"", s)).collect();
                println!("{}    > watch: {}", indent_str, formatted.join(", "));
            }
        }
    }

    // Print children
    for child in &command.children {
        print_command(child, indent + 1);
    }
}

