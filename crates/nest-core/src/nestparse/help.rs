//! Help message formatting for command groups.
//!
//! This module provides utilities for displaying help messages when
//! a group command is called without a subcommand and has no default.

use super::ast::{Command, Directive};
use super::output::OutputFormatter;

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
        println!(
            "{} nest {} [COMMAND]",
            OutputFormatter::help_label("Usage:"),
            OutputFormatter::help_command(&command_path.join(" "))
        );
        println!();

        if let Some(desc) = Self::extract_description(&command.directives) {
            println!("{}", OutputFormatter::help_description(desc));
            println!();
        }

        println!("{}", OutputFormatter::help_label("Available commands:"));
        for child in &command.children {
            let child_desc = Self::extract_description(&child.directives);
            let command_with_params = child.to_string();
            if let Some(desc) = child_desc {
                println!(
                    "  {}  {}",
                    OutputFormatter::help_command(&command_with_params),
                    OutputFormatter::help_description(desc)
                );
            } else {
                println!("  {}", OutputFormatter::help_command(&command_with_params));
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
