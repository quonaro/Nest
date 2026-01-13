//! Nest - A task runner and CLI generator based on declarative configuration files.
//!
//! This is the main entry point for the Nest application. It loads and parses
//! the configuration file (nestfile), builds a CLI interface dynamically,
//! and executes commands based on user input.

mod constants;
mod nestparse;

use constants::{FLAG_COMPLETE, FLAG_SHOW, FLAG_VERBOSE, FLAG_VERSION, FORMAT_AST, FORMAT_JSON};
use nestparse::cli::{handle_example, handle_init, handle_json, handle_show_ast, handle_update, handle_version, CliGenerator};
use nestparse::command_handler::CommandHandler;
use nestparse::completion::CompletionManager;
use nestparse::file::read_file_unchecked;
use nestparse::include::process_includes;
use nestparse::parser::{Parser, ParseError, ParseResult};
use nestparse::path::find_config_file;
use nestparse::validator::{print_validation_errors, validate_commands};
use std::process;

/// Main entry point of the application.
///
/// This function:
/// 1. Loads and parses the configuration file (nestfile)
/// 2. Builds a dynamic CLI using clap based on the parsed commands
/// 3. Handles special flags (--version, --show)
/// 4. Executes the requested command or shows help
///
/// # Errors
///
/// Exits with code 1 if:
/// - Configuration file is not found
/// - Configuration file cannot be read
/// - Parsing fails
/// - Command execution fails
fn main() {
    // Check for special flags that don't need config by parsing args manually
    let args: Vec<String> = std::env::args().collect();
    
    // Check for --version or -V
    if args.iter().any(|a| a == "--version" || a == "-V") {
        handle_version();
        return;
    }
    
    // Check for --example
    if args.iter().any(|a| a == "--example") {
        handle_example();
        return;
    }
    
    // Check for --init
    if args.iter().any(|a| a == "--init") {
        let force = args.iter().any(|a| a == "--force" || a == "-f");
        handle_init(force);
        return;
    }
    
    // Check for update command (first argument after program name)
    if args.len() > 1 && args[1] == "update" {
        handle_update();
        return;
    }

    // Check for --config flag and extract config path
    //
    // Important behavioural rule:
    // - Global --config / -c is only honoured when it appears
    //   *before* the first non-flag argument (i.e. before the command name).
    // - This ensures that constructs like:
    //     nest psql -c "\dt"
    //   are treated as passing -c to the subcommand, not to Nest itself.
    //
    // Examples:
    // - `nest -c nestfile dev`      -> uses custom config (OK)
    // - `nest dev -c nestfile`      -> -c belongs to `dev` / its script (ignored by Nest)
    // - `nest --config nestfile dev`-> uses custom config (OK)
    // - `nest dev --config nestfile`-> ignored by Nest
    let mut config_path_arg: Option<&str> = None;
    let mut first_non_flag_index: Option<usize> = None;

    // Start from index 1 (skip program name)
    for (idx, arg) in args.iter().enumerate().skip(1) {
        // Detect first non-flag argument (command name)
        if first_non_flag_index.is_none() && !arg.starts_with('-') {
            first_non_flag_index = Some(idx);
        }

        if arg == "--config" || arg == "-c" {
            // Only treat as global config if it appears before first non-flag (command name)
            if let Some(cmd_idx) = first_non_flag_index {
                if idx >= cmd_idx {
                    // This -c/--config is after the command name -> belongs to subcommand
                    break;
                }
            }

            // Take next argument as config path if available
            if let Some(path) = args.get(idx + 1) {
                config_path_arg = Some(path.as_str());
            }
            break;
        }
    }

    let (parse_result, config_path) = match load_and_parse_config(config_path_arg) {
        Ok(result) => result,
        Err(e) => {
            nestparse::output::OutputFormatter::error(&e.to_string());
            if config_path_arg.is_none() {
                nestparse::output::OutputFormatter::info(
                    "Tip: You can specify a custom config file path using --config <path>"
                );
            }
            process::exit(1);
        }
    };

    // Validate configuration
    if let Err(validation_errors) = validate_commands(&parse_result.commands, &config_path) {
        print_validation_errors(&validation_errors, &config_path);
        process::exit(1);
    }

    let generator = CliGenerator::new(
        parse_result.commands.clone(),
        parse_result.variables.clone(),
        parse_result.constants.clone(),
        parse_result.functions.clone(),
    );
    let mut cli = match generator.build_cli() {
        Ok(cli) => cli,
        Err(e) => {
            nestparse::output::OutputFormatter::error(&e.to_string());
            process::exit(1);
        }
    };
    let matches = cli.clone().get_matches();

    // Handle --complete flag (generate completion script)
    if let Some(shell_name) = matches.get_one::<String>(FLAG_COMPLETE) {
        let verbose = matches.get_flag(FLAG_VERBOSE);
        if let Err(e) = nestparse::completion::CompletionManager::handle_completion_request(
            &mut cli, 
            shell_name,
            verbose,
            &config_path,
        ) {
            nestparse::output::OutputFormatter::error(&e);
            process::exit(1);
        }
        return;
    }

    // Automatically generate/update completion scripts if nestfile changed
    if let Ok(completion_manager) = CompletionManager::new() {
        if let Ok(needs_regeneration) = completion_manager.needs_regeneration(&config_path) {
            if needs_regeneration {
                // Silently generate completions in background (don't interrupt user workflow)
                if let Ok(_) = completion_manager.generate_all_completions(&mut cli, &config_path) {
                    // Completion scripts generated successfully
                    // Now try to auto-install for current shell
                    if let Ok(Some(_installed_shell)) = completion_manager.auto_install_completion(&config_path) {
                        // Completion installed successfully (or already was installed)
                        // User will need to reload shell or restart terminal
                    }
                }
            } else {
                // Scripts are up to date, but check if installation is needed
                if let Ok(Some(_)) = completion_manager.auto_install_completion(&config_path) {
                    // Installation check completed (already installed or just installed)
                }
            }
        }
    }

    if handle_special_flags(&matches, &parse_result.commands) {
        return;
    }

    let command_path = extract_command_path(&matches);

    if command_path.is_empty() {
        if let Err(e) = cli.print_help() {
            nestparse::output::OutputFormatter::error(&format!("Failed to print help: {}", e));
            process::exit(1);
        }
        process::exit(0);
    }

    if let Some(command) = generator.find_command(&command_path) {
        handle_command_execution(&matches, command, &command_path, &generator, &matches);
    } else {
        nestparse::output::OutputFormatter::error(&format!(
            "Command not found: {}",
            command_path.join(" ")
        ));
        process::exit(1);
    }
}

/// Loads and parses the configuration file.
///
/// This function:
/// 1. Uses provided config path or searches for a configuration file in the current directory
/// 2. Reads the file content
/// 3. Parses it into commands, variables, and constants
///
/// # Arguments
///
/// * `config_path_arg` - Optional path to config file (from --config flag)
///
/// # Returns
///
/// - `Ok((parse_result, path))` - Successfully parsed configuration and file path
/// - `Err(message)` - Error message describing what went wrong
///
/// # Errors
///
/// Returns an error if:
/// - No configuration file is found (when path not provided)
/// - File cannot be read
/// - Parsing fails
fn load_and_parse_config(config_path_arg: Option<&str>) -> Result<(ParseResult, std::path::PathBuf), String> {
    let config_path = if let Some(path_str) = config_path_arg {
        let path = std::path::PathBuf::from(path_str);
        if !path.exists() {
            return Err(format!("Configuration file not found: {}", path.display()));
        }
        if !path.is_file() {
            return Err(format!("Path is not a file: {}", path.display()));
        }
        path
    } else {
        find_config_file().ok_or_else(|| {
            "Configuration file not found. Searched for: nestfile, Nestfile, nest, Nest".to_string()
        })?
    };

    let content =
        read_file_unchecked(&config_path).map_err(|e| format!("Error reading file: {}", e))?;

    // Process includes before parsing
    let mut visited = std::collections::HashSet::new();
    let processed_content = process_includes(&content, &config_path, &mut visited)
        .map_err(|e| format!("Include error: {}", e))?;

    // Add source file marker for the main file at the beginning
    let mut content_with_source = String::new();
    if let Ok(canonical_path) = config_path.canonicalize() {
        content_with_source.push_str(&format!("# @source: {}\n", canonical_path.display()));
    }
    content_with_source.push_str(&processed_content);

    let mut parser = Parser::new(&content_with_source);
    let mut parse_result = parser
        .parse()
        .map_err(|e| {
            match e {
                ParseError::UnexpectedEndOfFile(line) => {
                    format!("Parse error at line {}: Unexpected end of file. Check for incomplete command definitions.", line)
                }
                ParseError::InvalidSyntax(msg, line) => {
                    format!("Parse error at line {}: {}", line, msg)
                }
                ParseError::InvalidIndent(line) => {
                    format!("Parse error at line {}: Invalid indentation. Make sure nested commands are properly indented (4 spaces per level).", line)
                }
            }
        })?;

    // Merge duplicate commands (e.g. from includes or overrides)
    parse_result.commands = nestparse::merger::merge_commands(parse_result.commands);

    Ok((parse_result, config_path))
}

/// Handles special global flags that don't execute commands.
///
/// Special flags include:
/// - `--version` / `-V` - Prints version information
/// - `--show json` - Outputs commands in JSON format
/// - `--show ast` - Outputs commands as an Abstract Syntax Tree
///
/// # Arguments
///
/// * `matches` - The parsed CLI arguments from clap
/// * `commands` - The list of parsed commands from the configuration file
///
/// # Returns
///
/// Returns `true` if a special flag was handled (and execution should stop),
/// `false` otherwise.
fn handle_special_flags(matches: &clap::ArgMatches, commands: &[nestparse::ast::Command]) -> bool {
    if matches.get_flag(FLAG_VERSION) {
        handle_version();
        return true;
    }

    if let Some(format) = matches.get_one::<String>(FLAG_SHOW) {
        match format.as_str() {
            FORMAT_AST => {
                handle_show_ast(commands);
                return true;
            }
            FORMAT_JSON => {
                if let Err(e) = handle_json(commands) {
                    nestparse::output::OutputFormatter::error(&format!(
                        "JSON generation failed: {}",
                        e
                    ));
                    process::exit(1);
                }
                return true;
            }
            _ => {
                nestparse::output::OutputFormatter::error(&format!(
                    "Unknown format: {}. Available: {}, {}",
                    format, FORMAT_JSON, FORMAT_AST
                ));
                process::exit(1);
            }
        }
    }

    false
}

/// Extracts the command path from parsed CLI arguments.
///
/// For nested commands like `nest dev build`, this returns `["dev", "build"]`.
/// For top-level commands like `nest build`, this returns `["build"]`.
///
/// # Arguments
///
/// * `matches` - The parsed CLI arguments from clap
///
/// # Returns
///
/// A vector of command names representing the path to the command.
/// Returns an empty vector if no command was specified.
fn extract_command_path(matches: &clap::ArgMatches) -> Vec<String> {
    let mut path = Vec::new();
    let mut current_matches = matches;

    while let Some((name, sub_matches)) = current_matches.subcommand() {
        path.push(name.to_string());
        current_matches = sub_matches;
    }

    path
}

/// Handles the execution of a command.
///
/// This function determines the type of command and routes execution accordingly:
/// - Group commands without a default subcommand: shows help
/// - Group commands with a default subcommand: executes the default
/// - Regular commands: executes the command's script
///
/// # Arguments
///
/// * `matches` - The parsed CLI arguments from clap
/// * `command` - The command to execute
/// * `command_path` - The full path to the command (e.g., ["dev", "default"])
/// * `generator` - The CLI generator used to find and execute commands
///
/// # Errors
///
/// Exits with code 1 if command execution fails.
fn handle_command_execution(
    matches: &clap::ArgMatches,
    command: &nestparse::ast::Command,
    command_path: &[String],
    generator: &CliGenerator,
    root_matches: &clap::ArgMatches,
) {
    if !command.children.is_empty() {
        if !generator.has_default_command(command) {
            if CommandHandler::handle_group_without_default(command, command_path).is_err() {
                process::exit(1);
            }
            process::exit(0);
        } else {
            if let Err(e) = CommandHandler::handle_default_command(
                matches,
                command_path,
                generator,
                root_matches,
            ) {
                // Error is already formatted in executor
                eprint!("{}", e);
                process::exit(1);
            }
            return;
        }
    }

    // Get matches for the deepest subcommand (e.g., for "nest db down", get matches for "down")
    let current_matches = {
        let mut current = matches;
        while let Some((_, sub_matches)) = current.subcommand() {
            current = sub_matches;
        }
        current
    };

    if let Err(e) = CommandHandler::handle_regular_command(
        current_matches,
        command,
        generator,
        command_path,
        root_matches,
    ) {
        // Error is already formatted in executor
        eprint!("{}", e);
        process::exit(1);
    }
}
