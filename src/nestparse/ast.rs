use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Bool(bool),
    Number(f64),
    Array(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub alias: Option<String>,
    pub param_type: String,
    pub default: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum Directive {
    Desc(String),
    Cwd(String),
    Env(String),
    Script(String), // Single line or multiline script
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub directives: Vec<Directive>,
    pub children: Vec<Command>,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.parameters.is_empty() {
            let params: Vec<String> = self.parameters.iter().map(|p| {
                let mut s = p.name.clone();
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

