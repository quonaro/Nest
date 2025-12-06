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
}

/// Represents a command in the configuration file.
///
/// Commands can have:
/// - Parameters (arguments and flags)
/// - Directives (description, working directory, environment variables, script)
/// - Child commands (nested subcommands)
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
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.parameters.is_empty() {
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

