//! Help message formatting for command groups.
//!
//! This module provides utilities for displaying help messages when
//! a group command is called without a subcommand and has no default.

use super::ast::{Command, Directive};

/// Formats and prints help messages for command groups.
///
/// This is a utility struct with static methods for help formatting.
pub struct HelpFormatter;

impl HelpFormatter {
    /// Prints a help message for a group command.
    ///
    /// The help message includes:
    /// - Usage information
    /// - Command description (if available)
    /// - List of available subcommands with their descriptions
    ///
    /// # Arguments
    ///
    /// * `command` - The group command to show help for
    /// * `command_path` - The full path to the command (e.g., ["dev"])
    pub fn print_group_help(command: &Command, command_path: &[String]) {
        println!("Usage: nest {} [COMMAND]", command_path.join(" "));
        println!();

        if let Some(desc) = Self::extract_description(&command.directives) {
            println!("{}", desc);
            println!();
        }

        println!("Available commands:");
        for child in &command.children {
            let child_desc = Self::extract_description(&child.directives);
            if let Some(desc) = child_desc {
                println!("  {}  {}", child.name, desc);
            } else {
                println!("  {}", child.name);
            }
        }
    }

    fn extract_description(directives: &[Directive]) -> Option<&str> {
        directives.iter().find_map(|d| {
            if let Directive::Desc(s) = d {
                Some(s.as_str())
            } else {
                None
            }
        })
    }
}

