//! Script execution and error reporting.
//!
//! This module handles the actual execution of shell scripts defined in commands,
//! including environment variable setup, working directory configuration,
//! and detailed error reporting with beautiful formatting.

use super::ast::Command;
use std::collections::HashMap;
use std::process::{Command as ProcessCommand, Stdio};

/// Executes shell scripts for commands.
///
/// This is a utility struct with static methods for script execution.
pub struct CommandExecutor;

impl CommandExecutor {
    /// Executes a shell script with the provided arguments and environment.
    ///
    /// This function:
    /// 1. Sets up the working directory (if specified)
    /// 2. Configures environment variables from directives and arguments
    /// 3. Executes the script using `sh -c`
    /// 4. Captures and displays stdout/stderr
    /// 5. Formats detailed error messages if execution fails
    ///
    /// # Arguments
    ///
    /// * `command` - The command being executed (for error reporting)
    /// * `args` - Command arguments as key-value pairs (also set as env vars)
    /// * `script` - The shell script to execute
    /// * `env_vars` - Environment variables from directives
    /// * `cwd` - Optional working directory for script execution
    /// * `command_path` - Full path to command (for error reporting)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if execution succeeded,
    /// `Err(message)` with a formatted error message if execution failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Script execution fails to start
    /// - Script exits with non-zero status code
    ///
    /// The error message includes:
    /// - Command path and arguments
    /// - Working directory
    /// - Script preview
    /// - Exit code
    /// - Helpful suggestions (e.g., missing commands)
    pub fn execute(
        command: &Command,
        args: &HashMap<String, String>,
        script: &str,
        env_vars: &HashMap<String, String>,
        cwd: Option<&str>,
        command_path: Option<&[String]>,
    ) -> Result<(), String> {
        let mut cmd = ProcessCommand::new("sh");
        cmd.arg("-c");
        cmd.arg(script);

        if let Some(cwd_path) = cwd {
            cmd.current_dir(cwd_path);
        }

        // Set environment variables from directives
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Set command arguments as environment variables
        for (key, value) in args {
            cmd.env(key.to_uppercase(), value);
            cmd.env(key, value);
        }

        // Capture output for error reporting
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to start script execution: {}", e))?;

        // Print stdout and stderr
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            let stderr_str = String::from_utf8_lossy(&output.stderr);

            // Build beautiful formatted error message
            let error_msg = format_error_message(
                command,
                command_path,
                args,
                cwd,
                script,
                exit_code,
                &stderr_str,
            );

            return Err(error_msg);
        }

        Ok(())
    }
}

fn format_error_message(
    command: &Command,
    command_path: Option<&[String]>,
    args: &HashMap<String, String>,
    cwd: Option<&str>,
    script: &str,
    exit_code: i32,
    stderr_str: &str,
) -> String {
    use std::fmt::Write;

    // ANSI color codes
    const RESET: &str = "\x1b[0m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const CYAN: &str = "\x1b[36m";
    const GRAY: &str = "\x1b[90m";
    const BRIGHT_RED: &str = "\x1b[91m";

    let mut output = String::new();

    // Header
    writeln!(
        output,
        "\n{}╔═══════════════════════════════════════════════════════════════╗{}",
        RED, RESET
    )
    .unwrap();
    writeln!(
        output,
        "{}║{}  {}❌ Execution Error{}",
        RED, RESET, BRIGHT_RED, RESET
    )
    .unwrap();
    writeln!(
        output,
        "{}╚═══════════════════════════════════════════════════════════════╝{}\n",
        RED, RESET
    )
    .unwrap();

    // Command information
    let command_display = if let Some(path) = command_path {
        format!("nest {}", path.join(" "))
    } else {
        command.name.clone()
    };

    writeln!(output, "{}📋 Command:{} {}", CYAN, RESET, command_display).unwrap();

    // Arguments
    if !args.is_empty() {
        let args_str: Vec<String> = args
            .iter()
            .map(|(k, v)| format!("{}{}{}={}{}{}", YELLOW, k, RESET, CYAN, v, RESET))
            .collect();
        writeln!(
            output,
            "{}🔧 Arguments:{} {}",
            CYAN,
            RESET,
            args_str.join(", ")
        )
        .unwrap();
    }

    // Working directory
    if let Some(cwd_path) = cwd {
        writeln!(
            output,
            "{}📁 Working directory:{} {}",
            CYAN, RESET, cwd_path
        )
        .unwrap();
    }

    // Script preview
    let script_lines: Vec<&str> = script.lines().take(5).collect();
    if !script_lines.is_empty() {
        writeln!(output, "\n{}📜 Script preview:{}", CYAN, RESET).unwrap();
        writeln!(
            output,
            "{}┌─────────────────────────────────────────────────────────┐{}",
            GRAY, RESET
        )
        .unwrap();
        for (i, line) in script_lines.iter().enumerate() {
            let line_num = format!("{:2}", i + 1);
            writeln!(
                output,
                "{}│{} {} {}{}│{}",
                GRAY, RESET, line_num, line, RESET, GRAY
            )
            .unwrap();
        }
        if script.lines().count() > 5 {
            let more_lines = script.lines().count() - 5;
            writeln!(
                output,
                "{}│{}   ... ({} more lines){}│{}",
                GRAY, RESET, more_lines, RESET, GRAY
            )
            .unwrap();
        }
        writeln!(
            output,
            "{}└─────────────────────────────────────────────────────────┘{}",
            GRAY, RESET
        )
        .unwrap();
    }

    // Exit code
    writeln!(
        output,
        "\n{}⚠️  Exit code:{} {}{}{}",
        YELLOW, RESET, BRIGHT_RED, exit_code, RESET
    )
    .unwrap();

    // Command not found message
    if stderr_str.contains("command not found") {
        if let Some(cmd) = extract_missing_command(stderr_str) {
            writeln!(
                output,
                "\n{}💡 Suggestion:{} Command {}{}{} not found.",
                CYAN, RESET, YELLOW, cmd, RESET
            )
            .unwrap();
            writeln!(
                output,
                "   Please install it or check your PATH environment variable."
            )
            .unwrap();
        }
    } else if !stderr_str.trim().is_empty() {
        // Additional error output
        writeln!(output, "\n{}📝 Error details:{}", CYAN, RESET).unwrap();
        writeln!(
            output,
            "{}┌─────────────────────────────────────────────────────────┐{}",
            GRAY, RESET
        )
        .unwrap();
        for line in stderr_str.trim().lines() {
            writeln!(
                output,
                "{}│{} {}{}{}│{}",
                GRAY, RESET, RED, line, RESET, GRAY
            )
            .unwrap();
        }
        writeln!(
            output,
            "{}└─────────────────────────────────────────────────────────┘{}",
            GRAY, RESET
        )
        .unwrap();
    }

    output
}

fn extract_missing_command(stderr: &str) -> Option<String> {
    // Extract command name from various error patterns:
    // "sh: line X: command: command not found"
    // "command: command not found"
    // ": command: command not found"

    // Pattern 1: "sh: line X: command: command not found"
    if let Some(start) = stderr.find(": ") {
        let after_colon = &stderr[start + 2..];
        if let Some(end) = after_colon.find(": command not found") {
            let cmd_part = &after_colon[..end];
            // If it starts with "line", skip to the actual command
            if let Some(cmd_start) = cmd_part.find(": ") {
                let cmd = cmd_part[cmd_start + 2..].trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_string());
                }
            } else {
                let cmd = cmd_part.trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_string());
                }
            }
        }
    }

    None
}
