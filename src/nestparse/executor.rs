//! Script execution and error reporting.
//!
//! This module handles the actual execution of shell scripts defined in commands,
//! including environment variable setup, working directory configuration,
//! and detailed error reporting with beautiful formatting.

use super::ast::Command;
use std::collections::HashMap;
use std::process::{Command as ProcessCommand, Stdio};
use std::env;

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
    /// 3. Executes the script using `sh -c` (or shows preview if dry_run is true)
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
    /// * `dry_run` - If true, show what would be executed without running it
    /// * `verbose` - If true, show detailed output including env vars and cwd
    /// * `privileged` - If true, command requires privileged access (sudo/administrator)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if execution succeeded (or dry-run completed),
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
    #[allow(dead_code)]
    pub fn execute(
        command: &Command,
        args: &HashMap<String, String>,
        script: &str,
        env_vars: &HashMap<String, String>,
        cwd: Option<&str>,
        command_path: Option<&[String]>,
        dry_run: bool,
        verbose: bool,
        privileged: bool,
    ) -> Result<(), String> {
        // Check privileged access BEFORE execution
        if privileged && !dry_run {
            if !Self::check_privileged_access() {
                return Err(Self::format_privileged_error(command, command_path));
            }
        }

        // Show dry-run preview
        if dry_run {
            Self::show_dry_run_preview(command, command_path, args, env_vars, cwd, script, verbose, privileged);
            return Ok(());
        }

        // Show verbose information if requested
        if verbose {
            Self::show_verbose_info(command, command_path, args, env_vars, cwd, script, privileged);
        }

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

        // Print stdout and stderr only on success
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    /// Checks if the current process is running with privileged access.
    ///
    /// On Unix systems (Linux/macOS), checks if running as root (UID == 0) or via sudo.
    /// On Windows, checks if running as administrator.
    ///
    /// # Returns
    ///
    /// Returns `true` if running with privileged access, `false` otherwise.
    pub fn check_privileged_access() -> bool {
        #[cfg(unix)]
        {
            // Check if SUDO_USER is set (indicates running via sudo)
            if env::var("SUDO_USER").is_ok() {
                return true;
            }
            
            // Check if we're running as root by checking UID
            // Use id -u command to get effective user ID
            let test_cmd = ProcessCommand::new("sh")
                .arg("-c")
                .arg("[ $(id -u) -eq 0 ]")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output();
            
            if let Ok(output) = test_cmd {
                return output.status.success();
            }
            
            false
        }

        #[cfg(windows)]
        {
            // On Windows, check if running as administrator
            // Try to check if we can access admin-only resources
            use std::path::Path;
            let system32 = Path::new("C:\\Windows\\System32\\config");
            // Check if we can list system32 config directory (requires admin)
            system32.exists() && system32.read_dir().is_ok()
        }

        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }

    pub fn format_privileged_error(command: &Command, command_path: Option<&[String]>) -> String {
        use super::output::colors;
        use std::env::consts::OS;
        use std::fmt::Write;

        let mut output = String::new();

        let command_display = if let Some(path) = command_path {
            format!("nest {}", path.join(" "))
        } else {
            command.name.clone()
        };

        let sudo_command = if OS == "windows" {
            "Run PowerShell/CMD as Administrator, then: nest <command>"
        } else {
            "sudo nest <command>"
        };

        writeln!(
            output,
            "\n{}╔═══════════════════════════════════════════════════════════════╗{}",
            colors::RED,
            colors::RESET
        )
        .expect("Failed to format privileged error header");
        writeln!(
            output,
            "{}║{}  {}❌ Privileged Access Required{}",
            colors::RED,
            colors::RESET,
            colors::BRIGHT_RED,
            colors::RESET
        )
        .expect("Failed to format privileged error title");
        writeln!(
            output,
            "{}╚═══════════════════════════════════════════════════════════════╝{}\n",
            colors::RED,
            colors::RESET
        )
        .expect("Failed to format privileged error footer");

        writeln!(
            output,
            "{}📋 Command:{} {}",
            colors::CYAN,
            colors::RESET,
            command_display
        )
        .expect("Failed to format command in privileged error");

        writeln!(
            output,
            "\n{}⚠️  ERROR:{} This command requires privileged access, but you are not running with elevated privileges.",
            colors::YELLOW,
            colors::RESET
        )
        .expect("Failed to format privileged error message");
        writeln!(
            output,
            "   {}You MUST use {}{}{} or an alternative to run this command with elevated privileges.{}",
            colors::GRAY,
            colors::BRIGHT_CYAN,
            if OS == "windows" { "Run as Administrator" } else { "sudo" },
            colors::GRAY,
            colors::RESET
        )
        .expect("Failed to format privileged access instruction");
        writeln!(
            output,
            "   {}Example:{} {}{}{}",
            colors::GRAY,
            colors::RESET,
            colors::BRIGHT_CYAN,
            sudo_command,
            colors::RESET
        )
        .expect("Failed to format privileged command example");
        writeln!(
            output,
            "\n{}ℹ{} {}The command was not executed due to insufficient privileges.{}",
            colors::BRIGHT_CYAN,
            colors::RESET,
            colors::GRAY,
            colors::RESET
        )
        .expect("Failed to format privileged error info");

        output
    }


    pub fn show_dry_run_preview(
        command: &Command,
        command_path: Option<&[String]>,
        args: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
        cwd: Option<&str>,
        script: &str,
        verbose: bool,
        privileged: bool,
    ) {
        use super::output::colors;
        use std::fmt::Write;

        let mut output = String::new();

        // Header
        writeln!(
            output,
            "\n{}╔═══════════════════════════════════════════════════════════════╗{}",
            colors::BRIGHT_CYAN,
            colors::RESET
        )
        .expect("Failed to format dry run header");
        writeln!(
            output,
            "{}║{}  {}🔍 Dry Run Preview{}",
            colors::BRIGHT_CYAN,
            colors::RESET,
            colors::BRIGHT_BLUE,
            colors::RESET
        )
        .expect("Failed to format dry run title");
        writeln!(
            output,
            "{}╚═══════════════════════════════════════════════════════════════╝{}\n",
            colors::BRIGHT_CYAN,
            colors::RESET
        )
        .expect("Failed to format dry run footer");

        // Command information
        let command_display = if let Some(path) = command_path {
            format!("nest {}", path.join(" "))
        } else {
            command.name.clone()
        };

        writeln!(
            output,
            "{}📋 Command:{} {}",
            colors::CYAN,
            colors::RESET,
            command_display
        )
        .expect("Failed to format command in dry run");

        // Arguments
        if !args.is_empty() {
            let args_str: Vec<String> = args
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}{}{}={}{}{}",
                        colors::YELLOW,
                        k,
                        colors::RESET,
                        colors::CYAN,
                        v,
                        colors::RESET
                    )
                })
                .collect();
            writeln!(
                output,
                "{}🔧 Arguments:{} {}",
                colors::CYAN,
                colors::RESET,
                args_str.join(", ")
            )
            .expect("Failed to format arguments in dry run");
        }

        // Working directory
        if let Some(cwd_path) = cwd {
            writeln!(
                output,
                "{}📁 Working directory:{} {}",
                colors::CYAN,
                colors::RESET,
                cwd_path
            )
            .expect("Failed to format working directory in dry run");
        }

        // Privileged access requirement
        if privileged {
            use std::env::consts::OS;
            let sudo_command = if OS == "windows" {
                "Run as Administrator"
            } else {
                "sudo"
            };
            writeln!(
                output,
                "{}🔐 Privileged access:{} {}Required ({}){}",
                colors::YELLOW,
                colors::RESET,
                colors::BRIGHT_YELLOW,
                sudo_command,
                colors::RESET
            )
            .expect("Failed to format privileged access in dry run");
        }

        // Environment variables (if verbose)
        if verbose && !env_vars.is_empty() {
            writeln!(
                output,
                "\n{}🌍 Environment variables:{}",
                colors::CYAN,
                colors::RESET
            )
            .expect("Failed to format environment variables header in dry run");
            for (key, value) in env_vars {
                writeln!(
                    output,
                    "  {}{}{}={}{}{}",
                    colors::YELLOW,
                    key,
                    colors::RESET,
                    colors::CYAN,
                    value,
                    colors::RESET
                )
                .expect("Failed to format environment variable in dry run");
            }
        }

        // Script preview
        writeln!(
            output,
            "\n{}📜 Script to execute:{}",
            colors::CYAN,
            colors::RESET
        )
        .expect("Failed to format script header in dry run");
        writeln!(
            output,
            "{}┌─────────────────────────────────────────────────────────┐{}",
            colors::GRAY,
            colors::RESET
        )
        .expect("Failed to format script box top in dry run");
        for (i, line) in script.lines().enumerate() {
            let line_num = format!("{:2}", i + 1);
            writeln!(
                output,
                "{}│{} {} {}{}│{}",
                colors::GRAY,
                colors::RESET,
                line_num,
                line,
                colors::RESET,
                colors::GRAY
            )
            .expect("Failed to format script line in dry run");
        }
        writeln!(
            output,
            "{}└─────────────────────────────────────────────────────────┘{}",
            colors::GRAY,
            colors::RESET
        )
        .expect("Failed to format script box bottom in dry run");

        writeln!(
            output,
            "\n{}ℹ{} {}This is a dry run. The script was not executed.{}",
            colors::BRIGHT_CYAN,
            colors::RESET,
            colors::GRAY,
            colors::RESET
        )
        .expect("Failed to format dry run info message");

        eprint!("{}", output);
    }

    pub fn show_verbose_info(
        command: &Command,
        command_path: Option<&[String]>,
        args: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
        cwd: Option<&str>,
        script: &str,
        privileged: bool,
    ) {
        use super::output::colors;

        eprintln!(
            "\n{}╔═══════════════════════════════════════════════════════════════╗{}",
            colors::BRIGHT_BLUE,
            colors::RESET
        );
        eprintln!(
            "{}║{}  {}ℹ Verbose Mode{}",
            colors::BRIGHT_BLUE,
            colors::RESET,
            colors::BRIGHT_CYAN,
            colors::RESET
        );
        eprintln!(
            "{}╚═══════════════════════════════════════════════════════════════╝{}\n",
            colors::BRIGHT_BLUE,
            colors::RESET
        );

        let command_display = if let Some(path) = command_path {
            format!("nest {}", path.join(" "))
        } else {
            command.name.clone()
        };

        eprintln!(
            "{}📋 Command:{} {}",
            colors::CYAN,
            colors::RESET,
            command_display
        );

        if !args.is_empty() {
            let args_str: Vec<String> = args
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}{}{}={}{}{}",
                        colors::YELLOW,
                        k,
                        colors::RESET,
                        colors::CYAN,
                        v,
                        colors::RESET
                    )
                })
                .collect();
            eprintln!(
                "{}🔧 Arguments:{} {}",
                colors::CYAN,
                colors::RESET,
                args_str.join(", ")
            );
        }

        if let Some(cwd_path) = cwd {
            eprintln!(
                "{}📁 Working directory:{} {}",
                colors::CYAN,
                colors::RESET,
                cwd_path
            );
        }

        if privileged {
            use std::env::consts::OS;
            let sudo_command = if OS == "windows" {
                "Run as Administrator"
            } else {
                "sudo"
            };
            eprintln!(
                "{}🔐 Privileged access:{} {}Required ({}){}",
                colors::YELLOW,
                colors::RESET,
                colors::BRIGHT_YELLOW,
                sudo_command,
                colors::RESET
            );
        }

        if !env_vars.is_empty() {
            eprintln!(
                "\n{}🌍 Environment variables:{}",
                colors::CYAN,
                colors::RESET
            );
            for (key, value) in env_vars {
                eprintln!(
                    "  {}{}{}={}{}{}",
                    colors::YELLOW,
                    key,
                    colors::RESET,
                    colors::CYAN,
                    value,
                    colors::RESET
                );
            }
        }

        eprintln!("\n{}📜 Script:{}", colors::CYAN, colors::RESET);
        eprintln!(
            "{}┌─────────────────────────────────────────────────────────┐{}",
            colors::GRAY,
            colors::RESET
        );
        for (i, line) in script.lines().enumerate() {
            let line_num = format!("{:2}", i + 1);
            eprintln!(
                "{}│{} {} {}{}│{}",
                colors::GRAY,
                colors::RESET,
                line_num,
                line,
                colors::RESET,
                colors::GRAY
            );
        }
        eprintln!(
            "{}└─────────────────────────────────────────────────────────┘{}\n",
            colors::GRAY,
            colors::RESET
        );
    }
}

#[allow(dead_code)]
fn format_error_message(
    command: &Command,
    command_path: Option<&[String]>,
    args: &HashMap<String, String>,
    cwd: Option<&str>,
    script: &str,
    exit_code: i32,
    stderr_str: &str,
) -> String {
    use super::output::colors;
    use std::fmt::Write;

    let mut output = String::new();

    // Header
    writeln!(
        output,
        "\n{}╔═══════════════════════════════════════════════════════════════╗{}",
        colors::RED,
        colors::RESET
    )
    .expect("Failed to format error message header");
    writeln!(
        output,
        "{}║{}  {}❌ Execution Error{}",
        colors::RED,
        colors::RESET,
        colors::BRIGHT_RED,
        colors::RESET
    )
    .expect("Failed to format error message title");
    writeln!(
        output,
        "{}╚═══════════════════════════════════════════════════════════════╝{}\n",
        colors::RED,
        colors::RESET
    )
    .expect("Failed to format error message footer");

    // Command information
    let command_display = if let Some(path) = command_path {
        format!("nest {}", path.join(" "))
    } else {
        command.name.clone()
    };

    writeln!(
        output,
        "{}📋 Command:{} {}",
        colors::CYAN,
        colors::RESET,
        command_display
    )
    .expect("Failed to format command in error message");

    // Arguments
    if !args.is_empty() {
        let args_str: Vec<String> = args
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}{}{}={}{}{}",
                    colors::YELLOW,
                    k,
                    colors::RESET,
                    colors::CYAN,
                    v,
                    colors::RESET
                )
            })
            .collect();
        writeln!(
            output,
            "{}🔧 Arguments:{} {}",
            colors::CYAN,
            colors::RESET,
            args_str.join(", ")
        )
        .expect("Failed to format arguments in error message");
    }

    // Working directory
    if let Some(cwd_path) = cwd {
        writeln!(
            output,
            "{}📁 Working directory:{} {}",
            colors::CYAN,
            colors::RESET,
            cwd_path
        )
        .expect("Failed to format working directory in error message");
    }

    // Script preview
    let script_lines: Vec<&str> = script.lines().take(5).collect();
    if !script_lines.is_empty() {
        writeln!(
            output,
            "\n{}📜 Script preview:{}",
            colors::CYAN,
            colors::RESET
        )
        .expect("Failed to format script preview header in error message");
        for (i, line) in script_lines.iter().enumerate() {
            let line_num = format!("{:2}", i + 1);
            writeln!(
                output,
                "{}  {}{} {}{}",
                colors::GRAY,
                colors::RESET,
                line_num,
                line,
                colors::RESET
            )
            .expect("Failed to format script line in error message");
        }
        if script.lines().count() > 5 {
            let more_lines = script.lines().count() - 5;
            writeln!(
                output,
                "{}  {}... ({} more lines){}",
                colors::GRAY,
                colors::RESET,
                more_lines,
                colors::RESET
            )
            .expect("Failed to format script line count in error message");
        }
    }

    // Exit code
    writeln!(
        output,
        "\n{}⚠️  Exit code:{} {}{}{}",
        colors::YELLOW,
        colors::RESET,
        colors::BRIGHT_RED,
        exit_code,
        colors::RESET
    )
    .expect("Failed to format exit code in error message");

    // Command not found message
    if stderr_str.contains("command not found") {
        if let Some(cmd) = extract_missing_command(stderr_str) {
            writeln!(
                output,
                "\n{}💡 Suggestion:{} Command {}{}{} not found.",
                colors::CYAN,
                colors::RESET,
                colors::YELLOW,
                cmd,
                colors::RESET
            )
            .expect("Failed to format command not found suggestion in error message");
            writeln!(
                output,
                "   Please install it or check your PATH environment variable."
            )
            .expect("Failed to format command not found instruction in error message");
        }
    } else if !stderr_str.trim().is_empty() {
        // Additional error output - simple format without wrapping
        writeln!(
            output,
            "\n{}📝 Error details:{}",
            colors::CYAN,
            colors::RESET
        )
        .expect("Failed to format error details header in error message");
        
        // Output error as-is, line by line
        for line in stderr_str.trim().lines() {
            writeln!(
                output,
                "{}  {}{}{}",
                colors::GRAY,
                colors::RED,
                line,
                colors::RESET
            )
            .expect("Failed to format error detail line in error message");
        }
    }

    output
}

#[allow(dead_code)]
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
