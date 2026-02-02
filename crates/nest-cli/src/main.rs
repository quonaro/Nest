//! Nest - A task runner and CLI generator based on declarative configuration files.
//!
//! This is the main entry point for the Nest application. It loads and parses
//! the configuration file (nestfile), builds a CLI interface dynamically,
//! and executes commands based on user input.

use nest_core::constants::{
    CMD_CHECK, CMD_LIST, FLAG_CHECK, FLAG_CLEAN, FLAG_COMPLETE, FLAG_DOCTOR, FLAG_EXAMPLE,
    FLAG_INIT, FLAG_LIST, FLAG_SHOW, FLAG_STD, FLAG_UNINSTALL, FLAG_UPDATE, FLAG_VERBOSE,
    FORMAT_AST, FORMAT_JSON,
};
use nest_core::nestparse::cli::CliGenerator;
use nest_core::nestparse::command_handler::CommandHandler;
use nest_core::nestparse::completion::CompletionManager;
use nest_core::nestparse::file::read_file_unchecked;
use nest_core::nestparse::handlers::{
    handle_example, handle_init, handle_json, handle_show_ast, handle_update, handle_version,
};
use nest_core::nestparse::include::process_includes;
use nest_core::nestparse::parser::{ParseError, ParseResult, Parser};
use nest_core::nestparse::path::find_config_file;
use nest_core::nestparse::standard_commands::{
    handle_check, handle_clean, handle_doctor, handle_list, handle_uninstall,
};
use nest_core::nestparse::validator::{print_validation_errors, validate_commands};
use std::process;
use std::sync::atomic::{AtomicU32, Ordering};

static CHILD_PID: AtomicU32 = AtomicU32::new(0);

/// Main entry point of the application.
fn main() {
    // Register signal handler for cleanup
    let _ = ctrlc::set_handler(move || {
        let pid = CHILD_PID.load(Ordering::SeqCst);
        if pid != 0 {
            // Try to kill the child process group (if it exists)
            #[cfg(unix)]
            {
                // We don't wait for output here to avoid hanging the signal handler
                let _ = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(format!("{}", pid))
                    .spawn();
            }
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(&["/F", "/PID", &pid.to_string()])
                    .spawn();
            }
        }
        std::process::exit(130);
    });

    // Check for special flags that don't need config by parsing args manually
    let args: Vec<String> = std::env::args().collect();

    // Find the first argument that looks like a command (doesn't start with -)
    // We skip the value of --config if present.
    let mut first_command_idx = args.len();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if !arg.starts_with('-') {
            first_command_idx = i;
            break;
        }
        // Skip values for known global flags that take them
        if arg == "--config" || arg == "-c" {
            i += 1;
        }
        i += 1;
    }

    // Only handle global flags if they appear BEFORE the command
    let global_args = &args[..first_command_idx];
    let has_global_flag = |flag: &str| global_args.iter().any(|a| a == &format!("--{}", flag));

    // --version or -V
    if global_args.iter().any(|a| a == "--version" || a == "-V") {
        handle_version();
        return;
    }

    // --std
    if has_global_flag(FLAG_STD) {
        nest_core::nestparse::standard_commands::handle_std_help();
        return;
    }

    // --example
    if has_global_flag(FLAG_EXAMPLE) {
        handle_example();
        return;
    }

    // --init
    if has_global_flag(FLAG_INIT) {
        let force = global_args.iter().any(|a| a == "--force" || a == "-f");
        handle_init(force);
        return;
    }

    // --update
    if has_global_flag(FLAG_UPDATE) {
        let recreate = global_args.iter().any(|a| a == "--recreate");
        handle_update(recreate);
        return;
    }

    // --doctor
    if has_global_flag(FLAG_DOCTOR) {
        handle_doctor();
        return;
    }

    // --clean
    if has_global_flag(FLAG_CLEAN) {
        handle_clean();
        return;
    }

    // --uninstall
    if has_global_flag(FLAG_UNINSTALL) {
        handle_uninstall();
        return;
    }

    // Check for --config flag and extract config path
    let mut config_path_arg: Option<&str> = None;
    let mut first_non_flag_index: Option<usize> = None;

    // Start from index 1 (skip program name)
    for (idx, arg) in args.iter().enumerate().skip(1) {
        // Detect first non-flag argument (command name)
        if first_non_flag_index.is_none() && !arg.starts_with('-') {
            first_non_flag_index = Some(idx);
        }

        if arg == "--config" || arg == "-c" {
            // Only treat as global config if it appears before first non-flag
            if let Some(cmd_idx) = first_non_flag_index {
                if idx >= cmd_idx {
                    break;
                }
            }
            if let Some(path) = args.get(idx + 1) {
                config_path_arg = Some(path.as_str());
            }
            break;
        }
    }

    let (parse_result, config_path) = match load_and_parse_config(config_path_arg) {
        Ok(result) => result,
        Err(e) => {
            // User request: simplify error message for missing config
            if config_path_arg.is_none() && e.contains("Configuration file not found") {
                println!("nestfile not found");
                println!("Run 'nest --init' to create one.");
                println!("Run 'nest --std' to see standard commands.");
                process::exit(1);
            }

            nest_core::nestparse::output::OutputFormatter::error(&e.to_string());
            if config_path_arg.is_none() {
                nest_core::nestparse::output::OutputFormatter::info(
                    "Tip: You can specify a custom config file path using --config <path>",
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

    // --check (requires config)
    if has_global_flag(FLAG_CHECK) {
        // If user defined 'check', let them run it. Otherwise run built-in check.
        if !parse_result.commands.iter().any(|c| c.name == CMD_CHECK) {
            handle_check(&config_path);
            return;
        }
    }

    // --list (requires config)
    if has_global_flag(FLAG_LIST) {
        // If user defined 'list', let them run it. Otherwise run built-in list.
        if !parse_result.commands.iter().any(|c| c.name == CMD_LIST) {
            handle_list(&parse_result.commands);
            return;
        }
    }

    let generator = CliGenerator::new(parse_result.commands.clone());

    let runtime = nest_core::nestparse::runtime::Runtime::new(
        parse_result.commands.clone(),
        parse_result.variables.clone(),
        parse_result.constants.clone(),
        parse_result.functions.clone(),
        Some(Box::new(|pid: u32| {
            CHILD_PID.store(pid, Ordering::SeqCst);
        })),
    );

    let mut cli = match generator.build_cli() {
        Ok(cli) => cli,
        Err(e) => {
            nest_core::nestparse::output::OutputFormatter::error(&e.to_string());
            process::exit(1);
        }
    };
    let matches = cli.clone().get_matches();

    // Handle --complete flag
    if let Some(shell_name) = matches.get_one::<String>(FLAG_COMPLETE) {
        let verbose = matches.get_flag(FLAG_VERBOSE);
        if let Err(e) =
            nest_core::nestparse::completion::CompletionManager::handle_completion_request(
                &mut cli,
                shell_name,
                verbose,
                &config_path,
            )
        {
            nest_core::nestparse::output::OutputFormatter::error(&e);
            process::exit(1);
        }
        return;
    }

    // Automatically generate/update completion scripts
    if let Ok(completion_manager) = CompletionManager::new() {
        if let Ok(true) = completion_manager.needs_regeneration(&config_path) {
            if completion_manager
                .generate_all_completions(&mut cli, &config_path)
                .is_ok()
            {
                if let Ok(Some(_)) = completion_manager.auto_install_completion(&config_path) {}
            }
        }
    }

    if handle_special_flags(&matches, &parse_result.commands) {
        return;
    }

    let command_path = extract_command_path(&matches);

    if command_path.is_empty() {
        if let Err(e) = cli.print_help() {
            nest_core::nestparse::output::OutputFormatter::error(&format!(
                "Failed to print help: {}",
                e
            ));
            process::exit(1);
        }
        process::exit(0);
    }

    if let Some(command) = generator.find_command(&command_path) {
        // Check for --watch flag in root args
        let watch_pattern = args
            .iter()
            .position(|a| a == "--watch")
            .and_then(|i| args.get(i + 1).cloned());

        // Also check if command has > watch: directive
        let directive_watch_patterns = command
            .directives
            .iter()
            .filter_map(|d| match d {
                nest_core::nestparse::ast::Directive::Watch(patterns) => Some(patterns.clone()),
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>();

        let should_watch = watch_pattern.is_some() || !directive_watch_patterns.is_empty();

        if should_watch {
            let mut patterns = directive_watch_patterns;
            if let Some(p) = watch_pattern {
                // CLI flag adds to the patterns (or overrides? Let's add for now to be safe)
                patterns.push(p);
            }

            // Remove duplicates
            patterns.sort();
            patterns.dedup();

            if patterns.is_empty() {
                nest_core::nestparse::output::OutputFormatter::error("Watch mode enabled but no patterns specified via --watch or > watch: directive.");
                process::exit(1);
            }

            let config = nest_core::nestparse::watcher::WatcherConfig {
                patterns,
                debounce_ms: 200, // Slightly higher debounce for Safety
            };

            let exec_closure = || {
                // We need to re-parse arguments each time? No, matches are static.
                // Just re-run the handler.
                // Note: Logic inside handler uses `process::exit`, which kills the watcher.
                // WE NEED TO REFACTOR COMMAND HANDLER TO NOT EXIT ON SUCCESS
                // OR WE NEED TO CATCH IT.
                // Refactoring CommandHandler to return Result instead of exit is better.
                // For now, let's assume we can't easily change CommandHandler's exit behavior without big refactor.
                // Wait, CommandHandler::handle_regular_command returns Result<(), String>.
                // It only exits on error inside main.rs logic below.

                // Let's create a wrapper that doesn't exit process
                handle_command_execution_no_exit(
                    &matches,
                    command,
                    &command_path,
                    &generator,
                    &runtime,
                    &matches,
                )
            };

            if let Err(e) = nest_core::nestparse::watcher::run_watch_loop(config, exec_closure) {
                nest_core::nestparse::output::OutputFormatter::error(&format!(
                    "Watch loop error: {}",
                    e
                ));
                process::exit(1);
            }
        } else {
            handle_command_execution(
                &matches,
                command,
                &command_path,
                &generator,
                &runtime,
                &matches,
            );
        }
    } else {
        nest_core::nestparse::output::OutputFormatter::error(&format!(
            "Command not found: {}",
            command_path.join(" ")
        ));
        process::exit(1);
    }
}

// Wrapper that returns Result instead of exiting
// Wrapper that returns Result instead of exiting
fn handle_command_execution_no_exit(
    matches: &clap::ArgMatches,
    command: &nest_core::nestparse::ast::Command,
    command_path: &[String],
    generator: &CliGenerator,
    runtime: &nest_core::nestparse::runtime::Runtime,
    root_matches: &clap::ArgMatches,
) -> Result<(), String> {
    if !command.children.is_empty() {
        if !generator.has_default_command(command) {
            CommandHandler::handle_group_without_default(command, command_path)
                .map_err(|_| "Failed to handle group command".to_string())
        } else {
            CommandHandler::handle_default_command(
                matches,
                command_path,
                generator,
                runtime,
                root_matches,
            )
        }
    } else {
        // We need to extract the subcommand matches again
        let current_matches = {
            let mut current = matches;
            while let Some((_, sub_matches)) = current.subcommand() {
                current = sub_matches;
            }
            current
        };

        CommandHandler::handle_regular_command(
            current_matches,
            command,
            generator,
            runtime,
            command_path,
            root_matches,
        )
    }
}

fn load_and_parse_config(
    config_path_arg: Option<&str>,
) -> Result<(ParseResult, std::path::PathBuf), String> {
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

    let mut visited = std::collections::HashSet::new();
    let processed_content = process_includes(&content, &config_path, &mut visited)
        .map_err(|e| format!("Include error: {}", e))?;

    let mut content_with_source = String::new();
    if let Ok(canonical_path) = config_path.canonicalize() {
        content_with_source.push_str(&format!("# @source: {}\n", canonical_path.display()));
    }
    content_with_source.push_str(&processed_content);

    let mut parser = Parser::new(&content_with_source);
    let mut parse_result = parser.parse().map_err(|e| match e {
        ParseError::UnexpectedEndOfFile(line) => {
            format!("Parse error at line {}: Unexpected end of file.", line)
        }
        ParseError::InvalidSyntax(msg, line) => {
            format!("Parse error at line {}: {}", line, msg)
        }
        ParseError::InvalidIndent(line) => {
            format!("Parse error at line {}: Invalid indentation.", line)
        }
        ParseError::DeprecatedSyntax(msg, line) => {
            format!("Deprecated syntax error at line {}:\n{}", line, msg)
        }
    })?;

    // Merge duplicate commands
    parse_result.commands = nest_core::nestparse::merge::merge_commands(parse_result.commands);

    Ok((parse_result, config_path))
}

fn handle_special_flags(
    matches: &clap::ArgMatches,
    commands: &[nest_core::nestparse::ast::Command],
) -> bool {
    // --version handled manually
    // if matches.get_flag(FLAG_VERSION) ...

    if let Some(format) = matches.get_one::<String>(FLAG_SHOW) {
        match format.as_str() {
            FORMAT_AST => {
                handle_show_ast(commands);
                return true;
            }
            FORMAT_JSON => {
                if let Err(e) = handle_json(commands) {
                    nest_core::nestparse::output::OutputFormatter::error(&format!(
                        "JSON generation failed: {}",
                        e
                    ));
                    process::exit(1);
                }
                return true;
            }
            _ => {
                nest_core::nestparse::output::OutputFormatter::error(&format!(
                    "Unknown format: {}",
                    format
                ));
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
    command: &nest_core::nestparse::ast::Command,
    command_path: &[String],
    generator: &CliGenerator,
    runtime: &nest_core::nestparse::runtime::Runtime,
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
                runtime,
                root_matches,
            ) {
                eprint!("{}", e);
                process::exit(1);
            }
            return;
        }
    }

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
        runtime,
        command_path,
        root_matches,
    ) {
        eprint!("{}", e);
        process::exit(1);
    }
}
