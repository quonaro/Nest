//! Unified output formatting for all messages.
//!
//! This module provides a consistent, colorful style for:
//! - Error messages
//! - Help messages
//! - System/info messages
//! - Success messages

use std::fmt::Write;

/// ANSI color codes
#[allow(dead_code)]
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const GRAY: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const WHITE: &str = "\x1b[97m";
}

/// Unified output formatter for consistent styling
pub struct OutputFormatter;

#[allow(dead_code)]
impl OutputFormatter {
    /// Prints an error message with consistent formatting
    pub fn error(message: &str) {
        Self::error_box("Error", message);
    }

    /// Prints an error message with a title
    pub fn error_with_title(title: &str, message: &str) {
        Self::error_box(title, message);
    }

    /// Prints a success message
    pub fn success(message: &str) {
        eprintln!("{}✓{} {}", colors::BRIGHT_GREEN, colors::RESET, message);
    }

    /// Prints an info message
    pub fn info(message: &str) {
        eprintln!("{}ℹ{} {}", colors::BRIGHT_CYAN, colors::RESET, message);
    }

    /// Prints a warning message
    pub fn warning(message: &str) {
        eprintln!("{}⚠{} {}", colors::BRIGHT_YELLOW, colors::RESET, message);
    }

    /// Prints a formatted error box (like execution errors)
    pub fn error_box(title: &str, content: &str) {
        let mut output = String::new();
        writeln!(
            output,
            "\n{}╔═══════════════════════════════════════════════════════════════╗{}",
            colors::RED,
            colors::RESET
        )
        .expect("Failed to format error box header");
        writeln!(
            output,
            "{}║{}  {}{}{}",
            colors::RED,
            colors::RESET,
            colors::BRIGHT_RED,
            title,
            colors::RESET
        )
        .expect("Failed to format error box title");
        writeln!(
            output,
            "{}╚═══════════════════════════════════════════════════════════════╝{}",
            colors::RED,
            colors::RESET
        )
        .expect("Failed to format error box footer");
        writeln!(output, "\n{}", content).expect("Failed to format error box content");
        eprint!("{}", output);
    }

    /// Formats a help section header
    pub fn help_header(text: &str) -> String {
        format!("{}{}{}", colors::BRIGHT_CYAN, text, colors::RESET)
    }

    /// Formats a help command name
    pub fn help_command(name: &str) -> String {
        format!("{}{}{}", colors::BRIGHT_BLUE, name, colors::RESET)
    }

    /// Formats a help description
    pub fn help_description(desc: &str) -> String {
        format!("{}{}{}", colors::GRAY, desc, colors::RESET)
    }

    /// Formats a help label (like "Usage:", "Available commands:")
    pub fn help_label(label: &str) -> String {
        format!("{}{}{}", colors::BRIGHT_CYAN, label, colors::RESET)
    }

    /// Formats a value in help/output
    pub fn value(text: &str) -> String {
        format!("{}{}{}", colors::BRIGHT_GREEN, text, colors::RESET)
    }

    /// Formats a parameter name
    pub fn parameter(name: &str) -> String {
        format!("{}{}{}", colors::YELLOW, name, colors::RESET)
    }

    /// Formats a parameter value
    pub fn parameter_value(value: &str) -> String {
        format!("{}{}{}", colors::CYAN, value, colors::RESET)
    }

    /// Formats a file path
    pub fn path(path: &str) -> String {
        format!("{}{}{}", colors::BRIGHT_CYAN, path, colors::RESET)
    }

    /// Prints a box with content (for detailed error messages)
    pub fn print_box(content: &str, color: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0).min(60);

        eprintln!("{}┌{}┐{}", color, "─".repeat(max_width), colors::RESET);
        for line in &lines {
            eprintln!(
                "{}│{} {}{}│{}",
                color,
                colors::RESET,
                line,
                " ".repeat(max_width.saturating_sub(line.len())),
                color
            );
        }
        eprintln!("{}└{}┘{}", color, "─".repeat(max_width), colors::RESET);
    }

    /// Prints a section divider
    pub fn divider() {
        eprintln!("{}{}{}", colors::GRAY, "─".repeat(60), colors::RESET);
    }
}
