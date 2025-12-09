//! Nest - A task runner and CLI generator based on declarative configuration files.
//!
//! This is the main entry point for the Nest application. It loads and parses
//! the configuration file (nestfile), builds a CLI interface dynamically,
//! and executes commands based on user input.

mod constants;
mod nestparse;

use constants::{FLAG_SHOW, FLAG_VERSION, FORMAT_AST, FORMAT_JSON};
use nestparse::cli::{handle_example, handle_json, handle_show_ast, handle_update, handle_version, CliGenerator};
use nestparse::command_handler::CommandHandler;
use nestparse::file::read_config_file;
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
    
    // Check for update command (first argument after program name)
    if args.len() > 1 && args[1] == "update" {
        handle_update();
        return;
    }

    let (parse_result, config_path) = match load_and_parse_config() {
        Ok(result) => result,
        Err(e) => {
            nestparse::output::OutputFormatter::error(&e.to_string());
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
    );
    let mut cli = match generator.build_cli() {
        Ok(cli) => cli,
        Err(e) => {
            nestparse::output::OutputFormatter::error(&e.to_string());
            process::exit(1);
        }
    };
    let matches = cli.clone().get_matches();

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
/// 1. Searches for a configuration file in the current directory
/// 2. Reads the file content
/// 3. Parses it into commands, variables, and constants
///
/// # Returns
///
/// - `Ok((parse_result, path))` - Successfully parsed configuration and file path
/// - `Err(message)` - Error message describing what went wrong
///
/// # Errors
///
/// Returns an error if:
/// - No configuration file is found
/// - File cannot be read
/// - Parsing fails
fn load_and_parse_config() -> Result<(ParseResult, std::path::PathBuf), String> {
    let config_path =
        find_config_file().ok_or_else(|| "Configuration file not found".to_string())?;

    let content =
        read_config_file(&config_path).map_err(|e| format!("Error reading file: {}", e))?;

    // Process includes before parsing
    let mut visited = std::collections::HashSet::new();
    let processed_content = process_includes(&content, &config_path, &mut visited)
        .map_err(|e| format!("Include error: {}", e))?;

    let mut parser = Parser::new(&processed_content);
    let parse_result = parser
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
