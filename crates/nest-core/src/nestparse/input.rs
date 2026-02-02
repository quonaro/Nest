//! User input handling.
//!
//! This module handles interactive user prompts and confirmations.

use std::io::{self, Write};

/// Prompts user for confirmation before executing a command.
///
/// # Arguments
///
/// * `message` - Optional custom confirmation message. If None or empty, uses default message.
/// * `command_path` - Path to the command being executed (for default message)
///
/// # Returns
///
/// Returns `Ok(true)` if user confirmed (y), `Ok(false)` if user declined (n),
/// or `Err(message)` if there was an error reading input.
pub fn prompt_confirmation(
    message: Option<&str>,
    command_path: Option<&[String]>,
) -> Result<bool, String> {
    let prompt_text = if let Some(msg) = message {
        if msg.trim().is_empty() {
            // Empty message - use default
            let command_name = if let Some(path) = command_path {
                path.join(" ")
            } else {
                "this command".to_string()
            };
            format!(
                "Are you sure you want to execute '{}'? [y/n]: ",
                command_name
            )
        } else {
            // Custom message
            format!("{} [y/n]: ", msg.trim())
        }
    } else {
        // No message - use default
        let command_name = if let Some(path) = command_path {
            path.join(" ")
        } else {
            "this command".to_string()
        };
        format!(
            "Are you sure you want to execute '{}'? [y/n]: ",
            command_name
        )
    };

    // Print prompt and flush to ensure it's displayed
    print!("{}", prompt_text);
    io::stdout()
        .flush()
        .map_err(|e| format!("Failed to flush stdout: {}", e))?;

    // Read user input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {}", e))?;

    // Parse input (trim whitespace and convert to lowercase)
    let trimmed = input.trim().to_lowercase();

    match trimmed.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            // Invalid input - ask again
            println!("Please enter 'y' for yes or 'n' for no.");
            prompt_confirmation(message, command_path)
        }
    }
}
