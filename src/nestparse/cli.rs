use super::ast::{Command, Directive, Parameter, Value};
use super::env::EnvironmentManager;
use super::executor::CommandExecutor;
use super::template::TemplateProcessor;
use clap::{Arg, ArgAction, Command as ClapCommand};
use std::collections::HashMap;

pub struct CliGenerator {
    commands: Vec<Command>,
    default_param_ids: std::collections::HashMap<String, &'static str>,
}

impl CliGenerator {
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
            if let Some(default_cmd) = command.children.iter().find(|c| c.name == "default") {
                for param in &default_cmd.parameters {
                    let static_str: &'static str =
                        Box::leak(param.name.clone().into_boxed_str());
                    ids.insert(param.name.clone(), static_str);
                }
            }
        }

        ids
    }

    pub fn get_param_id(&self, param_name: &str) -> &'static str {
        self.default_param_ids
            .get(param_name)
            .copied()
            .unwrap_or_else(|| {
                Box::leak(param_name.to_string().into_boxed_str())
            })
    }

    pub fn build_cli(&self) -> ClapCommand {
        let mut app = Self::create_base_cli();

        for command in &self.commands {
            app = self.add_command_to_clap(app, command);
        }

        app
    }

    fn create_base_cli() -> ClapCommand {
        ClapCommand::new("nest")
            .about("Nest task runner")
            .arg(
                Arg::new("version")
                    .long("version")
                    .short('V')
                    .action(ArgAction::SetTrue)
                    .hide(true)
                    .help("Print version information"),
            )
            .arg(
                Arg::new("show")
                    .long("show")
                    .value_name("FORMAT")
                    .value_parser(["json", "ast"])
                    .hide(true)
                    .help("Show commands in different formats (json, ast)"),
            )
    }

    fn add_command_to_clap(&self, mut app: ClapCommand, command: &Command) -> ClapCommand {
        let cmd_name: &'static str = Box::leak(command.name.clone().into_boxed_str());
        let mut subcmd = ClapCommand::new(cmd_name);

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
        for param in parameters {
            let arg = generator.parameter_to_arg(param);
            subcmd = subcmd.arg(arg);
        }
        subcmd
    }

    fn add_default_args_if_needed(
        mut subcmd: ClapCommand,
        command: &Command,
        generator: &CliGenerator,
    ) -> ClapCommand {
        if !command.children.is_empty() {
            if let Some(default_cmd) = command.children.iter().find(|c| c.name == "default") {
                for param in &default_cmd.parameters {
                    let param_id = generator.get_param_id(&param.name);
                    let arg = generator.parameter_to_arg_with_id(param, param_id);
                    subcmd = subcmd.arg(arg);
                }
            }
        }
        subcmd
    }

    fn parameter_to_arg(&self, param: &Parameter) -> Arg {
        let param_name: &'static str = Box::leak(param.name.clone().into_boxed_str());
        self.parameter_to_arg_with_id(param, param_name)
    }

    fn parameter_to_arg_with_id(&self, param: &Parameter, param_id: &'static str) -> Arg {
        let mut arg = Arg::new(param_id);

        match param.param_type.as_str() {
            "bool" => Self::build_bool_flag(&mut arg, param, param_id),
            _ => Self::build_value_arg(&mut arg, param, param_id),
        }

        arg
    }

    fn build_bool_flag(arg: &mut Arg, param: &Parameter, param_id: &'static str) {
        // Allow boolean flags to accept true/false values or be used as a flag (defaults to true)
        let mut new_arg = arg
            .clone()
            .long(param_id)
            .action(ArgAction::Set)
            .value_parser(["true", "false"])
            .num_args(0..=1)
            .help(format!("Flag: {} (true/false, or use without value for true)", param.name));

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
            Self::build_required_arg(arg, param);
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

    fn build_required_arg(arg: &mut Arg, param: &Parameter) {
        let help_text = format!("Required parameter: {} ({})", param.name, param.param_type);
        *arg = arg
            .clone()
            .help(help_text)
            .required(true)
            .action(ArgAction::Set);
    }

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
        command.children.iter().any(|c| c.name == "default")
    }

    pub fn execute_command(
        &self,
        command: &Command,
        args: &HashMap<String, String>,
        command_path: Option<&[String]>,
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
        )
    }
}

pub fn handle_version() {
    println!("nest {}", env!("CARGO_PKG_VERSION"));
    std::process::exit(0);
}

pub fn handle_json(commands: &[Command]) -> Result<(), Box<dyn std::error::Error>> {
    use super::json::to_json;
    let json = to_json(commands)?;
    println!("{}", json);
    Ok(())
}

pub fn handle_show_ast(commands: &[Command]) {
    use super::display::print_command;
    println!("🌳 AST Structure:\n");
    for command in commands {
        print_command(command, 0);
        println!();
    }
}
