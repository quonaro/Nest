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
            Directive::Env(k, v, hide) => {
                let suffix = if *hide { ".hide" } else { "" };
                println!("{}    > env{}: {}={}", indent_str, suffix, k, v);
            }
            Directive::EnvFile(s, hide) => {
                let suffix = if *hide { ".hide" } else { "" };
                println!("{}    > env{}: {}", indent_str, suffix, s);
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
                let suffix = if *parallel { ".parallel" } else { "" };
                println!("{}    > depends{}: {}", indent_str, suffix, deps_str.join(", "));
            }
            Directive::Before(s, os, hide) => {
                let mut name = String::from("before");
                if let Some(os_name) = os { name.push('.'); name.push_str(os_name); }
                if *hide { name.push_str(".hide"); }
                
                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, name, s);
                }
            }
            Directive::After(s, os, hide) => {
                let mut name = String::from("after");
                if let Some(os_name) = os { name.push('.'); name.push_str(os_name); }
                if *hide { name.push_str(".hide"); }

                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, name, s);
                }
            }
            Directive::Fallback(s, os, hide) => {
                let mut name = String::from("fallback");
                if let Some(os_name) = os { name.push('.'); name.push_str(os_name); }
                if *hide { name.push_str(".hide"); }

                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, name, s);
                }
            }
            Directive::Finally(s, os, hide) => {
                let mut name = String::from("finally");
                if let Some(os_name) = os { name.push('.'); name.push_str(os_name); }
                if *hide { name.push_str(".hide"); }

                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, name, s);
                }
            }
            Directive::Validate(target, rule) => {
                println!("{}    > validate.{}: {}", indent_str, target, rule);
            }
            Directive::Privileged(value) => {
                println!("{}    > privileged: {}", indent_str, value);
            }
            Directive::Script(s, os, hide) => {
                let mut name = String::from("script");
                if let Some(os_name) = os { name.push('.'); name.push_str(os_name); }
                if *hide { name.push_str(".hide"); }

                if s.contains('\n') {
                    println!("{}    > {}: |", indent_str, name);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > {}: {}", indent_str, name, s);
                }
            }
            Directive::Logs(path, format) => {
                println!("{}    > logs.{}: {}", indent_str, format, path);
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

