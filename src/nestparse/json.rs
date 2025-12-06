//! JSON serialization for command structures.
//!
//! This module provides JSON representations of the AST structures,
//! allowing commands to be exported in JSON format via `nest --show json`.

use super::ast::{Command, Directive, Parameter, Value};
use serde::{Deserialize, Serialize};

/// JSON representation of a Value.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JsonValue {
    /// A string value
    String(String),
    /// A boolean value
    Bool(bool),
    /// A numeric value
    Number(f64),
    /// An array of strings
    Array(Vec<String>),
}

/// JSON representation of a Parameter.
#[derive(Serialize, Deserialize)]
pub struct JsonParameter {
    /// The parameter name
    pub name: String,
    /// Optional alias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    /// The parameter type
    pub param_type: String,
    /// Optional default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<JsonValue>,
}

/// JSON representation of a Directive.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum JsonDirective {
    /// Description directive
    #[serde(rename = "desc")]
    Desc(String),
    /// Working directory directive
    #[serde(rename = "cwd")]
    Cwd(String),
    /// Environment variable directive
    #[serde(rename = "env")]
    Env(String),
    /// Script directive
    #[serde(rename = "script")]
    Script(String),
}

/// JSON representation of a Command.
#[derive(Serialize, Deserialize)]
pub struct JsonCommand {
    /// The command name
    pub name: String,
    /// List of parameters
    pub parameters: Vec<JsonParameter>,
    /// List of directives
    pub directives: Vec<JsonDirective>,
    /// Child commands
    pub children: Vec<JsonCommand>,
}

impl From<&Value> for JsonValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Bool(b) => JsonValue::Bool(*b),
            Value::Number(n) => JsonValue::Number(*n),
            Value::Array(a) => JsonValue::Array(a.clone()),
        }
    }
}

impl From<&Parameter> for JsonParameter {
    fn from(param: &Parameter) -> Self {
        JsonParameter {
            name: param.name.clone(),
            alias: param.alias.clone(),
            param_type: param.param_type.clone(),
            default: param.default.as_ref().map(|v| v.into()),
        }
    }
}

impl From<&Directive> for JsonDirective {
    fn from(directive: &Directive) -> Self {
        match directive {
            Directive::Desc(s) => JsonDirective::Desc(s.clone()),
            Directive::Cwd(s) => JsonDirective::Cwd(s.clone()),
            Directive::Env(s) => JsonDirective::Env(s.clone()),
            Directive::Script(s) => JsonDirective::Script(s.clone()),
        }
    }
}

impl From<&Command> for JsonCommand {
    fn from(command: &Command) -> Self {
        JsonCommand {
            name: command.name.clone(),
            parameters: command.parameters.iter().map(|p| p.into()).collect(),
            directives: command.directives.iter().map(|d| d.into()).collect(),
            children: command.children.iter().map(|c| c.into()).collect(),
        }
    }
}

/// Converts a list of commands to a pretty-printed JSON string.
///
/// # Arguments
///
/// * `commands` - The list of commands to serialize
///
/// # Returns
///
/// Returns `Ok(json_string)` with the JSON representation,
/// or `Err(error)` if serialization fails.
pub fn to_json(commands: &[Command]) -> Result<String, serde_json::Error> {
    let json_commands: Vec<JsonCommand> = commands.iter().map(|c| c.into()).collect();
    serde_json::to_string_pretty(&json_commands)
}

