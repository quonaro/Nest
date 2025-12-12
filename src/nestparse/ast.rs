//! Abstract Syntax Tree (AST) structures for representing parsed Nestfile configuration.
//!
//! This module defines the data structures that represent the parsed configuration
//! in a structured, programmatic format.

use std::fmt;

/// Represents a value that can be used in parameters and directives.
///
/// Values can be strings, booleans, numbers, or arrays of strings.
#[derive(Debug, Clone)]
pub enum Value {
    /// A string value
    String(String),
    /// A boolean value (true/false)
    Bool(bool),
    /// A numeric value (floating point)
    Number(f64),
    /// An array of string values
    Array(Vec<String>),
}

/// Represents a command parameter with its type, default value, and optional alias.
///
/// Parameters can be required or optional (if a default value is provided).
/// They can also have aliases for shorter command-line usage.
/// Parameters can be positional (by default) or named (with ! prefix).
#[derive(Debug, Clone)]
pub struct Parameter {
    /// The parameter name
    pub name: String,
    /// Optional alias (e.g., "f" for "force")
    pub alias: Option<String>,
    /// The parameter type: "str", "bool", "num", or "arr"
    pub param_type: String,
    /// Optional default value
    pub default: Option<Value>,
    /// Whether this parameter is named (uses --name) or positional
    pub is_named: bool,
}

/// Represents a dependency with optional arguments.
///
/// A dependency can be a simple command path (e.g., "clean") or
/// a command with arguments (e.g., "build(target=\"x86_64\")").
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Command path (e.g., "clean" or "dev:build")
    pub command_path: String,
    /// Arguments to pass to the dependency command
    /// Key is parameter name, value is argument value as string
    pub args: std::collections::HashMap<String, String>,
}

/// Represents a directive that modifies command behavior.
///
/// Directives are special instructions in the Nestfile that control
/// how commands are executed.
#[derive(Debug, Clone)]
pub enum Directive {
    /// Description of the command (used in help text)
    Desc(String),
    /// Working directory for command execution
    Cwd(String),
    /// Environment variable assignment or .env file path
    Env(String),
    /// Script to execute (can be single line or multiline)
    Script(String),
    /// Script to execute with hidden output (can be single line or multiline)
    ScriptHide(String),
    /// Script to execute before the main script (can be single line or multiline)
    Before(String),
    /// Script to execute before the main script with hidden output (can be single line or multiline)
    BeforeHide(String),
    /// Script to execute after the main script (can be single line or multiline)
    After(String),
    /// Script to execute after the main script with hidden output (can be single line or multiline)
    AfterHide(String),
    /// Script to execute if the main script fails (can be single line or multiline)
    /// Replaces error output with this script's output
    Fallback(String),
    /// Script to execute if the main script fails with hidden output (can be single line or multiline)
    /// Replaces error output with this script's output
    FallbackHide(String),
    /// Whether this command requires privileged access (sudo/administrator)
    Privileged(bool),
    /// Dependencies - commands that must be executed before this command
    /// Each dependency can have arguments (e.g., "build(target=\"x86_64\")")
    Depends(Vec<Dependency>),
    /// Validation rules for parameters
    /// Format: "param_name matches /regex/" or "param_name matches /regex/ flags"
    Validate(String),
    /// Logging directive - logs command output to a file
    /// First String is the file path, second is the format ("json" or "txt")
    Logs(String, String),
    /// Conditional execution - if condition is true, execute the following script
    /// String contains the condition expression
    If(String),
    /// Else branch for conditional execution
    /// Executes if all previous if/elif conditions were false
    Else,
    /// Else-if branch for conditional execution
    /// String contains the condition expression
    Elif(String),
    /// Require user confirmation before executing the command
    /// String contains the confirmation message (optional, uses default if empty)
    RequireConfirm(String),
}

/// Represents a variable that can be redefined.
#[derive(Debug, Clone)]
pub struct Variable {
    /// The variable name
    pub name: String,
    /// The variable value
    pub value: String,
}

/// Represents a constant that cannot be redefined.
#[derive(Debug, Clone)]
pub struct Constant {
    /// The constant name
    pub name: String,
    /// The constant value
    pub value: String,
}

/// Represents a function that can be reused in scripts.
///
/// Functions are defined at the global level and can:
/// - Execute commands
/// - Call other functions
/// - Use variables, constants, and environment variables
/// - Have parameters
/// - Define local variables
#[derive(Debug, Clone)]
pub struct Function {
    /// The function name
    pub name: String,
    /// List of parameters this function accepts
    #[allow(dead_code)]
    pub parameters: Vec<Parameter>,
    /// The function body (script content)
    pub body: String,
    /// Local variables for this function
    pub local_variables: Vec<Variable>,
}

/// Represents a command in the configuration file.
///
/// Commands can have:
/// - Parameters (arguments and flags)
/// - Directives (description, working directory, environment variables, script)
/// - Child commands (nested subcommands)
/// - Wildcard parameter (*) that accepts all remaining arguments
/// - Local variables and constants (scoped to this command)
///
/// Commands form a tree structure where parent commands can have child commands.
#[derive(Debug, Clone)]
pub struct Command {
    /// The command name
    pub name: String,
    /// List of parameters this command accepts
    pub parameters: Vec<Parameter>,
    /// List of directives that modify command behavior
    pub directives: Vec<Directive>,
    /// Child commands (subcommands)
    pub children: Vec<Command>,
    /// Whether this command accepts all remaining arguments via wildcard (*)
    pub has_wildcard: bool,
    /// Local variables for this command (can override global variables)
    pub local_variables: Vec<Variable>,
    /// Local constants for this command (can override global constants)
    pub local_constants: Vec<Constant>,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.has_wildcard {
            write!(f, "(*)")?;
        } else if !self.parameters.is_empty() {
            let params: Vec<String> = self.parameters.iter().map(|p| {
                let mut s = if p.is_named { "!".to_string() } else { String::new() };
                s.push_str(&p.name);
                if let Some(alias) = &p.alias {
                    s.push_str(&format!("|{}", alias));
                }
                s.push_str(&format!(": {}", p.param_type));
                if let Some(default) = &p.default {
                    s.push_str(&format!(" = {:?}", default));
                }
                s
            }).collect();
            write!(f, "({})", params.join(", "))?;
        }
        Ok(())
    }
}

