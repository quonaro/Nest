mod constants;
mod nestparse;

use nestparse::cli::{handle_json, handle_show_ast, handle_version, CliGenerator};
use nestparse::command_handler::CommandHandler;
use nestparse::file::read_config_file;
use nestparse::parser::Parser;
use nestparse::path::find_config_file;
use std::process;

fn main() {
    let commands = match load_and_parse_config() {
        Ok(commands) => commands,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let generator = CliGenerator::new(commands.clone());
    let mut cli = generator.build_cli();
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

fn load_and_parse_config() -> Result<Vec<nestparse::ast::Command>, String> {
    let config_path =
        find_config_file().ok_or_else(|| "Configuration file not found".to_string())?;

    let content =
        read_config_file(&config_path).map_err(|e| format!("Error reading file: {}", e))?;

    let mut parser = Parser::new(&content);
    parser.parse().map_err(|e| format!("Parse error: {:?}", e))
}

fn handle_special_flags(matches: &clap::ArgMatches, commands: &[nestparse::ast::Command]) -> bool {
    if matches.get_flag("version") {
        handle_version();
        return true;
    }

    if let Some(format) = matches.get_one::<String>("show") {
        match format.as_str() {
            "ast" => {
                handle_show_ast(commands);
                return true;
            }
            "json" => {
                if let Err(e) = handle_json(commands) {
                    eprintln!("Error: JSON generation failed: {}", e);
                    process::exit(1);
                }
                return true;
            }
            _ => {
                eprintln!("Error: Unknown format: {}. Available: json, ast", format);
                process::exit(1);
            }
        }
    }

    false
}

fn extract_command_path(matches: &clap::ArgMatches) -> Vec<String> {
    let mut path = Vec::new();
    let mut current_matches = matches;

    while let Some((name, sub_matches)) = current_matches.subcommand() {
        path.push(name.to_string());
        current_matches = sub_matches;
    }

    path
}

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
