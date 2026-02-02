//! Dynamic CLI generation from parsed commands.
//!
//! This module builds a clap-based CLI interface dynamically from the parsed
//! command structure. It handles nested commands, parameters, flags, and
//! special cases like default subcommands.

use super::ast::{Command, Directive, Parameter, Value};

use crate::constants::{
    APP_NAME, BOOL_FALSE, BOOL_TRUE, DEFAULT_SUBCOMMAND, FLAG_COMPLETE, FLAG_CONFIG, FLAG_DRY_RUN,
    FLAG_EXAMPLE, FLAG_SHOW, FLAG_UPDATE, FLAG_VERBOSE, FLAG_VERSION, FORMAT_AST, FORMAT_JSON,
    SHORT_VERSION,
};
use clap::{Arg, ArgAction, Command as ClapCommand};

// Removed: ShortAliasConflict - validation is now done in validator module

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
    /// Note: This validation is also done in the validator module.
    /// This check is kept for backward compatibility and early error detection.
    pub fn build_cli(&self) -> Result<ClapCommand, String> {
        // Note: Reserved alias validation is also done in validator module
        // This check is kept for early error detection before CLI building
        // If validator is called first, this should not be needed
        let mut app = Self::create_base_cli();

        for command in &self.commands {
            app = self.add_command_to_clap(app, command);
        }

        Ok(app)
    }

    // Removed: validate_short_aliases, collect_short_aliases, collect_short_aliases_recursive
    // Validation is now done in the validator module before CLI building

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
                Arg::new(FLAG_CONFIG)
                    .long(FLAG_CONFIG)
                    .short('c')
                    .value_name("PATH")
                    .hide(true)
                    .help("Specify path to configuration file"),
            )
            .subcommand(
                ClapCommand::new(FLAG_UPDATE)
                    .hide(true)
                    .about("Update Nest CLI to the latest version"),
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
            .arg(
                Arg::new(FLAG_COMPLETE)
                    .long(FLAG_COMPLETE)
                    .value_name("SHELL")
                    .hide(true)
                    .help("Generate and install shell completion (bash, zsh, fish, powershell, elvish). Use -V to show script content."),
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
        use super::ast::ParamKind;

        // First, add all named arguments (they don't use indices)
        for param in parameters {
            if param.is_named {
                // Use parameter name directly as ID (same as used in extract_bool_flag)
                let param_id: &'static str = Box::leak(param.name.clone().into_boxed_str());
                let arg = generator.parameter_to_arg_with_id(param, param_id);
                subcmd = subcmd.arg(arg);
            }
        }

        // Then, add all positional arguments with sequential indices.
        // Wildcard parameters are represented as positional arguments that can
        // accept multiple values.
        let mut positional_index = 1; // Start from 1 (0 is command name)
        let positional_params: Vec<&Parameter> =
            parameters.iter().filter(|p| !p.is_named).collect();

        for (idx, param) in positional_params.iter().enumerate() {
            match &param.kind {
                ParamKind::Normal => {
                    let arg = generator.parameter_to_arg_positional(param, positional_index);
                    subcmd = subcmd.arg(arg);
                    positional_index += 1;
                }
                ParamKind::Wildcard { name: _, count } => {
                    let param_name: &'static str = Box::leak(param.name.clone().into_boxed_str());
                    let mut arg = Arg::new(param_name).index(positional_index);

                    // Wildcard parameters always accept hyphen-prefixed values.
                    arg = arg.allow_hyphen_values(true);

                    if let Some(n) = count {
                        // Fixed-size wildcard: must capture exactly n arguments.
                        arg = arg.num_args(*n).help(format!(
                            "Wildcard positional segment '{}' capturing exactly {} argument(s)",
                            param.name, n
                        ));
                    } else {
                        // Unbounded wildcard: only safe when it's the last positional parameter.
                        let is_last = idx == positional_params.len() - 1;
                        if is_last {
                            arg = arg.num_args(1..).trailing_var_arg(true).help(format!(
                                "Wildcard positional segment '{}' capturing remaining arguments",
                                param.name
                            ));
                        } else {
                            // Fallback: require at least one argument but let clap handle
                            // distribution. This is intentionally strict to avoid ambiguity.
                            arg = arg.num_args(1..).help(format!(
                                "Wildcard positional segment '{}' capturing one or more arguments",
                                param.name
                            ));
                        }
                    }

                    subcmd = subcmd.arg(arg);
                    positional_index += 1;
                }
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
    /// Arrays are joined with spaces by default.
    pub fn value_to_string(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::Array(a) => Some(a.join(" ")),
        }
    }

    /// Checks if the directive's OS requirement matches the current system.
    fn check_os_match(os: &Option<String>) -> bool {
        match os {
            Some(required_os) => {
                let current_os = std::env::consts::OS;
                if required_os.eq_ignore_ascii_case(current_os) {
                    return true;
                }
                if required_os.eq_ignore_ascii_case("unix") && cfg!(unix) {
                    return true;
                }
                if required_os.eq_ignore_ascii_case("bsd") && current_os.contains("bsd") {
                    return true;
                }
                false
            }
            None => true,
        }
    }

    pub fn get_directive_value(directives: &[Directive], name: &str) -> Option<String> {
        let mut best_match: Option<String> = None;
        let mut best_score = 0;

        for d in directives {
            let (val, os, target_name) = match d {
                Directive::Desc(s) => (Some(s.clone()), &None, "desc"),
                Directive::Cwd(s) => (Some(s.clone()), &None, "cwd"),
                Directive::Env(k, v, _) => (Some(format!("{}={}", k, v)), &None, "env"),
                Directive::EnvFile(s, _) => (Some(s.clone()), &None, "env"),

                Directive::Script(s, os, _) => (Some(s.clone()), os, "script"),
                Directive::Before(s, os, _) => (Some(s.clone()), os, "before"),
                Directive::After(s, os, _) => (Some(s.clone()), os, "after"),
                Directive::Fallback(s, os, _) => (Some(s.clone()), os, "fallback"),
                Directive::Finally(s, os, _) => (Some(s.clone()), os, "finally"),
                Directive::Validate(_, _) => (None, &None, "validate"),
                _ => (None, &None, ""),
            };

            if target_name == name && Self::check_os_match(os) {
                let score = if os.is_some() { 2 } else { 1 };
                if score > best_score {
                    best_score = score;
                    best_match = val;
                }
            }
        }
        best_match
    }

    /// Finds a command by its path.
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

    pub fn has_default_command(&self, command: &Command) -> bool {
        command
            .children
            .iter()
            .any(|c| c.name == DEFAULT_SUBCOMMAND)
    }
}
