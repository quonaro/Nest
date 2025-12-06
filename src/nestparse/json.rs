use super::ast::{Command, Directive, Parameter, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsonValue {
    String(String),
    Bool(bool),
    Number(f64),
    Array(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct JsonParameter {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub param_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<JsonValue>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsonDirective {
    #[serde(rename = "desc")]
    Desc(String),
    #[serde(rename = "cwd")]
    Cwd(String),
    #[serde(rename = "env")]
    Env(String),
    #[serde(rename = "script")]
    Script(String),
}

#[derive(Serialize, Deserialize)]
pub struct JsonCommand {
    pub name: String,
    pub parameters: Vec<JsonParameter>,
    pub directives: Vec<JsonDirective>,
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

pub fn to_json(commands: &[Command]) -> Result<String, serde_json::Error> {
    let json_commands: Vec<JsonCommand> = commands.iter().map(|c| c.into()).collect();
    serde_json::to_string_pretty(&json_commands)
}

