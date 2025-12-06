//! Nest - A task runner and CLI generator based on declarative configuration files.
//!
//! This is the main entry point for the Nest application. It loads and parses
//! the configuration file (nestfile), builds a CLI interface dynamically,
//! and executes commands based on user input.

mod constants;
mod nestparse;

use constants::{FLAG_EXAMPLE, FLAG_SHOW, FLAG_VERSION, FORMAT_AST, FORMAT_JSON};
use nestparse::cli::{handle_example, handle_json, handle_show_ast, handle_version, CliGenerator};
use nestparse::command_handler::CommandHandler;
use nestparse::file::read_config_file;
use nestparse::parser::Parser;
use nestparse::path::find_config_file;
use std::process;
use clap::Command as ClapCommand;

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
    // Build a minimal CLI first to check for special flags that don't need config
    let minimal_cli = ClapCommand::new("nest")
        .arg(
            clap::Arg::new(FLAG_EXAMPLE)
                .long(FLAG_EXAMPLE)
                .action(clap::ArgAction::SetTrue)
                .hide(true),
        );
    let minimal_matches = minimal_cli.get_matches();

    // Handle --example flag before loading config (it doesn't need config)
    if minimal_matches.get_flag(FLAG_EXAMPLE) {
        handle_example();
        return;
    }

    let commands = match load_and_parse_config() {
        Ok(commands) => commands,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let generator = CliGenerator::new(commands.clone());
    let mut cli = match generator.build_cli() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };
    let matches = cli.clone().get_matches();

    if handle_special_flags(&matches, &commands) {
        return;
    }

    let command_path = extract_command_path(&matches);

    if command_path.is_empty() {
        cli.print_help().unwrap();
        process::exit(0);
    }

    if let Some(command) = generator.find_command(&command_path) {
        handle_command_execution(&matches, command, &command_path, &generator);
    } else {
        eprintln!("Error: Command not found: {}", command_path.join(" "));
        process::exit(1);
    }
}

/// Loads and parses the configuration file.
///
/// This function:
/// 1. Searches for a configuration file in the current directory
/// 2. Reads the file content
/// 3. Parses it into a list of Command structures
///
/// # Returns
///
/// - `Ok(commands)` - Successfully parsed list of commands
/// - `Err(message)` - Error message describing what went wrong
///
/// # Errors
///
/// Returns an error if:
/// - No configuration file is found
/// - File cannot be read
/// - Parsing fails
fn load_and_parse_config() -> Result<Vec<nestparse::ast::Command>, String> {
    let config_path =
        find_config_file().ok_or_else(|| "Configuration file not found".to_string())?;

    let content =
        read_config_file(&config_path).map_err(|e| format!("Error reading file: {}", e))?;

    let mut parser = Parser::new(&content);
    parser.parse().map_err(|e| format!("Parse error: {:?}", e))
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
                    eprintln!("Error: JSON generation failed: {}", e);
                    process::exit(1);
                }
                return true;
            }
            _ => {
                eprintln!(
                    "Error: Unknown format: {}. Available: {}, {}",
                    format, FORMAT_JSON, FORMAT_AST
                );
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
) {
    if !command.children.is_empty() {
        if !generator.has_default_command(command) {
            if let Err(_) = CommandHandler::handle_group_without_default(command, command_path) {
                process::exit(1);
            }
            process::exit(0);
        } else {
            if let Err(e) = CommandHandler::handle_default_command(matches, command_path, generator)
            {
                eprintln!("Execution error: {}", e);
                process::exit(1);
            }
            return;
        }
    }

    let current_matches = matches
        .subcommand()
        .map(|(_, sub_matches)| sub_matches)
        .unwrap_or(matches);

    if let Err(e) =
        CommandHandler::handle_regular_command(current_matches, command, generator, command_path)
    {
        eprintln!("Execution error: {}", e);
        process::exit(1);
    }
}
