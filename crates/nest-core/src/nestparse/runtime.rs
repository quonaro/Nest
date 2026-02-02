//! Runtime for executing Nest commands.
//!
//! This module handles the execution phase of the CLI, separating it from
//! the build/generation phase.

use super::ast::{Command, Constant, Function, Variable};
use super::directives::DirectiveResolver;
use super::env::EnvironmentManager;
use super::runtime_validator::RuntimeValidator;
use super::template::{FunctionResolver, TemplateContext, TemplateProcessor};
use crate::constants::{DEFAULT_SUBCOMMAND, ENV_NEST_CALL_STACK};

use std::collections::HashMap;

/// Context for script execution within the Runtime.
pub struct ScriptExecutionContext<'a> {
    pub env_vars: &'a HashMap<String, String>,
    pub cwd: Option<&'a str>,
    pub command_path: Option<&'a [String]>,
    pub args: &'a HashMap<String, String>,
    pub dry_run: bool,
    pub verbose: bool,
    pub parent_args: &'a HashMap<String, String>,
    pub hide_output: bool,
    pub privileged: bool,
    pub pid_callback: Option<&'a (dyn Fn(u32) + Send + Sync)>,
}

/// Context for command execution containing related parameters.
pub struct CommandExecutionContext<'a> {
    pub command: &'a Command,
    pub args: &'a HashMap<String, String>,
    pub command_path: Option<&'a [String]>,
    pub dry_run: bool,
    pub verbose: bool,
    pub visited: &'a mut std::collections::HashSet<Vec<String>>,
    pub parent_args: &'a HashMap<String, String>,
}

pub struct Runtime {
    /// The parsed commands from the configuration file
    commands: Vec<Command>,
    /// The parsed variables (can be redefined)
    variables: Vec<Variable>,
    /// The parsed constants (cannot be redefined)
    constants: Vec<Constant>,
    /// The parsed functions (reusable scripts)
    functions: Vec<Function>,
    /// Callback for reporting child process PIDs (for signal handling)
    pid_callback: Option<Box<dyn Fn(u32) + Send + Sync>>,
}

/// Internal helper for resolving function calls during template processing.
struct RuntimeFunctionResolver<'a> {
    runtime: &'a Runtime,
    context: &'a ScriptExecutionContext<'a>,
}

impl<'a> FunctionResolver for RuntimeFunctionResolver<'a> {
    fn resolve(
        &self,
        func_name: &str,
        args: &HashMap<String, String>,
    ) -> Result<Option<String>, String> {
        // Resolve function from the runtime
        if let Some(func) = self.runtime.find_function(func_name) {
            // Functions called from templates use the current context's environment
            // but their own arguments.
            let mut merged_env = self.context.env_vars.clone();
            use std::env;
            for (key, value) in env::vars() {
                merged_env.insert(key, value);
            }

            let func_context = ScriptExecutionContext {
                args,
                env_vars: &merged_env,
                ..*self.context
            };

            self.runtime.execute_function(func, &func_context)
        } else {
            // Function not found - returning Ok(None) allows TemplateProcessor
            // to keep the original {{ func() }} tag if it wasn't a valid function call.
            Ok(None)
        }
    }
}

impl Runtime {
    pub fn new(
        commands: Vec<Command>,
        variables: Vec<Variable>,
        constants: Vec<Constant>,
        functions: Vec<Function>,
        pid_callback: Option<Box<dyn Fn(u32) + Send + Sync>>,
    ) -> Self {
        Self {
            commands,
            variables,
            constants,
            functions,
            pid_callback,
        }
    }

    // / Checks if directives are valid
    // removed directive getters in favor of DirectiveResolver

    // Insert execute_script, execute_command, etc. next
}
impl Runtime {
    // / Validates command parameters according to validation directives.
    // Logic moved to RuntimeValidator

    /// Executes a script with the given environment and working directory.
    ///
    /// This function supports both regular shell commands and command calls.
    /// Command calls use the format: `command` or `group:command` or `command(arg="value")`.
    ///
    /// This is a helper function for executing before, after, and fallback scripts.
    fn execute_script(&self, script: &str, context: &ScriptExecutionContext) -> Result<(), String> {
        if context.dry_run {
            use super::output::OutputFormatter;
            OutputFormatter::info(&format!("[DRY RUN] Would execute: {}", script));
            return Ok(());
        }

        // Pre-process script to handle function calls in templates {{ func() }}
        let script = self.process_function_calls_in_templates(script, context)?;

        // Process script line by line
        let lines: Vec<&str> = script.lines().collect();
        let mut current_shell_block: Vec<String> = Vec::new();

        for line in lines {
            let trimmed_line = line.trim();

            // Preserve empty lines in shell blocks (they might be needed for syntax)
            if trimmed_line.is_empty() {
                current_shell_block.push(line.to_string());
                continue;
            }

            // Check for shell: prefix (explicit external command call)
            if trimmed_line.starts_with("shell:") {
                // Execute any accumulated shell commands first
                if !current_shell_block.is_empty() {
                    let shell_script = current_shell_block.join("\n");
                    Self::execute_shell_script(&shell_script, context)?;
                    current_shell_block.clear();
                }

                // Remove "shell:" prefix and process template variables (e.g., $*)
                let external_command = trimmed_line
                    .strip_prefix("shell:")
                    .unwrap_or(trimmed_line)
                    .trim_start();
                if !external_command.is_empty() {
                    // Process template variables in the command (e.g., $* -> arguments) and execute
                    let tpl_context = TemplateContext {
                        global_variables: &self.variables,
                        global_constants: &self.constants,
                        local_variables: &[],
                        local_constants: &[],
                        parent_variables: &[],
                        parent_constants: &[],
                    };
                    let processed_command = TemplateProcessor::process(
                        external_command,
                        context.args,
                        &tpl_context,
                        context.parent_args,
                    );
                    Self::execute_shell_script(&processed_command, context)?;
                }
                continue;
            }

            // Try to parse as command or function call
            if let Some((call_name, call_args)) =
                super::executor::CommandExecutor::parse_command_call(trimmed_line)
            {
                // Execute any accumulated shell commands first
                if !current_shell_block.is_empty() {
                    let shell_script = current_shell_block.join("\n");
                    Self::execute_shell_script(&shell_script, context)?;
                    current_shell_block.clear();
                }

                // Check if it's a function call (single name, no colons)
                if !call_name.contains(':') {
                    if let Some(func) = self.find_function(&call_name) {
                        // Merge global env_vars with system env
                        let mut merged_env = context.env_vars.clone();
                        use std::env;
                        for (key, value) in env::vars() {
                            merged_env.insert(key, value);
                        }

                        // Execute function and capture return value (if any)
                        let func_context = ScriptExecutionContext {
                            args: &call_args,
                            env_vars: &merged_env,
                            ..*context
                        };
                        let _return_value = self.execute_function(func, &func_context)?;
                        // Note: return value is currently ignored when calling from script
                        // To use return value, use {{ func() }} syntax in templates
                        continue;
                    }
                }

                // Try to find as command
                let resolved_path: Vec<String> = if call_name.contains(':') {
                    // Absolute path from root (e.g., "dev:build")
                    call_name.split(':').map(|s| s.trim().to_string()).collect()
                } else {
                    // Relative path - resolve from current command's parent
                    let cmd_name = call_name.trim().to_string();
                    if let Some(current_path) = context.command_path {
                        if current_path.is_empty() {
                            // Top-level command - dependency is also top-level
                            vec![cmd_name]
                        } else {
                            // Nested command - dependency is relative to parent
                            let mut resolved =
                                current_path[..current_path.len().saturating_sub(1)].to_vec();
                            resolved.push(cmd_name);
                            resolved
                        }
                    } else {
                        vec![cmd_name]
                    }
                };

                // Check if resolved path matches current command (recursive call)
                if let Some(current_path) = context.command_path {
                    if resolved_path == current_path {
                        // This is a recursive call - treat as shell command instead
                        use super::output::colors;
                        use super::output::OutputFormatter;
                        let path_str = current_path.join(" ");
                        let full_path_str = resolved_path.join(":");
                        OutputFormatter::warning(&format!(
                            "{}⚠️  Warning:{} Command '{}' in script would call itself recursively.\n\
                             {}   Treating '{}' as external shell command instead.\n\
                             {}   To explicitly call this nest command, use: {}{}{}\n\
                             {}   To explicitly call external command, use: {}shell:{}{}",
                            colors::YELLOW, colors::RESET,
                            path_str,
                            colors::GRAY, call_name,
                            colors::GRAY, colors::BRIGHT_CYAN, full_path_str, colors::RESET,
                            colors::GRAY, colors::BRIGHT_CYAN, call_name, colors::RESET
                        ));
                        // Build command with arguments for shell execution
                        // For wildcard commands, arguments are in args["*"]
                        let mut shell_cmd = call_name.clone();

                        // Add wildcard arguments if available (for wildcard commands)
                        if let Some(wildcard_args) = context.args.get("*") {
                            if !wildcard_args.is_empty() {
                                shell_cmd.push(' ');
                                shell_cmd.push_str(wildcard_args);
                            }
                        } else if !call_args.is_empty() {
                            // For regular commands with arguments, add them
                            let args_str: Vec<String> = call_args
                                .iter()
                                .map(|(k, v)| {
                                    // Quote values that contain spaces
                                    if v.contains(' ') {
                                        format!("{}=\"{}\"", k, v)
                                    } else {
                                        format!("{}={}", k, v)
                                    }
                                })
                                .collect();
                            shell_cmd.push(' ');
                            shell_cmd.push_str(&args_str.join(" "));
                        }

                        // Process template variables (e.g., $*) before treating as shell command
                        let tpl_context = TemplateContext {
                            global_variables: &self.variables,
                            global_constants: &self.constants,
                            local_variables: &[],
                            local_constants: &[],
                            parent_variables: &[],
                            parent_constants: &[],
                        };
                        let processed_line = TemplateProcessor::process(
                            &shell_cmd,
                            context.args,
                            &tpl_context,
                            &HashMap::new(),
                        );
                        current_shell_block.push(processed_line);
                        continue;
                    }
                }

                if let Some(cmd) = self.find_command(&resolved_path) {
                    let mut visited = std::collections::HashSet::new();
                    // Commands called from scripts don't inherit parent args
                    let empty_parent_args: HashMap<String, String> = HashMap::new();
                    let mut cmd_context = CommandExecutionContext {
                        command: cmd,
                        args: &call_args,
                        command_path: Some(&resolved_path),
                        dry_run: context.dry_run,
                        verbose: context.verbose,
                        visited: &mut visited,
                        parent_args: &empty_parent_args,
                    };
                    self.execute_command_with_deps(&mut cmd_context)?;
                } else {
                    // Neither command nor function found - treat as shell command
                    current_shell_block.push(line.to_string());
                }
            } else {
                // Not a command/function call - treat as shell command
                current_shell_block.push(line.to_string());
            }
        }

        // Execute any remaining shell commands
        if !current_shell_block.is_empty() {
            let shell_script = current_shell_block.join("\n");
            // Don't trim - preserve all whitespace and structure
            if !shell_script.trim().is_empty() {
                Self::execute_shell_script(&shell_script, context)?;
            }
        }

        Ok(())
    }

    /// Executes a shell script (helper function).
    fn execute_shell_script(script: &str, context: &ScriptExecutionContext) -> Result<(), String> {
        use super::ast::Command;
        use super::executor::{CommandExecutor, ExecutionContext};

        let exec_context = ExecutionContext {
            command: &Command::default(), // Placeholder if not available
            args: context.args,
            env_vars: context.env_vars,
            cwd: context.cwd,
            command_path: context.command_path,
            dry_run: context.dry_run,
            verbose: context.verbose,
            privileged: context.privileged,
            pid_callback: context.pid_callback.map(|cb| cb as &dyn Fn(u32)),
            hide_output: context.hide_output,
        };

        CommandExecutor::execute(script, &exec_context)
    }

    /// Executes a shell script with specified shell (helper function).
    /// Processes template variables in @return value.
    ///
    /// Replaces {{variable}} and {{param}} placeholders with their actual values.
    ///
    /// # Arguments
    ///
    /// * `return_expr` - The return expression (value after @return)
    /// * `args` - Function arguments
    /// * `var_map` - Variable map (local and global variables)
    ///
    /// # Returns
    ///
    /// Returns the processed return value with all placeholders replaced.
    fn process_template_in_return_value(
        return_expr: &str,
        args: &HashMap<String, String>,
        var_map: &HashMap<String, String>,
    ) -> String {
        let mut result = return_expr.to_string();

        // Replace parameter placeholders {{param}}
        for (key, value) in args {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Replace variable placeholders {{VAR}}
        for (key, value) in var_map {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Remove surrounding quotes if present (for string literals)
        let trimmed = result.trim();
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            result = trimmed[1..trimmed.len() - 1].to_string();
        }

        result
    }

    /// Processes function calls in templates like {{ func() }} or {{ func(arg="value") }}.
    ///
    /// Finds all function calls in double curly braces and replaces them with their return values.
    ///
    /// # Arguments
    ///
    /// * `script` - The script containing template function calls
    /// * `context` - Script execution context
    ///
    /// # Returns
    ///
    /// Returns the script with function calls replaced by their return values.
    fn process_function_calls_in_templates(
        &self,
        script: &str,
        context: &ScriptExecutionContext,
    ) -> Result<String, String> {
        let resolver = RuntimeFunctionResolver {
            runtime: self,
            context,
        };
        TemplateProcessor::process_function_calls(script, &resolver)
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

    /// and constants from each parent command. The order is from root to leaf,
    /// so variables from closer parents can override variables from farther parents.
    fn collect_parent_variables(
        &self,
        command_path: &[String],
    ) -> (Vec<super::ast::Variable>, Vec<super::ast::Constant>) {
        let mut parent_variables = Vec::new();
        let mut parent_constants = Vec::new();

        // If path is empty or has only one element, no parents
        if command_path.len() <= 1 {
            return (parent_variables, parent_constants);
        }

        // Traverse path from root to parent (excluding the last element which is the current command)
        let mut current = &self.commands;
        for name in command_path.iter().take(command_path.len() - 1) {
            if let Some(cmd) = current.iter().find(|c| &c.name == name) {
                // Add variables and constants from this parent command
                parent_variables.extend(cmd.local_variables.iter().cloned());
                parent_constants.extend(cmd.local_constants.iter().cloned());
                current = &cmd.children;
            } else {
                break;
            }
        }

        (parent_variables, parent_constants)
    }

    /// Collects directives (CWD, AFTER, BEFORE, FALLBACK) from all parent commands in the path.
    fn collect_parent_directives(
        &self,
        command_path: &[String],
    ) -> std::collections::HashMap<String, (String, bool)> {
        let mut parent_directives = std::collections::HashMap::new();

        // If path is empty or has only one element, no parents
        if command_path.len() <= 1 {
            return parent_directives;
        }

        // Traverse path from root to parent (excluding the last element which is the current command)
        let mut current = &self.commands;
        for name in command_path.iter().take(command_path.len() - 1) {
            if let Some(cmd) = current.iter().find(|c| &c.name == name) {
                // Collect inheritable directives from this parent command
                if let Some(cwd) = DirectiveResolver::get_directive_value(&cmd.directives, "cwd") {
                    parent_directives.insert("cwd".to_string(), (cwd, false));
                }
                if let Some((after, hide_after)) =
                    DirectiveResolver::get_directive_value_with_hide(&cmd.directives, "after")
                {
                    parent_directives.insert("after".to_string(), (after, hide_after));
                }
                if let Some((before, hide_before)) =
                    DirectiveResolver::get_directive_value_with_hide(&cmd.directives, "before")
                {
                    parent_directives.insert("before".to_string(), (before, hide_before));
                }
                if let Some((fallback, hide_fallback)) =
                    DirectiveResolver::get_directive_value_with_hide(&cmd.directives, "fallback")
                {
                    parent_directives.insert("fallback".to_string(), (fallback, hide_fallback));
                }
                if let Some((finally, hide_finally)) =
                    DirectiveResolver::get_directive_value_with_hide(&cmd.directives, "finally")
                {
                    parent_directives.insert("finally".to_string(), (finally, hide_finally));
                }
                current = &cmd.children;
            } else {
                break;
            }
        }

        parent_directives
    }

    /// Collects ENV directives from all parent commands in the path.
    fn collect_parent_env_directives(&self, command_path: &[String]) -> Vec<super::ast::Directive> {
        let mut parent_env_directives = Vec::new();

        // If path is empty or has only one element, no parents
        if command_path.len() <= 1 {
            return parent_env_directives;
        }

        // Traverse path from root to parent (excluding the last element which is the current command)
        let mut current = &self.commands;
        for name in command_path.iter().take(command_path.len() - 1) {
            if let Some(cmd) = current.iter().find(|c| &c.name == name) {
                // Collect all ENV directives from this parent command
                for directive in &cmd.directives {
                    match directive {
                        super::ast::Directive::Env(..) | super::ast::Directive::EnvFile(..) => {
                            parent_env_directives.push(directive.clone());
                        }
                        _ => {}
                    }
                }
                current = &cmd.children;
            } else {
                break;
            }
        }

        parent_env_directives
    }

    /// Finds a function by its name.
    fn find_function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Checks if a command has a default subcommand.
    pub fn has_default_command(&self, command: &Command) -> bool {
        command
            .children
            .iter()
            .any(|c| c.name == DEFAULT_SUBCOMMAND)
    }
}
impl Runtime {
    /// Executes a function with the provided arguments.
    fn execute_function(
        &self,
        function: &Function,
        context: &ScriptExecutionContext,
    ) -> Result<Option<String>, String> {
        if context.verbose {
            use super::output::OutputFormatter;
            let args_str = if context.args.is_empty() {
                String::new()
            } else {
                let args_display: Vec<String> = context
                    .args
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("({})", args_display.join(", "))
            };
            OutputFormatter::info(&format!(
                "Executing function: {}{}",
                function.name, args_str
            ));
        }

        // Build variable map: function local > global
        let mut var_map: HashMap<String, String> = HashMap::new();

        // Add global constants
        for constant in &self.constants {
            var_map.insert(constant.name.clone(), constant.value.to_string_unquoted());
        }

        // Add global variables
        for variable in &self.variables {
            var_map.insert(variable.name.clone(), variable.value.to_string_unquoted());
        }

        // Add function local variables (override global)
        for variable in &function.local_variables {
            var_map.insert(variable.name.clone(), variable.value.to_string_unquoted());
        }

        // Process function body with template substitution
        let processed_body = {
            let mut body = function.body.clone();

            // Replace parameter placeholders {{param}}
            for (key, value) in context.args {
                let placeholder = format!("{{{{{}}}}}", key);
                body = body.replace(&placeholder, value);
            }

            // Replace variable and constant placeholders {{VAR}}
            for (key, value) in &var_map {
                let placeholder = format!("{{{{{}}}}}", key);
                body = body.replace(&placeholder, value);
            }

            // Replace special variables
            use crate::constants::{
                DEFAULT_USER, ENV_VAR_USER, TEMPLATE_VAR_NOW, TEMPLATE_VAR_USER,
            };
            use chrono::Utc;
            use std::env;
            body = body.replace(TEMPLATE_VAR_NOW, &Utc::now().to_rfc3339());
            body = body.replace(
                TEMPLATE_VAR_USER,
                &env::var(ENV_VAR_USER).unwrap_or_else(|_| DEFAULT_USER.to_string()),
            );

            body
        };

        // Execute function body line by line to support @return directive
        // Functions don't inherit parent args - they use their own args
        // Functions inherit hide_output from the calling script
        let lines: Vec<&str> = processed_body.lines().collect();
        let mut current_shell_block: Vec<String> = Vec::new();

        for line in lines {
            let trimmed_line = line.trim();

            // Check for @return directive
            if trimmed_line.starts_with("@return") {
                // Execute any accumulated shell commands first
                if !current_shell_block.is_empty() {
                    let shell_script = current_shell_block.join("\n");
                    Self::execute_shell_script(&shell_script, context)?;
                    current_shell_block.clear();
                }

                // Extract return value
                let return_value = if let Some(return_expr) = trimmed_line.strip_prefix("return ") {
                    Self::process_template_in_return_value(
                        return_expr.trim_start(),
                        context.args,
                        &var_map,
                    )
                } else {
                    String::new()
                };

                return Ok(Some(return_value));
            }

            // Preserve empty lines in shell blocks
            if trimmed_line.is_empty() {
                current_shell_block.push(line.to_string());
                continue;
            }

            // Check for shell: prefix (explicit external command call)
            if trimmed_line.starts_with("shell:") {
                // Execute any accumulated shell commands first
                if !current_shell_block.is_empty() {
                    let shell_script = current_shell_block.join("\n");
                    Self::execute_shell_script(&shell_script, context)?;
                    current_shell_block.clear();
                }

                // Remove "shell:" prefix and process template variables
                let external_command = trimmed_line
                    .strip_prefix("shell:")
                    .unwrap_or(trimmed_line)
                    .trim_start();
                if !external_command.is_empty() {
                    let tpl_context = TemplateContext {
                        global_variables: &self.variables,
                        global_constants: &self.constants,
                        local_variables: &[],
                        local_constants: &[],
                        parent_variables: &[],
                        parent_constants: &[],
                    };
                    let processed_command = TemplateProcessor::process(
                        external_command,
                        context.args,
                        &tpl_context,
                        context.parent_args,
                    );
                    Self::execute_shell_script(&processed_command, context)?;
                }
                continue;
            }

            // Try to parse as command or function call
            if let Some((call_name, call_args)) =
                super::executor::CommandExecutor::parse_command_call(trimmed_line)
            {
                // Execute any accumulated shell commands first
                if !current_shell_block.is_empty() {
                    let shell_script = current_shell_block.join("\n");
                    Self::execute_shell_script(&shell_script, context)?;
                    current_shell_block.clear();
                }

                // Check if it's a function call (single name, no colons)
                if !call_name.contains(':') {
                    if let Some(func) = self.find_function(&call_name) {
                        // Merge global env_vars with system env
                        let mut merged_env = context.env_vars.clone();
                        use std::env;
                        for (key, value) in env::vars() {
                            merged_env.insert(key, value);
                        }

                        // Execute function and capture return value (if any)
                        let _return_value = self.execute_function(func, context)?;
                        // Note: return value is ignored here - functions called from within functions
                        // don't automatically propagate their return values unless explicitly handled
                        continue;
                    }
                }

                // Try to find as command
                let resolved_path: Vec<String> = if call_name.contains(':') {
                    call_name.split(':').map(|s| s.trim().to_string()).collect()
                } else {
                    let cmd_name = call_name.trim().to_string();
                    if let Some(current_path) = context.command_path {
                        if current_path.is_empty() {
                            vec![cmd_name]
                        } else {
                            let mut resolved =
                                current_path[..current_path.len().saturating_sub(1)].to_vec();
                            resolved.push(cmd_name);
                            resolved
                        }
                    } else {
                        vec![cmd_name]
                    }
                };

                // Execute command (recursive call detection handled in execute_command)
                if let Some(cmd) = self.find_command(&resolved_path) {
                    self.execute_command(
                        cmd,
                        &call_args,
                        Some(&resolved_path),
                        context.dry_run,
                        context.verbose,
                    )?;
                } else {
                    // Not a command, treat as shell command
                    current_shell_block.push(line.to_string());
                }
            } else {
                // Regular shell command line
                current_shell_block.push(line.to_string());
            }
        }

        // Execute any remaining shell commands
        if !current_shell_block.is_empty() {
            let shell_script = current_shell_block.join("\n");
            Self::execute_shell_script(&shell_script, context)?;
        }

        // Function completed without @return - return None
        Ok(None)
    }

    /// Executes dependencies before executing the main command.
    fn execute_dependencies(
        &self,
        depends: &[super::ast::Dependency],
        context: &mut CommandExecutionContext<'_>,
        parallel: bool,
    ) -> Result<(), String> {
        let current_path = context.command_path.unwrap_or(&[]);
        let dry_run = context.dry_run;
        let verbose = context.verbose;
        let visited = &mut *context.visited;
        let parent_args = context.parent_args;
        // Resolve all dependency paths first
        let mut tasks = Vec::new();
        for dep in depends {
            // Parse dependency path
            // Format: "command" (relative to current command's parent)
            //         "parent:command" (absolute path from root)
            let dep_path: Vec<String> = if dep.command_path.contains(':') {
                // Absolute path from root (e.g., "dev:build")
                dep.command_path
                    .split(':')
                    .map(|s| s.trim().to_string())
                    .collect()
            } else {
                // Relative dependency - resolve from current command's parent
                let dep_name = dep.command_path.trim().to_string();
                if current_path.is_empty() {
                    // Top-level command - dependency is also top-level
                    vec![dep_name]
                } else {
                    // Nested command - dependency is relative to parent
                    let mut resolved =
                        current_path[..current_path.len().saturating_sub(1)].to_vec();
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

            tasks.push((dep, dep_path));
        }

        if parallel {
            use std::sync::{Arc, Mutex};
            use std::thread;

            let errors = Arc::new(Mutex::new(Vec::new()));

            thread::scope(|s| {
                for (dep, dep_path) in tasks {
                    let mut thread_visited = visited.clone();
                    let errors_clone = Arc::clone(&errors);
                    let parent_args_clone = parent_args.clone(); // Clone for thread ownership

                    s.spawn(move || {
                        if let Some(dep_command) = self.find_command(&dep_path) {
                            let mut dep_context = CommandExecutionContext {
                                command: dep_command,
                                args: &dep.args,
                                command_path: Some(&dep_path),
                                dry_run,
                                verbose,
                                visited: &mut thread_visited,
                                parent_args: &parent_args_clone,
                            };
                            if let Err(e) = self.execute_command_with_deps(&mut dep_context) {
                                let mut errs = errors_clone.lock().unwrap();
                                errs.push(format!(
                                    "Dependency '{}' failed: {}",
                                    dep.command_path, e
                                ));
                            }
                        } else {
                            let mut errs = errors_clone.lock().unwrap();
                            errs.push(format!(
                                "Dependency not found: {} (required by {})",
                                dep_path.join(" "),
                                current_path.join(" ")
                            ));
                        }
                    });
                }
            });

            let errors = errors.lock().unwrap();
            if !errors.is_empty() {
                return Err(errors.join("\n"));
            }
        } else {
            // Serial execution
            for (dep, dep_path) in tasks {
                if let Some(dep_command) = self.find_command(&dep_path) {
                    let mut dep_context = CommandExecutionContext {
                        command: dep_command,
                        args: &dep.args,
                        command_path: Some(&dep_path),
                        dry_run,
                        verbose,
                        visited,
                        parent_args,
                    };
                    self.execute_command_with_deps(&mut dep_context)?;
                } else {
                    return Err(format!(
                        "Dependency not found: {} (required by {})",
                        dep_path.join(" "),
                        current_path.join(" ")
                    ));
                }
            }
        }

        Ok(())
    }

    /// Executes a command with its dependencies.
    fn execute_command_with_deps(
        &self,
        context: &mut CommandExecutionContext<'_>,
    ) -> Result<(), String> {
        let command = context.command;
        let args = context.args;
        let command_path = context.command_path;
        let dry_run = context.dry_run;
        let verbose = context.verbose;
        let parent_args = context.parent_args;
        let command_path_unwrapped = command_path.unwrap_or(&[]);
        let command_path_for_logging = command_path;

        // Check for recursion cycle via ENV_NEST_CALL_STACK
        let command_id = command_path_unwrapped.join(":");
        if !command_id.is_empty() {
            if let Ok(stack_str) = std::env::var(ENV_NEST_CALL_STACK) {
                let stack: Vec<&str> = stack_str.split(',').collect();
                if stack.contains(&command_id.as_str()) {
                    return Err(format!(
                         "Circular dependency detected: Command '{}' is already in the call stack.\nCall stack: {}", 
                         command_id, stack_str
                     ));
                }
            }
        }

        // Validate parameters
        let validate_directives = DirectiveResolver::get_validate_directives(&command.directives);
        if !validate_directives.is_empty() {
            let (parent_vars, parent_consts) =
                self.collect_parent_variables(command_path_unwrapped);

            let mut all_env_directives = self.collect_parent_env_directives(command_path_unwrapped);
            for directive in &command.directives {
                match directive {
                    super::ast::Directive::Env(..) | super::ast::Directive::EnvFile(..) => {
                        all_env_directives.push(directive.clone());
                    }
                    _ => {}
                }
            }
            let env_vars = EnvironmentManager::extract_env_vars(&all_env_directives);

            let tpl_context = TemplateContext {
                global_variables: &self.variables,
                global_constants: &self.constants,
                local_variables: &[],
                local_constants: &[],
                parent_variables: &parent_vars,
                parent_constants: &parent_consts,
            };

            RuntimeValidator::validate(
                &validate_directives,
                args,
                &env_vars,
                &tpl_context,
                command_path_unwrapped,
                parent_args,
            )?;
        }

        // Execute dependencies first
        let (depends, parallel) = DirectiveResolver::get_depends_directive(&command.directives);
        if !depends.is_empty() {
            if verbose {
                use super::output::OutputFormatter;
                let deps_str: Vec<String> = depends
                    .iter()
                    .map(|dep| {
                        if dep.args.is_empty() {
                            dep.command_path.clone()
                        } else {
                            let args_str: Vec<String> = dep
                                .args
                                .iter()
                                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                                .collect();
                            format!("{}({})", dep.command_path, args_str.join(", "))
                        }
                    })
                    .collect();
                let parallel_msg = if parallel { " (parallel)" } else { "" };
                OutputFormatter::info(&format!(
                    "Executing dependencies for {}{}: {}",
                    command_path_unwrapped.join(" "),
                    parallel_msg,
                    deps_str.join(", ")
                ));
            }
            self.execute_dependencies(&depends, context, parallel)?;
        }

        // Check if confirmation is required
        if !dry_run {
            if let Some(confirm_message) =
                DirectiveResolver::get_require_confirm_directive(&command.directives)
            {
                match super::input::prompt_confirmation(Some(&confirm_message), command_path) {
                    Ok(true) => {
                        // User confirmed - continue execution
                    }
                    Ok(false) => {
                        // User declined - return without error
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(format!("Confirmation prompt failed: {}", e));
                    }
                }
            }
        }

        // Prepare environment
        let mut all_env_directives = if let Some(path) = command_path {
            self.collect_parent_env_directives(path)
        } else {
            Vec::new()
        };

        for directive in &command.directives {
            match directive {
                super::ast::Directive::Env(..) | super::ast::Directive::EnvFile(..) => {
                    all_env_directives.push(directive.clone());
                }
                _ => {}
            }
        }

        let env_vars = EnvironmentManager::extract_env_vars(&all_env_directives);

        let parent_directives = if let Some(path) = command_path {
            self.collect_parent_directives(path)
        } else {
            std::collections::HashMap::new()
        };

        let cwd = DirectiveResolver::get_directive_value(&command.directives, "cwd")
            .or_else(|| parent_directives.get("cwd").map(|(s, _)| s.clone()))
            .or_else(|| {
                command
                    .source_file
                    .as_ref()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_string_lossy().to_string())
            });
        let privileged = DirectiveResolver::get_privileged_directive(&command.directives);
        let logs = DirectiveResolver::get_logs_directive(&command.directives);

        let (parent_variables, parent_constants) = if let Some(path) = command_path {
            self.collect_parent_variables(path)
        } else {
            (Vec::new(), Vec::new())
        };

        let merged_parent_args = parent_args.clone();

        let mut processed_env_vars = std::collections::HashMap::new();

        EnvironmentManager::export_all_vars(
            &mut processed_env_vars,
            &self.variables,
            &self.constants,
        );

        EnvironmentManager::export_all_vars(
            &mut processed_env_vars,
            &parent_variables,
            &parent_constants,
        );

        EnvironmentManager::export_all_vars(
            &mut processed_env_vars,
            &command.local_variables,
            &command.local_constants,
        );

        for (key, value) in &env_vars {
            processed_env_vars.insert(key.clone(), value.clone());
        }

        let tpl_context = TemplateContext {
            global_variables: &self.variables,
            global_constants: &self.constants,
            local_variables: &command.local_variables,
            local_constants: &command.local_constants,
            parent_variables: &parent_variables,
            parent_constants: &parent_constants,
        };

        for value in processed_env_vars.values_mut() {
            *value = TemplateProcessor::process(value, args, &tpl_context, &merged_parent_args);
        }

        if !command_id.is_empty() {
            let new_stack = if let Ok(stack_str) = std::env::var(ENV_NEST_CALL_STACK) {
                if stack_str.is_empty() {
                    command_id.clone()
                } else {
                    format!("{},{}", stack_str, command_id)
                }
            } else {
                command_id.clone()
            };
            processed_env_vars.insert(ENV_NEST_CALL_STACK.to_string(), new_stack);
        }

        let env_vars = processed_env_vars;

        let mut script_exec_context = ScriptExecutionContext {
            args,
            env_vars: &env_vars,
            cwd: cwd.as_deref(),
            command_path: Some(command_path_unwrapped),
            dry_run,
            verbose,
            privileged,
            pid_callback: self.pid_callback.as_deref(),
            parent_args: &merged_parent_args,
            hide_output: false,
        };

        let before_info =
            DirectiveResolver::get_directive_value_with_hide(&command.directives, "before")
                .or_else(|| parent_directives.get("before").cloned());
        if let Some((before_script, hide_before)) = before_info {
            let processed_before =
                TemplateProcessor::process(&before_script, args, &tpl_context, &merged_parent_args);

            if verbose {
                use super::output::OutputFormatter;
                OutputFormatter::info("Executing before script...");
            }

            script_exec_context.hide_output = hide_before;
            if let Err(e) = self.execute_script(&processed_before, &script_exec_context) {
                return Err(format!("Before script failed: {}", e));
            }
        }

        let (script, hide_script) =
            DirectiveResolver::get_directive_value_with_hide(&command.directives, "script")
                .ok_or_else(|| "Command has no script directive".to_string())?;

        let processed_script =
            TemplateProcessor::process(&script, args, &tpl_context, &merged_parent_args);

        if privileged && !dry_run {
            use super::executor::CommandExecutor;
            if !CommandExecutor::check_privileged_access() {
                return Err(CommandExecutor::format_privileged_error(
                    command,
                    Some(command_path_unwrapped),
                ));
            }
        }

        if dry_run {
            use super::executor::{CommandExecutor, ExecutionContext};
            let dry_run_context = ExecutionContext {
                command,
                args,
                env_vars: &env_vars,
                cwd: cwd.as_deref(),
                command_path: Some(command_path_unwrapped),
                dry_run: true,
                verbose,
                privileged,
                pid_callback: None,
                hide_output: false,
            };
            CommandExecutor::show_dry_run_preview(&processed_script, &dry_run_context);
            return Ok(());
        }

        if verbose {
            use super::executor::{CommandExecutor, ExecutionContext};
            let verbose_context = ExecutionContext {
                command,
                args,
                env_vars: &env_vars,
                cwd: cwd.as_deref(),
                command_path: Some(command_path_unwrapped),
                dry_run: false,
                verbose: true,
                privileged,
                pid_callback: None,
                hide_output: hide_script,
            };
            CommandExecutor::show_verbose_info(&processed_script, &verbose_context);
        }

        script_exec_context.hide_output = hide_script;
        let main_result = self.execute_script(&processed_script, &script_exec_context);

        let result = match main_result {
            Ok(()) => {
                let after_info =
                    DirectiveResolver::get_directive_value_with_hide(&command.directives, "after")
                        .or_else(|| parent_directives.get("after").cloned());
                if let Some((after_script, hide_after)) = after_info {
                    let processed_after = TemplateProcessor::process(
                        &after_script,
                        args,
                        &tpl_context,
                        &merged_parent_args,
                    );

                    if verbose {
                        use super::output::OutputFormatter;
                        OutputFormatter::info("Executing after script...");
                    }

                    script_exec_context.hide_output = hide_after;
                    if let Err(e) = self.execute_script(&processed_after, &script_exec_context) {
                        return Err(format!("After script failed: {}", e));
                    }
                }
                Ok(())
            }
            Err(error_msg) => {
                let fallback_info = DirectiveResolver::get_directive_value_with_hide(
                    &command.directives,
                    "fallback",
                )
                .or_else(|| parent_directives.get("fallback").cloned());
                if let Some((fallback_script, hide_fallback)) = fallback_info {
                    let mut fallback_args = args.clone();
                    fallback_args.insert("SYSTEM_ERROR_MESSAGE".to_string(), error_msg.clone());
                    fallback_args.insert("error".to_string(), error_msg.clone());

                    let processed_fallback = TemplateProcessor::process(
                        &fallback_script,
                        &fallback_args,
                        &tpl_context,
                        &merged_parent_args,
                    );

                    if verbose {
                        use super::output::OutputFormatter;
                        OutputFormatter::info("Executing fallback script...");
                    }

                    let fallback_context = ScriptExecutionContext {
                        args: &fallback_args,
                        hide_output: hide_fallback,
                        ..script_exec_context
                    };
                    if let Err(e) = self.execute_script(&processed_fallback, &fallback_context) {
                        return Err(format!("Fallback script failed: {}", e));
                    }
                    Ok(())
                } else {
                    Err(error_msg)
                }
            }
        };

        if let Some((log_path, log_format)) = logs {
            if !dry_run {
                if let Err(e) = super::logging::write_log_entry(
                    &log_path,
                    &log_format,
                    command_path_for_logging,
                    args,
                    &result,
                ) {
                    if verbose {
                        use super::output::OutputFormatter;
                        OutputFormatter::warning(&format!("Failed to write log: {}", e));
                    }
                }
            }
        }

        let finally_info =
            DirectiveResolver::get_directive_value_with_hide(&command.directives, "finally")
                .or_else(|| parent_directives.get("finally").cloned());
        if let Some((finally_script, hide_finally)) = finally_info {
            let original_result = match &result {
                Ok(_) => Ok(()),
                Err(e) => Err(e.clone()),
            };

            let tpl_context = TemplateContext {
                global_variables: &self.variables,
                global_constants: &self.constants,
                local_variables: &command.local_variables,
                local_constants: &command.local_constants,
                parent_variables: &parent_variables,
                parent_constants: &parent_constants,
            };
            let processed_finally = TemplateProcessor::process(
                &finally_script,
                args,
                &tpl_context,
                &merged_parent_args,
            );

            if verbose {
                use super::output::OutputFormatter;
                OutputFormatter::info("Executing finally script...");
            }

            script_exec_context.hide_output = hide_finally;
            script_exec_context.args = args;
            if let Err(e) = self.execute_script(&processed_finally, &script_exec_context) {
                if verbose {
                    use super::output::OutputFormatter;
                    OutputFormatter::warning(&format!("Finally script failed: {}", e));
                }
            }

            return original_result;
        }

        if let Some(path) = command_path {
            context.visited.remove(path);
        }

        result
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
        let parent_args = HashMap::new(); // Top-level command has no parent args
        let mut context = CommandExecutionContext {
            command,
            args,
            command_path,
            dry_run,
            verbose,
            visited: &mut visited,
            parent_args: &parent_args,
        };
        self.execute_command_with_deps(&mut context)
    }

    pub fn execute_command_with_parent_args(
        &self,
        command: &Command,
        args: &HashMap<String, String>,
        command_path: Option<&[String]>,
        dry_run: bool,
        verbose: bool,
        parent_args: &HashMap<String, String>,
    ) -> Result<(), String> {
        let mut visited = std::collections::HashSet::new();
        let mut context = CommandExecutionContext {
            command,
            args,
            command_path,
            dry_run,
            verbose,
            visited: &mut visited,
            parent_args,
        };
        self.execute_command_with_deps(&mut context)
    }
}
