//! Dynamic CLI generation from parsed commands.
//!
//! This module builds a clap-based CLI interface dynamically from the parsed
//! command structure. It handles nested commands, parameters, flags, and
//! special cases like default subcommands.

use super::ast::{Command, Directive, Parameter, Value};
use super::env::EnvironmentManager;
use super::executor::CommandExecutor;
use super::template::TemplateProcessor;
use crate::constants::{
    APP_NAME, BOOL_FALSE, BOOL_TRUE, DEFAULT_SUBCOMMAND, FLAG_DRY_RUN, FLAG_EXAMPLE, FLAG_SHOW,
    FLAG_VERBOSE, FLAG_VERSION, FORMAT_AST, FORMAT_JSON, RESERVED_FLAG_HELP, RESERVED_FLAG_VERSION,
    RESERVED_SHORT_HELP, RESERVED_SHORT_OPTIONS, RESERVED_SHORT_VERSION, SHORT_VERSION,
};
use clap::{Arg, ArgAction, Command as ClapCommand};
use std::collections::HashMap;

/// Represents a conflict with a reserved short option.
struct ShortAliasConflict {
    short: char,
    param_name: String,
    command_path: String,
}

/// Generates a CLI interface from parsed commands.
///
/// This struct builds a clap `Command` structure dynamically based on
/// the commands parsed from the configuration file. It handles:
/// - Nested command hierarchies
/// - Parameter and flag definitions
/// - Default subcommands
/// - Special flags (--version, --show)
///
/// # Lifetime Management
///
/// Uses `Box::leak` to create `&'static str` values required by clap.
/// This is necessary because clap requires static string references for
/// argument IDs and names.
pub struct CliGenerator {
    /// The parsed commands from the configuration file
    commands: Vec<Command>,
    /// Pre-allocated static strings for default command parameters
    default_param_ids: std::collections::HashMap<String, &'static str>,
}

impl CliGenerator {
    /// Creates a new CLI generator from parsed commands.
    ///
    /// # Arguments
    ///
    /// * `commands` - The list of commands parsed from the configuration file
    ///
    /// # Returns
    ///
    /// Returns a new `CliGenerator` instance ready to build CLI interfaces.
    pub fn new(commands: Vec<Command>) -> Self {
        let default_param_ids = Self::preallocate_default_param_ids(&commands);
        Self {
            commands,
            default_param_ids,
        }
    }

    fn preallocate_default_param_ids(
        commands: &[Command],
    ) -> std::collections::HashMap<String, &'static str> {
        let mut ids = std::collections::HashMap::new();

        for command in commands {
            if let Some(default_cmd) = command
                .children
                .iter()
                .find(|c| c.name == DEFAULT_SUBCOMMAND)
            {
                for param in &default_cmd.parameters {
                    let static_str: &'static str = Box::leak(param.name.clone().into_boxed_str());
                    ids.insert(param.name.clone(), static_str);
                }
            }
        }

        ids
    }

    /// Gets a static string reference for a parameter name.
    ///
    /// This is used to satisfy clap's requirement for `&'static str` references.
    /// If the parameter ID was pre-allocated (for default commands), it returns
    /// that. Otherwise, it leaks a new string.
    ///
    /// # Arguments
    ///
    /// * `param_name` - The parameter name
    ///
    /// # Returns
    ///
    /// Returns a `&'static str` reference to the parameter name.
    pub fn get_param_id(&self, param_name: &str) -> &'static str {
        self.default_param_ids
            .get(param_name)
            .copied()
            .unwrap_or_else(|| Box::leak(param_name.to_string().into_boxed_str()))
    }

    /// Builds a complete clap CLI structure from the parsed commands.
    ///
    /// This function creates the root CLI command and recursively adds all
    /// commands and subcommands with their parameters and flags.
    ///
    /// # Returns
    ///
    /// Returns a `ClapCommand` ready to be used with `get_matches()`.
    ///
    /// # Errors
    ///
    /// Returns an error if there are conflicts with reserved short option names.
    pub fn build_cli(&self) -> Result<ClapCommand, String> {
        // Check for reserved short option conflicts
        self.validate_short_aliases()?;

        let mut app = Self::create_base_cli();

        for command in &self.commands {
            app = self.add_command_to_clap(app, command);
        }

        Ok(app)
    }

    /// Validates that no parameter aliases conflict with reserved short options.
    ///
    /// Reserved short options:
    /// - `h` - reserved for `--help`
    /// - `V` - reserved for `--version`
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if no conflicts found, `Err(message)` with a user-friendly error message otherwise.
    fn validate_short_aliases(&self) -> Result<(), String> {
        let mut conflicts = Vec::new();

        self.collect_short_aliases(&mut conflicts);

        for conflict in conflicts {
            let reserved_for = match conflict.short {
                RESERVED_SHORT_HELP => RESERVED_FLAG_HELP,
                RESERVED_SHORT_VERSION => RESERVED_FLAG_VERSION,
                _ => unreachable!(),
            };

            return Err(format!(
                "❌ Error: Short option '-{}' is reserved for '--{}'\n\n\
                 Parameter '{}' in command '{}' uses reserved alias '-{}'.\n\
                 Please use a different alias (e.g., change '{}|{}' to '{}|i').",
                conflict.short,        // 1: -{}
                reserved_for,          // 2: --{}
                conflict.param_name,   // 3: '{}'
                conflict.command_path, // 4: '{}'
                conflict.short,        // 5: -{}
                conflict.param_name,   // 6: '{}|{}' first
                conflict.short,        // 7: '{}|{}' second
                conflict.param_name    // 8: '{}|i'
            ));
        }

        Ok(())
    }

    /// Collects all short aliases from all commands and checks for conflicts.
    fn collect_short_aliases(&self, conflicts: &mut Vec<ShortAliasConflict>) {
        for command in &self.commands {
            self.collect_short_aliases_recursive(command, &[command.name.clone()], conflicts);
        }
    }

    fn collect_short_aliases_recursive(
        &self,
        command: &Command,
        path: &[String],
        conflicts: &mut Vec<ShortAliasConflict>,
    ) {
        // Check parameters of this command
        for param in &command.parameters {
            if let Some(alias) = &param.alias {
                if let Some(short) = alias.chars().next() {
                    if RESERVED_SHORT_OPTIONS.contains(&short) {
                        conflicts.push(ShortAliasConflict {
                            short,
                            param_name: param.name.clone(),
                            command_path: path.join(" "),
                        });
                    }
                }
            }
        }

        // Check default subcommand parameters
        if let Some(default_cmd) = command
            .children
            .iter()
            .find(|c| c.name == DEFAULT_SUBCOMMAND)
        {
            for param in &default_cmd.parameters {
                if let Some(alias) = &param.alias {
                    if let Some(short) = alias.chars().next() {
                        if RESERVED_SHORT_OPTIONS.contains(&short) {
                            let mut path = path.to_vec();
                            path.push(DEFAULT_SUBCOMMAND.to_string());
                            conflicts.push(ShortAliasConflict {
                                short,
                                param_name: param.name.clone(),
                                command_path: path.join(" "),
                            });
                        }
                    }
                }
            }
        }

        // Recursively check child commands
        for child in &command.children {
            let mut child_path = path.to_vec();
            child_path.push(child.name.clone());
            self.collect_short_aliases_recursive(child, &child_path, conflicts);
        }
    }

    fn create_base_cli() -> ClapCommand {
        let version = env!("CARGO_PKG_VERSION");
        let about = format!("Nest {}", version);
        ClapCommand::new(APP_NAME)
            .about(about)
            .arg(
                Arg::new(FLAG_VERSION)
                    .long(FLAG_VERSION)
                    .short(SHORT_VERSION)
                    .action(ArgAction::SetTrue)
                    .hide(true)
                    .help("Print version information"),
            )
            .arg(
                Arg::new(FLAG_SHOW)
                    .long(FLAG_SHOW)
                    .value_name("FORMAT")
                    .value_parser([FORMAT_JSON, FORMAT_AST])
                    .hide(true)
                    .help("Show commands in different formats (json, ast)"),
            )
            .arg(
                Arg::new(FLAG_EXAMPLE)
                    .long(FLAG_EXAMPLE)
                    .action(ArgAction::SetTrue)
                    .hide(true)
                    .help("Copy example nestfile to current directory"),
            )
            .arg(
                Arg::new(FLAG_DRY_RUN)
                    .long(FLAG_DRY_RUN)
                    .short('n')
                    .action(ArgAction::SetTrue)
                    .help("Show what would be executed without actually running it"),
            )
            .arg(
                Arg::new(FLAG_VERBOSE)
                    .long(FLAG_VERBOSE)
                    .short('v')
                    .action(ArgAction::SetTrue)
                    .help("Show detailed output including environment variables and working directory"),
            )
    }

    fn add_command_to_clap(&self, mut app: ClapCommand, command: &Command) -> ClapCommand {
        let cmd_name: &'static str = Box::leak(command.name.clone().into_boxed_str());
        let mut subcmd = ClapCommand::new(cmd_name).arg_required_else_help(false);

        subcmd = Self::add_description(subcmd, &command.directives);
        subcmd = Self::add_parameters(subcmd, &command.parameters, self);
        subcmd = Self::add_default_args_if_needed(subcmd, command, self);

        for child in &command.children {
            subcmd = self.add_command_to_clap(subcmd, child);
        }

        app = app.subcommand(subcmd);
        app
    }

    fn add_description(mut subcmd: ClapCommand, directives: &[Directive]) -> ClapCommand {
        if let Some(desc) = Self::get_directive_value(directives, "desc") {
            subcmd = subcmd.about(desc);
        }
        subcmd
    }

    fn add_parameters(
        mut subcmd: ClapCommand,
        parameters: &[Parameter],
        generator: &CliGenerator,
    ) -> ClapCommand {
        // First, add all named arguments (they don't use indices)
        for param in parameters {
            if param.is_named {
                // Use parameter name directly as ID (same as used in extract_bool_flag)
                let param_id: &'static str = Box::leak(param.name.clone().into_boxed_str());
                let arg = generator.parameter_to_arg_with_id(param, param_id);
                subcmd = subcmd.arg(arg);
            }
        }

        // Then, add all positional arguments with sequential indices
        let mut positional_index = 1; // Start from 1 (0 is command name)
        for param in parameters {
            if !param.is_named {
                let arg = generator.parameter_to_arg_positional(param, positional_index);
                subcmd = subcmd.arg(arg);
                positional_index += 1;
            }
        }

        subcmd
    }

    fn add_default_args_if_needed(
        mut subcmd: ClapCommand,
        command: &Command,
        generator: &CliGenerator,
    ) -> ClapCommand {
        if !command.children.is_empty() {
            if let Some(default_cmd) = command
                .children
                .iter()
                .find(|c| c.name == DEFAULT_SUBCOMMAND)
            {
                // First, add all named arguments
                for param in &default_cmd.parameters {
                    if param.is_named {
                        let param_id = generator.get_param_id(&param.name);
                        let arg = generator.parameter_to_arg_with_id(param, param_id);
                        subcmd = subcmd.arg(arg);
                    }
                }

                // Then, add all positional arguments with sequential indices
                let mut positional_index = 1;
                for param in &default_cmd.parameters {
                    if !param.is_named {
                        let arg = generator.parameter_to_arg_positional(param, positional_index);
                        subcmd = subcmd.arg(arg);
                        positional_index += 1;
                    }
                }
            }
        }
        subcmd
    }

    fn parameter_to_arg_with_id(&self, param: &Parameter, param_id: &'static str) -> Arg {
        let mut arg = Arg::new(param_id);

        match param.param_type.as_str() {
            "bool" => Self::build_bool_flag(&mut arg, param, param_id),
            _ => Self::build_value_arg(&mut arg, param, param_id),
        }

        arg
    }

    fn parameter_to_arg_positional(&self, param: &Parameter, index: usize) -> Arg {
        let param_name: &'static str = Box::leak(param.name.clone().into_boxed_str());
        let mut arg = Arg::new(param_name).index(index);

        match param.param_type.as_str() {
            "bool" => {
                // Positional bool arguments are not common, but we'll support them
                let help_text = format!("Positional argument: {} (bool)", param.name);
                // If no default value, make it required
                if param.default.is_none() {
                    arg = arg.required(true).help(help_text);
                } else {
                    arg = arg.required(false).help(help_text);
                }
            }
            _ => {
                let help_text = if param.default.is_some() {
                    format!("Positional argument: {} ({})", param.name, param.param_type)
                } else {
                    format!(
                        "Required positional argument: {} ({})",
                        param.name, param.param_type
                    )
                };
                // If no default value, make it required
                if param.default.is_none() {
                    arg = arg.required(true).help(help_text);
                } else {
                    arg = arg.required(false).help(help_text);
                }
            }
        }

        arg
    }

    fn build_bool_flag(arg: &mut Arg, param: &Parameter, param_id: &'static str) {
        // Allow boolean flags to accept true/false values or be used as a flag (defaults to true)
        let mut new_arg = arg
            .clone()
            .long(param_id)
            .action(ArgAction::Set)
            .value_parser([BOOL_TRUE, BOOL_FALSE])
            .num_args(0..=1)
            .help(format!(
                "Flag: {} (true/false, or use without value for true)",
                param.name
            ));

        if let Some(alias) = &param.alias {
            if let Some(short) = alias.chars().next() {
                new_arg = new_arg.short(short);
            }
        }
        *arg = new_arg;
    }

    fn build_value_arg(arg: &mut Arg, param: &Parameter, param_id: &'static str) {
        if param.default.is_some() {
            Self::build_optional_arg(arg, param, param_id);
        } else {
            Self::build_required_arg(arg, param, param_id);
        }
    }

    fn build_optional_arg(arg: &mut Arg, param: &Parameter, param_id: &'static str) {
        let help_text = format!("Parameter: {} ({})", param.name, param.param_type);
        let mut new_arg = arg
            .clone()
            .long(param_id)
            .action(ArgAction::Set)
            .help(help_text);

        if let Some(alias) = &param.alias {
            if let Some(short) = alias.chars().next() {
                new_arg = new_arg.short(short);
            }
        }
        *arg = new_arg;
    }

    fn build_required_arg(arg: &mut Arg, param: &Parameter, param_id: &'static str) {
        let help_text = format!("Required parameter: {} ({})", param.name, param.param_type);
        let mut new_arg = arg
            .clone()
            .long(param_id)
            .help(help_text)
            .required(true)
            .action(ArgAction::Set);

        if let Some(alias) = &param.alias {
            if let Some(short) = alias.chars().next() {
                new_arg = new_arg.short(short);
            }
        }
        *arg = new_arg;
    }

    /// Converts a Value to its string representation.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to convert
    ///
    /// # Returns
    ///
    /// Returns `Some(string)` with the string representation,
    /// or `None` if conversion is not possible.
    ///
    /// Arrays are joined with commas.
    pub fn value_to_string(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::Array(a) => Some(a.join(",")),
        }
    }

    fn get_directive_value(directives: &[Directive], name: &str) -> Option<String> {
        directives.iter().find_map(|d| match (d, name) {
            (Directive::Desc(s), "desc") => Some(s.clone()),
            (Directive::Cwd(s), "cwd") => Some(s.clone()),
            (Directive::Env(s), "env") => Some(s.clone()),
            (Directive::Script(s), "script") => Some(s.clone()),
            _ => None,
        })
    }

    /// Finds a command by its path.
    ///
    /// # Arguments
    ///
    /// * `path` - The command path (e.g., ["dev", "default"])
    ///
    /// # Returns
    ///
    /// Returns `Some(command)` if found, `None` otherwise.
    pub fn find_command(&self, path: &[String]) -> Option<&Command> {
        let mut current = &self.commands;
        let mut found: Option<&Command> = None;

        for name in path {
            found = current.iter().find(|c| &c.name == name);
            if let Some(cmd) = found {
                current = &cmd.children;
            } else {
                return None;
            }
        }

        found
    }

    /// Checks if a command has a default subcommand.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the command has a child named "default", `false` otherwise.
    pub fn has_default_command(&self, command: &Command) -> bool {
        command
            .children
            .iter()
            .any(|c| c.name == DEFAULT_SUBCOMMAND)
    }

    /// Executes a command with the provided arguments.
    ///
    /// This function:
    /// 1. Extracts the script from the command's directives
    /// 2. Processes template variables in the script
    /// 3. Extracts environment variables from directives
    /// 4. Executes the script with the configured environment
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute
    /// * `args` - Arguments to pass to the command
    /// * `command_path` - Full path to the command (for error reporting)
    /// * `dry_run` - If true, show what would be executed without running it
    /// * `verbose` - If true, show detailed output
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if execution succeeded,
    /// `Err(message)` if execution failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Command has no script directive
    /// - Script execution fails
    pub fn execute_command(
        &self,
        command: &Command,
        args: &HashMap<String, String>,
        command_path: Option<&[String]>,
        dry_run: bool,
        verbose: bool,
    ) -> Result<(), String> {
        let script = Self::get_directive_value(&command.directives, "script")
            .ok_or_else(|| "Command has no script directive".to_string())?;

        let processed_script = TemplateProcessor::process(&script, args);
        let env_vars = EnvironmentManager::extract_env_vars(&command.directives);
        let cwd = Self::get_directive_value(&command.directives, "cwd");

        CommandExecutor::execute(
            command,
            args,
            &processed_script,
            &env_vars,
            cwd.as_deref(),
            command_path,
            dry_run,
            verbose,
        )
    }
}

pub fn handle_version() {
    use super::output::colors;
    use super::output::OutputFormatter;
    println!(
        "{}nest{} {}",
        colors::BRIGHT_BLUE,
        colors::RESET,
        OutputFormatter::value(env!("CARGO_PKG_VERSION"))
    );
    std::process::exit(0);
}

/// Handles the --show json flag.
///
/// Converts commands to JSON format and prints them.
///
/// # Arguments
///
/// * `commands` - The list of commands to serialize
///
/// # Returns
///
/// Returns `Ok(())` if successful, `Err(error)` if serialization fails.
pub fn handle_json(commands: &[Command]) -> Result<(), Box<dyn std::error::Error>> {
    use super::json::to_json;
    let json = to_json(commands)?;
    println!("{}", json);
    Ok(())
}

/// Handles the --show ast flag.
///
/// Prints commands in a tree format showing the AST structure.
///
/// # Arguments
///
/// * `commands` - The list of commands to display
pub fn handle_show_ast(commands: &[Command]) {
    use super::display::print_command;
    use super::output::colors;
    println!(
        "{}🌳{} {}AST Structure:{}\n",
        colors::BRIGHT_GREEN,
        colors::RESET,
        colors::BRIGHT_CYAN,
        colors::RESET
    );
    for command in commands {
        print_command(command, 0);
        println!();
    }
}

/// Handles the --example flag.
///
/// Downloads the example nestfile from GitHub and saves it as "nestfile" in the current directory.
///
/// # Errors
///
/// Exits with code 1 if:
/// - curl or wget is not available
/// - Download fails
/// - The file cannot be written to the current directory
/// - Nestfile already exists in the current directory
pub fn handle_example() {
    use std::env;
    use std::fs;
    use std::process::Command;

    use super::output::OutputFormatter;

    // Get current directory
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            OutputFormatter::error(&format!("Error getting current directory: {}", e));
            std::process::exit(1);
        }
    };

    // Write to nestfile in current directory
    let target_path = current_dir.join("nestfile");

    // Check if nestfile already exists
    if target_path.exists() {
        OutputFormatter::error("nestfile already exists in the current directory");
        OutputFormatter::info("Please remove it first or choose a different location.");
        std::process::exit(1);
    }

    // GitHub raw URL for nestfile.example
    let url = "https://raw.githubusercontent.com/quonaro/nest/main/nestfile.example";

    OutputFormatter::info("Downloading nestfile.example from GitHub...");

    // Try curl first, then wget
    let content = match Command::new("curl").args(&["-fsSL", url]).output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).to_string()
        }
        Ok(_) => {
            // curl exists but failed, try wget
            match Command::new("wget").args(&["-qO-", url]).output() {
                Ok(output) if output.status.success() => {
                    String::from_utf8_lossy(&output.stdout).to_string()
                }
                Ok(_) => {
                    OutputFormatter::error("Both curl and wget failed to download file");
                    std::process::exit(1);
                }
                Err(_) => {
                    OutputFormatter::error("Neither curl nor wget is available");
                    OutputFormatter::info("Please install curl or wget to use this feature.");
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            // curl not found, try wget
            match Command::new("wget").args(&["-qO-", url]).output() {
                Ok(output) if output.status.success() => {
                    String::from_utf8_lossy(&output.stdout).to_string()
                }
                Ok(_) => {
                    OutputFormatter::error("wget failed to download file");
                    std::process::exit(1);
                }
                Err(_) => {
                    OutputFormatter::error("Neither curl nor wget is available");
                    OutputFormatter::info("Please install curl or wget to use this feature.");
                    std::process::exit(1);
                }
            }
        }
    };

    // Write content to nestfile
    use super::output::colors;
    match fs::write(&target_path, content) {
        Ok(_) => {
            OutputFormatter::success("Created nestfile in current directory");
            println!(
                "  {}Location:{} {}",
                OutputFormatter::help_label("Location:"),
                colors::RESET,
                OutputFormatter::path(&target_path.display().to_string())
            );
        }
        Err(e) => {
            OutputFormatter::error(&format!("Error writing nestfile: {}", e));
            std::process::exit(1);
        }
    }
}
