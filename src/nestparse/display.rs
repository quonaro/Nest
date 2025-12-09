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
            Directive::Privileged(value) => {
                println!("{}    > privileged: {}", indent_str, value);
            }
            Directive::Script(s) => {
                if s.contains('\n') {
                    println!("{}    > script: |", indent_str);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > script: {}", indent_str, s);
                }
            }
        }
    }

    // Print children
    for child in &command.children {
        print_command(child, indent + 1);
    }
}

