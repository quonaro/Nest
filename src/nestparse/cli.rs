//! Dynamic CLI generation from parsed commands.
//!
//! This module builds a clap-based CLI interface dynamically from the parsed
//! command structure. It handles nested commands, parameters, flags, and
//! special cases like default subcommands.

use super::ast::{Command, Directive, Parameter, Value, Variable, Constant};
use super::env::EnvironmentManager;
use super::executor::CommandExecutor;
use super::template::TemplateProcessor;
use crate::constants::{
    APP_NAME, BOOL_FALSE, BOOL_TRUE, DEFAULT_SUBCOMMAND, FLAG_DRY_RUN, FLAG_EXAMPLE, FLAG_SHOW,
    FLAG_UPDATE, FLAG_VERBOSE, FLAG_VERSION, FORMAT_AST, FORMAT_JSON, SHORT_VERSION,
};
use clap::{Arg, ArgAction, Command as ClapCommand};
use std::collections::HashMap;

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
    commands: Vec<Command>,
    /// The parsed variables (can be redefined)
    variables: Vec<Variable>,
    /// The parsed constants (cannot be redefined)
    constants: Vec<Constant>,
    /// Pre-allocated static strings for default command parameters
    default_param_ids: std::collections::HashMap<String, &'static str>,
}

impl CliGenerator {
    /// Creates a new CLI generator from parsed commands, variables, and constants.
    ///
    /// # Arguments
    ///
    /// * `commands` - The list of commands parsed from the configuration file
    /// * `variables` - The list of variables (can be redefined)
    /// * `constants` - The list of constants (cannot be redefined)
    ///
    /// # Returns
    ///
    /// Returns a new `CliGenerator` instance ready to build CLI interfaces.
    pub fn new(commands: Vec<Command>, variables: Vec<Variable>, constants: Vec<Constant>) -> Self {
        let default_param_ids = Self::preallocate_default_param_ids(&commands);
        Self {
            commands,
            variables,
            constants,
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
    }

    fn add_command_to_clap(&self, mut app: ClapCommand, command: &Command) -> ClapCommand {
        let cmd_name: &'static str = Box::leak(command.name.clone().into_boxed_str());
        let mut subcmd = ClapCommand::new(cmd_name).arg_required_else_help(false);

        subcmd = Self::add_description(subcmd, &command.directives);
        
        // If command has wildcard, add trailing var arg and skip regular parameters
        if command.has_wildcard {
            subcmd = Self::add_wildcard_arg(subcmd);
        } else {
            subcmd = Self::add_parameters(subcmd, &command.parameters, self);
            subcmd = Self::add_default_args_if_needed(subcmd, command, self);
        }

        for child in &command.children {
            subcmd = self.add_command_to_clap(subcmd, child);
        }

        app = app.subcommand(subcmd);
        app
    }
    
    fn add_wildcard_arg(mut subcmd: ClapCommand) -> ClapCommand {
        // Add a trailing var arg that accepts all remaining arguments
        // allow_hyphen_values(true) allows arguments starting with -- or -
        let wildcard_id: &'static str = Box::leak("*".to_string().into_boxed_str());
        let arg = Arg::new(wildcard_id)
            .num_args(1..)
            .trailing_var_arg(true)
            .allow_hyphen_values(true)
            .help("All remaining arguments");
        subcmd = subcmd.arg(arg).allow_missing_positional(true);
        subcmd
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
            (Directive::Before(s), "before") => Some(s.clone()),
            (Directive::After(s), "after") => Some(s.clone()),
            (Directive::Fallback(s), "fallback") => Some(s.clone()),
            (Directive::Validate(s), "validate") => Some(s.clone()),
            _ => None,
        })
    }

    fn get_depends_directive(directives: &[Directive]) -> Vec<super::ast::Dependency> {
        directives.iter()
            .find_map(|d| match d {
                Directive::Depends(deps) => Some(deps.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    fn get_privileged_directive(directives: &[Directive]) -> bool {
        directives.iter().find_map(|d| match d {
            Directive::Privileged(value) => Some(*value),
            _ => None,
        }).unwrap_or(false)
    }

    fn get_validate_directives(directives: &[Directive]) -> Vec<String> {
        directives.iter()
            .filter_map(|d| match d {
                Directive::Validate(s) => Some(s.clone()),
                _ => None,
            })
            .collect()
    }

    /// Validates command parameters according to validation directives.
    ///
    /// Supports format: "param_name matches /regex/"
    ///
    /// # Arguments
    ///
    /// * `validate_directives` - List of validation rules
    /// * `args` - Arguments to validate
    /// * `command_path` - Command path for error messages
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all validations pass,
    /// `Err(message)` if validation fails.
    fn validate_parameters(
        validate_directives: &[String],
        args: &HashMap<String, String>,
        command_path: &[String],
    ) -> Result<(), String> {
        use regex::Regex;
        
        for validate_rule in validate_directives {
            // Parse validation rule: "param_name matches /regex/"
            // Format: <param_name> matches /<pattern>/
            let trimmed = validate_rule.trim();
            
            // Check for "matches" keyword
            if !trimmed.contains("matches") {
                return Err(format!(
                    "Invalid validation rule: '{}'. Expected format: 'param_name matches /regex/'",
                    trimmed
                ));
            }
            
            // Split by "matches"
            let parts: Vec<&str> = trimmed.splitn(2, "matches").collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid validation rule: '{}'. Expected format: 'param_name matches /regex/'",
                    trimmed
                ));
            }
            
            let param_name = parts[0].trim();
            let pattern_part = parts[1].trim();
            
            // Extract regex pattern from /pattern/ or /pattern/flags
            let pattern = if pattern_part.starts_with('/') && pattern_part.len() > 1 {
                // Find closing /
                let mut end_pos = None;
                let mut escaped = false;
                for (i, ch) in pattern_part[1..].char_indices() {
                    if escaped {
                        escaped = false;
                        continue;
                    }
                    if ch == '\\' {
                        escaped = true;
                        continue;
                    }
                    if ch == '/' {
                        end_pos = Some(i + 1);
                        break;
                    }
                }
                
                if let Some(end) = end_pos {
                    // Extract pattern (without leading /)
                    let pattern_str = &pattern_part[1..end - 1];
                    // Unescape the pattern
                    let unescaped = pattern_str.replace("\\/", "/");
                    
                    // Check for flags after closing /
                    let flags = pattern_part[end..].trim();
                    let regex = if flags.is_empty() {
                        Regex::new(&unescaped)
                    } else {
                        // Parse flags (e.g., "i" for case-insensitive)
                        let mut regex_builder = regex::RegexBuilder::new(&unescaped);
                        if flags.contains('i') {
                            regex_builder.case_insensitive(true);
                        }
                        regex_builder.build()
                    };
                    
                    match regex {
                        Ok(re) => re,
                        Err(e) => {
                            return Err(format!(
                                "Invalid regex pattern in validation rule: '{}'. Error: {}",
                                trimmed, e
                            ));
                        }
                    }
                } else {
                    return Err(format!(
                        "Invalid regex pattern in validation rule: '{}'. Expected format: '/pattern/'",
                        trimmed
                    ));
                }
            } else {
                return Err(format!(
                    "Invalid regex pattern in validation rule: '{}'. Expected format: '/pattern/'",
                    trimmed
                ));
            };
            
            // Get parameter value
            let param_value = args.get(param_name)
                .ok_or_else(|| format!(
                    "Validation error: parameter '{}' not found in arguments",
                    param_name
                ))?;
            
            // Validate
            if !pattern.is_match(param_value) {
                let command_str = command_path.join(" ");
                return Err(format!(
                    "❌ Validation error in command 'nest {}':\n   Parameter '{}' with value '{}' does not match pattern '{}'",
                    command_str, param_name, param_value, pattern_part
                ));
            }
        }
        
        Ok(())
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
    fn parse_command_call(line: &str) -> Option<(String, std::collections::HashMap<String, String>)> {
        let trimmed = line.trim();
        
        // Check if line looks like a command call
        // Command calls should start with alphanumeric or underscore, and may contain colons
        // They should not contain shell operators like |, &&, ||, ;, >, <, etc.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        
        // Check for shell operators that indicate this is not a command call
        let shell_operators = ["|", "&&", "||", ";", ">", "<", ">>", "<<", "&", "$", "`"];
        if shell_operators.iter().any(|&op| trimmed.contains(op)) {
            return None;
        }
        
        // Try to parse as command call
        // Pattern: [group:]command[(args)]
        let command_path: String;
        let mut args = std::collections::HashMap::new();
        
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
        let is_valid = command_path.chars().all(|c| {
            c.is_alphanumeric() || c == ':' || c == '_' || c == '-'
        }) && !command_path.starts_with(':') && !command_path.ends_with(':');
        
        if !is_valid {
            return None;
        }
        
        Some((command_path, args))
    }
    
    /// Parses arguments from a command call argument string.
    /// Format: `name="value", name2=true, name3=123`
    fn parse_command_args(args_str: &str) -> Result<std::collections::HashMap<String, String>, ()> {
        let mut args = std::collections::HashMap::new();
        
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
        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            // Remove quotes
            let unquoted = &trimmed[1..trimmed.len() - 1];
            // Unescape quotes
            let unescaped = unquoted
                .replace("\\\"", "\"")
                .replace("\\'", "'")
                .replace("\\\\", "\\");
            unescaped
        }
        // Boolean or number value (keep as is)
        else {
            trimmed.to_string()
        }
    }

    /// Executes a script with the given environment and working directory.
    ///
    /// This function supports both regular shell commands and command calls.
    /// Command calls use the format: `command` or `group:command` or `command(arg="value")`.
    ///
    /// This is a helper function for executing before, after, and fallback scripts.
    fn execute_script(
        &self,
        script: &str,
        env_vars: &HashMap<String, String>,
        cwd: Option<&str>,
        command_path: Option<&[String]>,
        args: &HashMap<String, String>,
        dry_run: bool,
        verbose: bool,
    ) -> Result<(), String> {
        if dry_run {
            use super::output::OutputFormatter;
            OutputFormatter::info(&format!("[DRY RUN] Would execute: {}", script));
            return Ok(());
        }

        // Process script line by line
        let lines: Vec<&str> = script.lines().collect();
        let mut current_shell_block = Vec::new();
        
        for line in lines {
            // Try to parse as command call
            if let Some((cmd_path, cmd_args)) = Self::parse_command_call(line) {
                // Execute any accumulated shell commands first
                if !current_shell_block.is_empty() {
                    let shell_script = current_shell_block.join("\n");
                    Self::execute_shell_script(
                        &shell_script,
                        env_vars,
                        cwd,
                        args,
                        verbose,
                    )?;
                    current_shell_block.clear();
                }
                
                // Resolve command path
                let resolved_path: Vec<String> = if cmd_path.contains(':') {
                    // Absolute path from root (e.g., "dev:build")
                    cmd_path.split(':').map(|s| s.trim().to_string()).collect()
                } else {
                    // Relative path - resolve from current command's parent
                    let cmd_name = cmd_path.trim().to_string();
                    if let Some(current_path) = command_path {
                        if current_path.is_empty() {
                            // Top-level command - dependency is also top-level
                            vec![cmd_name]
                        } else {
                            // Nested command - dependency is relative to parent
                            let mut resolved = current_path[..current_path.len().saturating_sub(1)].to_vec();
                            resolved.push(cmd_name);
                            resolved
                        }
                    } else {
                        vec![cmd_name]
                    }
                };
                
                // Find and execute command
                if let Some(cmd) = self.find_command(&resolved_path) {
                    if verbose {
                        use super::output::OutputFormatter;
                        let args_str = if cmd_args.is_empty() {
                            String::new()
                        } else {
                            let args_display: Vec<String> = cmd_args.iter()
                                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                                .collect();
                            format!("({})", args_display.join(", "))
                        };
                        OutputFormatter::info(&format!(
                            "Executing command: {}{}",
                            resolved_path.join(" "),
                            args_str
                        ));
                    }
                    
                    let mut visited = std::collections::HashSet::new();
                    self.execute_command_with_deps(
                        cmd,
                        &cmd_args,
                        Some(&resolved_path),
                        dry_run,
                        verbose,
                        &mut visited,
                    )?;
                } else {
                    // Command not found - treat as shell command
                    current_shell_block.push(line);
                }
            } else {
                // Not a command call - treat as shell command
                current_shell_block.push(line);
            }
        }
        
        // Execute any remaining shell commands
        if !current_shell_block.is_empty() {
            let shell_script = current_shell_block.join("\n");
            Self::execute_shell_script(
                &shell_script,
                env_vars,
                cwd,
                args,
                verbose,
            )?;
        }

        Ok(())
    }
    
    /// Executes a shell script (helper function).
    fn execute_shell_script(
        script: &str,
        env_vars: &HashMap<String, String>,
        cwd: Option<&str>,
        args: &HashMap<String, String>,
        _verbose: bool,
    ) -> Result<(), String> {
        use std::process::{Command as ProcessCommand, Stdio};
        
        if script.trim().is_empty() {
            return Ok(());
        }

        let mut cmd = ProcessCommand::new("sh");
        cmd.arg("-c");
        cmd.arg(script);

        if let Some(cwd_path) = cwd {
            cmd.current_dir(cwd_path);
        }

        // Set environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Set command arguments as environment variables
        for (key, value) in args {
            cmd.env(key.to_uppercase(), value);
            cmd.env(key, value);
        }

        // Capture output
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let status = cmd
            .status()
            .map_err(|e| format!("Failed to start script execution: {}", e))?;

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            return Err(format!("Script exited with code {}", exit_code));
        }

        Ok(())
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
    /// Executes dependencies before executing the main command.
    ///
    /// # Arguments
    ///
    /// * `depends` - List of dependencies with optional arguments
    /// * `current_path` - Current command path (for resolving relative dependencies)
    /// * `dry_run` - If true, show what would be executed without running it
    /// * `verbose` - If true, show detailed output
    /// * `visited` - Set of already executed commands (for cycle detection)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all dependencies executed successfully,
    /// `Err(message)` if any dependency failed or cycle detected.
    fn execute_dependencies(
        &self,
        depends: &[super::ast::Dependency],
        current_path: &[String],
        dry_run: bool,
        verbose: bool,
        visited: &mut std::collections::HashSet<Vec<String>>,
    ) -> Result<(), String> {
        for dep in depends {
            // Parse dependency path
            // Format: "command" (relative to current command's parent)
            //         "parent:command" (absolute path from root)
            let dep_path: Vec<String> = if dep.command_path.contains(':') {
                // Absolute path from root (e.g., "dev:build")
                dep.command_path.split(':').map(|s| s.trim().to_string()).collect()
            } else {
                // Relative dependency - resolve from current command's parent
                let dep_name = dep.command_path.trim().to_string();
                if current_path.is_empty() {
                    // Top-level command - dependency is also top-level
                    vec![dep_name]
                } else {
                    // Nested command - dependency is relative to parent
                    let mut resolved = current_path[..current_path.len().saturating_sub(1)].to_vec();
                    resolved.push(dep_name);
                    resolved
                }
            };

            // Check for cycles
            if visited.contains(&dep_path) {
                return Err(format!(
                    "Circular dependency detected: {} -> {}",
                    current_path.join(" "),
                    dep_path.join(" ")
                ));
            }

            // Find and execute dependency
            if let Some(dep_command) = self.find_command(&dep_path) {
                // Mark as visited
                visited.insert(dep_path.clone());

                // Execute dependency recursively (with its own dependencies and provided arguments)
                self.execute_command_with_deps(
                    dep_command,
                    &dep.args,
                    Some(&dep_path),
                    dry_run,
                    verbose,
                    visited,
                )?;

                // Remove from visited after execution (allow reuse in different branches)
                visited.remove(&dep_path);
            } else {
                return Err(format!(
                    "Dependency not found: {} (required by {})",
                    dep_path.join(" "),
                    current_path.join(" ")
                ));
            }
        }
        Ok(())
    }

    /// Executes a command with its dependencies.
    ///
    /// This is an internal method that handles dependency resolution.
    fn execute_command_with_deps(
        &self,
        command: &Command,
        args: &HashMap<String, String>,
        command_path: Option<&[String]>,
        dry_run: bool,
        verbose: bool,
        visited: &mut std::collections::HashSet<Vec<String>>,
    ) -> Result<(), String> {
        let command_path = command_path.unwrap_or(&[]);

        // Execute dependencies first
        let depends = Self::get_depends_directive(&command.directives);
        if !depends.is_empty() {
            if verbose {
                use super::output::OutputFormatter;
                let deps_str: Vec<String> = depends.iter().map(|dep| {
                    if dep.args.is_empty() {
                        dep.command_path.clone()
                    } else {
                        let args_str: Vec<String> = dep.args.iter()
                            .map(|(k, v)| format!("{}=\"{}\"", k, v))
                            .collect();
                        format!("{}({})", dep.command_path, args_str.join(", "))
                    }
                }).collect();
                OutputFormatter::info(&format!(
                    "Executing dependencies for {}: {}",
                    command_path.join(" "),
                    deps_str.join(", ")
                ));
            }
            self.execute_dependencies(&depends, command_path, dry_run, verbose, visited)?;
        }

        // Validate parameters (if validation directives are present)
        let validate_directives = Self::get_validate_directives(&command.directives);
        if !validate_directives.is_empty() {
            if let Err(e) = Self::validate_parameters(&validate_directives, args, command_path) {
                return Err(e);
            }
        }

        // Prepare environment
        let env_vars = EnvironmentManager::extract_env_vars(&command.directives);
        let cwd = Self::get_directive_value(&command.directives, "cwd");
        let privileged = Self::get_privileged_directive(&command.directives);

        // Execute before script (if present)
        if let Some(before_script) = Self::get_directive_value(&command.directives, "before") {
            let processed_before = TemplateProcessor::process(
                &before_script,
                args,
                &self.variables,
                &self.constants,
                &command.local_variables,
                &command.local_constants,
            );
            
            if verbose {
                use super::output::OutputFormatter;
                OutputFormatter::info("Executing before script...");
            }
            
            if let Err(e) = self.execute_script(
                &processed_before,
                &env_vars,
                cwd.as_deref(),
                Some(command_path),
                args,
                dry_run,
                verbose,
            ) {
                return Err(format!("Before script failed: {}", e));
            }
        }

        // Execute main script
        let script = Self::get_directive_value(&command.directives, "script")
            .ok_or_else(|| "Command has no script directive".to_string())?;

        let processed_script = TemplateProcessor::process(
            &script,
            args,
            &self.variables,
            &self.constants,
            &command.local_variables,
            &command.local_constants,
        );

        // Check privileged access BEFORE execution
        if privileged && !dry_run {
            use super::executor::CommandExecutor;
            if !CommandExecutor::check_privileged_access() {
                return Err(CommandExecutor::format_privileged_error(command, Some(command_path)));
            }
        }

        // Show dry-run preview
        if dry_run {
            use super::executor::CommandExecutor;
            CommandExecutor::show_dry_run_preview(
                command,
                Some(command_path),
                args,
                &env_vars,
                cwd.as_deref(),
                &processed_script,
                verbose,
                privileged,
            );
            return Ok(());
        }

        // Show verbose information if requested
        if verbose {
            use super::executor::CommandExecutor;
            CommandExecutor::show_verbose_info(
                command,
                Some(command_path),
                args,
                &env_vars,
                cwd.as_deref(),
                &processed_script,
                privileged,
            );
        }

        // Execute script with command call support
        let main_result = self.execute_script(
            &processed_script,
            &env_vars,
            cwd.as_deref(),
            Some(command_path),
            args,
            dry_run,
            verbose,
        );

        // Handle main script result
        match main_result {
            Ok(()) => {
                // Main script succeeded - execute after script (if present)
                if let Some(after_script) = Self::get_directive_value(&command.directives, "after") {
                    let processed_after = TemplateProcessor::process(
                        &after_script,
                        args,
                        &self.variables,
                        &self.constants,
                        &command.local_variables,
                        &command.local_constants,
                    );
                    
                    if verbose {
                        use super::output::OutputFormatter;
                        OutputFormatter::info("Executing after script...");
                    }
                    
                    if let Err(e) = self.execute_script(
                        &processed_after,
                        &env_vars,
                        cwd.as_deref(),
                        Some(command_path),
                        args,
                        dry_run,
                        verbose,
                    ) {
                        return Err(format!("After script failed: {}", e));
                    }
                }
                Ok(())
            }
            Err(_) => {
                // Main script failed - execute fallback script (if present)
                if let Some(fallback_script) = Self::get_directive_value(&command.directives, "fallback") {
                    let processed_fallback = TemplateProcessor::process(
                        &fallback_script,
                        args,
                        &self.variables,
                        &self.constants,
                        &command.local_variables,
                        &command.local_constants,
                    );
                    
                    if verbose {
                        use super::output::OutputFormatter;
                        OutputFormatter::info("Executing fallback script...");
                    }
                    
                    // Execute fallback and return its output instead of error
                    if let Err(e) = self.execute_script(
                        &processed_fallback,
                        &env_vars,
                        cwd.as_deref(),
                        Some(command_path),
                        args,
                        dry_run,
                        verbose,
                    ) {
                        return Err(format!("Fallback script failed: {}", e));
                    }
                    // Fallback succeeded - return Ok (suppress original error)
                    Ok(())
                } else {
                    // No fallback - return original error
                    main_result
                }
            }
        }
    }

    pub fn execute_command(
        &self,
        command: &Command,
        args: &HashMap<String, String>,
        command_path: Option<&[String]>,
        dry_run: bool,
        verbose: bool,
    ) -> Result<(), String> {
        let mut visited = std::collections::HashSet::new();
        self.execute_command_with_deps(command, args, command_path, dry_run, verbose, &mut visited)
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

/// Handles the update command.
///
/// Updates Nest CLI to the latest version by downloading from GitHub releases
/// and replacing the binary in ~/.local/bin.
///
/// # Errors
///
/// Exits with code 1 if:
/// - OS or architecture is not supported
/// - curl or wget is not available
/// - Download fails
/// - Archive extraction fails
/// - Binary replacement fails
pub fn handle_update() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use super::output::colors;
    use super::output::OutputFormatter;

    // Detect OS and architecture
    let (platform, architecture) = match detect_platform() {
        Ok((p, a)) => (p, a),
        Err(e) => {
            OutputFormatter::error(&e);
            std::process::exit(1);
        }
    };

    // Determine binary name
    let binary_name = "nest";
    let install_dir = match env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".local").join("bin"),
        Err(_) => {
            OutputFormatter::error("HOME environment variable is not set");
            std::process::exit(1);
        }
    };
    let binary_path = install_dir.join(binary_name);

    // GitHub repository
    let repo = "quonaro/nest";
    let version = "latest";

    // Print header
    OutputFormatter::info("Updating Nest CLI...");
    println!("  Platform: {}-{}", platform, architecture);
    println!("  Install directory: {}", install_dir.display());

    // Create install directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&install_dir) {
        OutputFormatter::error(&format!("Failed to create install directory: {}", e));
        std::process::exit(1);
    }

    // Build download URL
    let url = if version == "latest" {
        format!(
            "https://github.com/{}/releases/latest/download/nest-{}-{}.tar.gz",
            repo, platform, architecture
        )
    } else {
        format!(
            "https://github.com/{}/releases/download/v{}/nest-{}-{}.tar.gz",
            repo, version, platform, architecture
        )
    };

    // Create temporary directory
    let temp_dir = match env::temp_dir().join(format!("nest-update-{}", std::process::id())) {
        dir => dir,
    };
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        OutputFormatter::error(&format!("Failed to create temporary directory: {}", e));
        std::process::exit(1);
    }
    let temp_file = temp_dir.join(format!("nest-{}-{}.tar.gz", platform, architecture));

    // Download binary
    OutputFormatter::info("Downloading Nest CLI...");
    println!("  URL: {}", url);

    // Convert paths to strings with proper error handling
    let temp_file_str = match temp_file.to_str() {
        Some(s) => s,
        None => {
            OutputFormatter::error("Invalid temporary file path encoding");
            std::process::exit(1);
        }
    };

    let download_success = if Command::new("curl").arg("--version").output().is_ok() {
        // Use curl
        let output = Command::new("curl")
            .args(&[
                "-L",
                "-s",
                "-S",
                "--show-error",
                "-w",
                "%{http_code}",
                "-o",
                temp_file_str,
                &url,
            ])
            .output();

        match output {
            Ok(result) => {
                // HTTP code is in stdout (last line)
                let stdout = String::from_utf8_lossy(&result.stdout);
                let http_code = stdout.trim();

                if http_code == "200" {
                    true
                } else {
                    // Print stderr if available
                    if !result.stderr.is_empty() {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        OutputFormatter::error(&format!(
                            "Failed to download binary (HTTP {}): {}",
                            http_code, stderr
                        ));
                    } else {
                        OutputFormatter::error(&format!(
                            "Failed to download binary (HTTP {})",
                            http_code
                        ));
                    }
                    false
                }
            }
            Err(e) => {
                OutputFormatter::error(&format!("curl failed: {}", e));
                false
            }
        }
    } else if Command::new("wget").arg("--version").output().is_ok() {
        // Use wget
        let output = Command::new("wget")
            .args(&["-O", temp_file_str, &url])
            .output();

        match output {
            Ok(result) if result.status.success() => true,
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                if !stderr.is_empty() {
                    OutputFormatter::error(&format!("wget failed: {}", stderr));
                } else {
                    OutputFormatter::error("wget failed to download file");
                }
                false
            }
            Err(e) => {
                OutputFormatter::error(&format!("wget failed: {}", e));
                false
            }
        }
    } else {
        OutputFormatter::error("Neither curl nor wget is available");
        OutputFormatter::info("Please install curl or wget to use this feature.");
        false
    };

    if !download_success {
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Verify downloaded file exists and is not empty
    match fs::metadata(&temp_file) {
        Ok(meta) if meta.len() > 0 => {}
        Ok(_) => {
            OutputFormatter::error("Downloaded file is empty");
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
        Err(e) => {
            OutputFormatter::error(&format!("Failed to verify downloaded file: {}", e));
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
    }

    // Extract archive
    OutputFormatter::info("Extracting archive...");
    let extract_dir = temp_dir.join("extract");
    if let Err(e) = fs::create_dir_all(&extract_dir) {
        OutputFormatter::error(&format!("Failed to create extract directory: {}", e));
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Convert extract directory path to string with proper error handling
    let extract_dir_str = match extract_dir.to_str() {
        Some(s) => s,
        None => {
            OutputFormatter::error("Invalid extract directory path encoding");
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
    };

    let extract_output = Command::new("tar")
        .args(&[
            "-xzf",
            temp_file_str,
            "-C",
            extract_dir_str,
        ])
        .output();

    match extract_output {
        Ok(result) if result.status.success() => {}
        Ok(_) => {
            OutputFormatter::error("Failed to extract archive");
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
        Err(e) => {
            OutputFormatter::error(&format!("tar failed: {}", e));
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
    }

    // Check if binary exists in extracted archive
    let extracted_binary = extract_dir.join(binary_name);
    if !extracted_binary.exists() {
        OutputFormatter::error(&format!("Binary '{}' not found in archive", binary_name));
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Replace binary using atomic rename to avoid "Text file busy" error
    OutputFormatter::info("Installing binary...");

    // Copy new binary to temporary file in the same directory as target
    // This allows atomic rename operation
    let new_binary_path = binary_path.with_extension("new");
    if let Err(e) = fs::copy(&extracted_binary, &new_binary_path) {
        OutputFormatter::error(&format!("Failed to copy new binary: {}", e));
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Make new binary executable before renaming
    // On Unix systems, set explicit permissions; on Windows, permissions are handled automatically
    #[cfg(unix)]
    {
        let mut perms = match fs::metadata(&new_binary_path) {
            Ok(meta) => meta.permissions(),
            Err(e) => {
                OutputFormatter::error(&format!("Failed to get file permissions: {}", e));
                let _ = fs::remove_dir_all(&temp_dir);
                let _ = fs::remove_file(&new_binary_path);
                std::process::exit(1);
            }
        };
        perms.set_mode(0o755);
        if let Err(e) = fs::set_permissions(&new_binary_path, perms) {
            OutputFormatter::error(&format!("Failed to set executable permissions: {}", e));
            let _ = fs::remove_dir_all(&temp_dir);
            let _ = fs::remove_file(&new_binary_path);
            std::process::exit(1);
        }
    }

    // Try to remove old binary first (if it exists and is not in use)
    // This is best-effort - if it fails due to "Text file busy", we'll try rename anyway
    if binary_path.exists() {
        let _ = fs::remove_file(&binary_path);
    }

    // Atomically replace old binary with new one using rename
    // This should work even if the old binary is in use, as rename is atomic
    if let Err(e) = fs::rename(&new_binary_path, &binary_path) {
        // If rename fails, try to restore the new binary and give helpful error
        let error_msg = format!("Failed to install binary: {}", e);

        // Check if it's the "Text file busy" error
        if error_msg.contains("Text file busy") || error_msg.contains("os error 26") {
            OutputFormatter::error("Cannot update binary while it is running.");
            OutputFormatter::info(
                "Please close this terminal session and run the update command again.",
            );
            OutputFormatter::info("Alternatively, you can manually replace the binary:");
            println!(
                "  {}mv{} {} {}",
                OutputFormatter::help_label("mv"),
                colors::RESET,
                OutputFormatter::path(&new_binary_path.display().to_string()),
                OutputFormatter::path(&binary_path.display().to_string())
            );
        } else {
            OutputFormatter::error(&error_msg);
        }

        let _ = fs::remove_dir_all(&temp_dir);
        // Don't remove new_binary_path - user might want to manually install it
        std::process::exit(1);
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    // Success message
    OutputFormatter::success("Nest CLI updated successfully!");
    println!(
        "  {}Location:{} {}",
        OutputFormatter::help_label("Location:"),
        colors::RESET,
        OutputFormatter::path(&binary_path.display().to_string())
    );
    println!(
        "  Run {}nest --version{} to verify the update.",
        colors::BRIGHT_BLUE,
        colors::RESET
    );
}

/// Detects the platform and architecture.
///
/// # Returns
///
/// Returns `Ok((platform, architecture))` if detection succeeds,
/// or `Err(error_message)` if the OS or architecture is not supported.
///
/// # Platform Support
///
/// This function currently supports Linux and macOS. On Windows, use the PowerShell install script.
fn detect_platform() -> Result<(String, String), String> {
    use std::process::Command;

    // Check if uname is available (Unix systems)
    let os_output = match Command::new("uname").arg("-s").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => {
            // On Windows or if uname is not available
            #[cfg(windows)]
            return Err("Update command is not supported on Windows. Please use the PowerShell install script (install.ps1) instead.".to_string());

            #[cfg(not(windows))]
            return Err("Failed to detect OS. The 'uname' command is required.".to_string());
        }
    };

    let platform = match os_output.as_str() {
        "Linux" => "linux",
        "Darwin" => "macos",
        _ => {
            return Err(format!(
                "Unsupported OS: {}. Update command currently supports Linux and macOS only.",
                os_output
            ))
        }
    };

    // Detect architecture
    let arch_output = Command::new("uname").arg("-m").output();
    let arch = match arch_output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => return Err("Failed to detect architecture".to_string()),
    };

    let architecture = match arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" | "arm64" => "aarch64",
        _ => return Err(format!("Unsupported architecture: {}", arch)),
    };

    Ok((platform.to_string(), architecture.to_string()))
}
