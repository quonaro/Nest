//! Script execution and error reporting.
//!
//! This module handles the actual execution of shell scripts defined in commands,
//! including environment variable setup, working directory configuration,
//! and detailed error reporting with beautiful formatting.

use super::ast::Command;
use std::collections::HashMap;
use std::env;
use std::process::{Command as ProcessCommand, Stdio};

/// Context for script execution containing all necessary parameters.
pub struct ExecutionContext<'a> {
    pub command: &'a Command,
    pub args: &'a HashMap<String, String>,
    pub env_vars: &'a HashMap<String, String>,
    pub cwd: Option<&'a str>,
    pub command_path: Option<&'a [String]>,
    pub dry_run: bool,
    pub verbose: bool,
    pub privileged: bool,
    pub pid_callback: Option<&'a dyn Fn(u32)>,
    pub hide_output: bool,
}

/// Executes shell scripts for commands.
///
/// This is a utility struct with static methods for script execution.
pub struct CommandExecutor;

impl CommandExecutor {
    /// Detects shell from shebang and removes it from script.
    /// Returns (shell_command, script_without_shebang)
    fn detect_shell_and_remove_shebang(script: &str) -> (&str, String) {
        let trimmed = script.trim_start();
        if trimmed.starts_with("#!") {
            // Extract shell from shebang
            let shebang_line = trimmed.lines().next().unwrap_or("");
            let shell_path = shebang_line.trim_start_matches("#!").trim();

            // Determine shell command
            let shell = if shell_path.contains("bash") {
                "bash"
            } else if shell_path.contains("zsh") {
                "zsh"
            } else if shell_path.contains("fish") {
                "fish"
            } else {
                "sh"
            };

            // Remove shebang line from script
            let script_without_shebang = script.lines().skip(1).collect::<Vec<_>>().join("\n");

            (shell, script_without_shebang)
        } else {
            ("sh", script.to_string())
        }
    }

    /// Executes a shell script with the provided arguments and environment.
    ///
    /// This function:
    /// 1. Sets up the working directory (if specified)
    /// 2. Configures environment variables from directives and arguments
    /// 3. Executes the script using the shell from shebang (or `sh -c` by default)
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
    pub fn execute(script: &str, context: &ExecutionContext) -> Result<(), String> {
        // Check privileged access BEFORE execution
        if context.privileged && !context.dry_run && !Self::check_privileged_access() {
            return Err(Self::format_privileged_error(
                context.command,
                context.command_path,
            ));
        }

        // Show dry-run preview
        if context.dry_run {
            Self::show_dry_run_preview(script, context);
            return Ok(());
        }

        // Show verbose information if requested
        if context.verbose {
            Self::show_verbose_info(script, context);
        }

        // Detect shell from shebang and remove it
        let (shell, script_without_shebang) = Self::detect_shell_and_remove_shebang(script);
        let script_to_execute = script_without_shebang.trim();

        let mut cmd = ProcessCommand::new(shell);
        cmd.arg("-c");
        cmd.arg(script_to_execute);

        if let Some(cwd_path) = context.cwd {
            cmd.current_dir(cwd_path);
        }

        // Set environment variables from directives
        for (key, value) in context.env_vars {
            cmd.env(key, value);
        }

        // Set command arguments as environment variables
        for (key, value) in context.args {
            cmd.env(key.to_uppercase(), value);
            cmd.env(key, value);
        }

        // Capture output - hide if requested
        if context.hide_output {
            cmd.stdin(Stdio::null());
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
        } else {
            cmd.stdin(Stdio::inherit());
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start script execution: {}", e))?;

        if let Some(callback) = context.pid_callback {
            callback(child.id());
        }

        // Wait for command to finish
        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for script execution: {}", e))?;

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);

            // Build beautiful formatted error message
            // Note: we don't have the stderr output since it was inherited directly to terminal
            let error_msg = format_error_message(
                context.command,
                context.command_path,
                context.args,
                context.cwd,
                script,
                exit_code,
                "(See output above)",
            );

            return Err(error_msg);
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

    pub fn show_dry_run_preview(script: &str, context: &ExecutionContext) {
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
        let command_display = if let Some(path) = context.command_path {
            format!("nest {}", path.join(" "))
        } else {
            context.command.name.clone()
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
        if !context.args.is_empty() {
            let args_str: Vec<String> = context
                .args
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
        if let Some(cwd_path) = context.cwd {
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
        if context.privileged {
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
        if context.verbose && !context.env_vars.is_empty() {
            writeln!(
                output,
                "\n{}🌍 Environment variables:{}",
                colors::CYAN,
                colors::RESET
            )
            .expect("Failed to format environment variables header in dry run");
            for (key, value) in context.env_vars {
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

    pub fn show_verbose_info(script: &str, context: &ExecutionContext) {
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

        let command_display = if let Some(path) = context.command_path {
            format!("nest {}", path.join(" "))
        } else {
            context.command.name.clone()
        };

        eprintln!(
            "{}📋 Command:{} {}",
            colors::CYAN,
            colors::RESET,
            command_display
        );

        if !context.args.is_empty() {
            let args_str: Vec<String> = context
                .args
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

        if let Some(cwd_path) = context.cwd {
            eprintln!(
                "{}📁 Working directory:{} {}",
                colors::CYAN,
                colors::RESET,
                cwd_path
            );
        }

        if context.privileged {
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

        if !context.env_vars.is_empty() {
            eprintln!(
                "\n{}🌍 Environment variables:{}",
                colors::CYAN,
                colors::RESET
            );
            for (key, value) in context.env_vars {
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

    /// Parses a command call from a string.
    ///
    /// Supports formats:
    /// - `command` - simple command
    /// - `group:command` - nested command
    /// - `command(arg="value")` - command with arguments
    /// - `group:command(arg="value")` - nested command with arguments
    ///
    /// Returns (command_path, args) if it's a command call, None otherwise.
    pub fn parse_command_call(line: &str) -> Option<(String, HashMap<String, String>)> {
        let trimmed = line.trim();

        // Check if line looks like a command call
        // Command calls should start with alphanumeric or underscore, and may contain colons
        // They should not contain shell operators like |, &&, ||, ;, >, <, etc.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        // Check for shell operators that indicate this is not a command call
        let shell_operators = [
            "|", "&&", "||", ";", ">", "<", ">>", "<<", "&", "$", "`", "[", "]", "=",
        ];

        // If it looks like a potential Nest call (ends with ')'), we bypass the shell operator check
        // because those operators are likely inside string arguments (e.g., SQL queries with ';').
        let is_potential_call = trimmed.contains('(') && trimmed.ends_with(')');

        if !is_potential_call && shell_operators.iter().any(|&op| trimmed.contains(op)) {
            return None;
        }

        // Check for shell keywords that indicate this is not a command call
        let shell_keywords = [
            "if", "then", "else", "elif", "fi", "case", "esac", "for", "while", "until", "do",
            "done", "function",
        ];
        let first_word = trimmed.split_whitespace().next().unwrap_or("");
        if shell_keywords.contains(&first_word) {
            return None;
        }

        // Try to parse as command call
        // Pattern: [group:]command[(args)]
        let command_path: String;
        let mut args = HashMap::new();

        // Check if there are parentheses (arguments)
        if let Some(open_paren) = trimmed.find('(') {
            // Extract command path (before parentheses)
            command_path = trimmed[..open_paren].trim().to_string();

            // Find matching closing parenthesis
            let mut depth = 0;
            let mut in_quotes = false;
            let mut quote_char = '\0';
            let mut close_paren = None;

            for (i, ch) in trimmed[open_paren..].char_indices() {
                match ch {
                    '"' | '\'' if !in_quotes => {
                        in_quotes = true;
                        quote_char = ch;
                    }
                    ch if ch == quote_char && in_quotes => {
                        in_quotes = false;
                    }
                    '(' if !in_quotes => {
                        depth += 1;
                    }
                    ')' if !in_quotes => {
                        depth -= 1;
                        if depth == 0 {
                            close_paren = Some(open_paren + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if let Some(close) = close_paren {
                let args_str = &trimmed[open_paren + 1..close];
                // Parse arguments using similar logic to dependency parsing
                args = Self::parse_command_args(args_str).unwrap_or_default();
            } else {
                // Unclosed parentheses - not a valid command call
                return None;
            }
        } else {
            // No arguments - just command path
            command_path = trimmed.to_string();
        }

        // Validate command path (should contain only alphanumeric, underscore, colon, hyphen)
        if command_path.is_empty() {
            return None;
        }

        // Check if it looks like a valid command path
        let is_valid = command_path
            .chars()
            .all(|c| c.is_alphanumeric() || c == ':' || c == '_' || c == '-')
            && !command_path.starts_with(':')
            && !command_path.ends_with(':');

        if !is_valid {
            return None;
        }

        Some((command_path, args))
    }

    /// Parses arguments from a command call argument string.
    /// Format: `name="value", name2=true, name3=123`
    fn parse_command_args(args_str: &str) -> Result<HashMap<String, String>, ()> {
        let mut args = HashMap::new();

        if args_str.trim().is_empty() {
            return Ok(args);
        }

        // Split by comma, but respect quotes
        let mut current = args_str.trim();
        while !current.is_empty() {
            let (arg_str, remainder) = Self::split_next_arg(current)?;

            if arg_str.is_empty() {
                break;
            }

            // Parse name=value
            let equals_pos = arg_str.find('=').ok_or(())?;

            let name = arg_str[..equals_pos].trim().to_string();
            let value_str = arg_str[equals_pos + 1..].trim();

            // Parse value (string, bool, or number)
            let value = Self::parse_command_value(value_str);

            args.insert(name, value);

            current = remainder.trim();
        }

        Ok(args)
    }

    /// Splits the next argument from the string, handling quotes.
    fn split_next_arg(s: &str) -> Result<(&str, &str), ()> {
        let mut in_quotes = false;
        let mut quote_char = '\0';

        for (i, ch) in s.char_indices() {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                }
                ch if ch == quote_char && in_quotes => {
                    in_quotes = false;
                }
                ',' if !in_quotes => {
                    return Ok((&s[..i], &s[i + 1..]));
                }
                _ => {}
            }
        }

        Ok((s, ""))
    }

    /// Parses a command argument value (string, bool, or number).
    fn parse_command_value(value_str: &str) -> String {
        let trimmed = value_str.trim();

        // String value (quoted)
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            // Remove quotes
            let unquoted = &trimmed[1..trimmed.len() - 1];
            // Unescape quotes
            unquoted
                .replace("\\\"", "\"")
                .replace("\\'", "'")
                .replace("\\\\", "\\")
        }
        // Boolean or number value (keep as is)
        else {
            trimmed.to_string()
        }
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
