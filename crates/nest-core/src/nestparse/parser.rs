//! Parser for Nestfile configuration files.
//!
//! This module parses the Nestfile syntax into an Abstract Syntax Tree (AST).
//! It handles nested commands, parameters, directives, and multiline constructs.

use super::ast::{Constant, Dependency, Directive, Function, Parameter, Value, Variable};
use crate::constants::{BOOL_FALSE, BOOL_TRUE, INDENT_SIZE};
use std::collections::HashMap;
use std::process::Command as ProcessCommand;

/// Parser state for processing Nestfile content.
///
/// The parser maintains its position in the file and processes commands
/// recursively based on indentation levels.
pub struct Parser {
    /// All lines of the configuration file
    lines: Vec<String>,
    /// Current position in the file (line index)
    current_index: usize,
    /// Current source file path (for tracking where commands come from)
    current_source_file: Option<std::path::PathBuf>,
}

/// Errors that can occur during parsing.
#[derive(Debug)]
pub enum ParseError {
    /// Unexpected end of file (e.g., incomplete command definition)
    UnexpectedEndOfFile(usize),
    /// Invalid syntax in the configuration file
    InvalidSyntax(String, usize),
    /// Invalid indentation (e.g., child command not properly indented)
    InvalidIndent(usize),
    /// Deprecated syntax used in the configuration file
    DeprecatedSyntax(String, usize),
}

/// Result of parsing a configuration file.
#[derive(Debug)]
pub struct ParseResult {
    /// List of parsed commands
    pub commands: Vec<super::ast::Command>,
    /// List of parsed variables (can be redefined)
    pub variables: Vec<Variable>,
    /// List of parsed constants (cannot be redefined)
    pub constants: Vec<Constant>,
    /// List of parsed functions (reusable scripts)
    pub functions: Vec<Function>,
}

impl Parser {
    /// Creates a new parser from file content.
    ///
    /// # Arguments
    ///
    /// * `content` - The entire content of the configuration file
    ///
    /// # Returns
    ///
    /// Returns a new `Parser` instance ready to parse the content.
    pub fn new(content: &str) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            lines,
            current_index: 0,
            current_source_file: None,
        }
    }

    /// Gets the current line number (1-based).
    fn current_line_number(&self) -> usize {
        self.current_index + 1
    }

    /// Parses the entire configuration file into commands, variables, and constants.
    ///
    /// This is the main entry point for parsing. It processes all top-level
    /// commands, variables, and constants.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ParseResult)` with the parsed structure,
    /// or `Err(error)` if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File structure is invalid
    /// - Indentation is incorrect
    /// - Unexpected end of file
    /// - Constant is redefined
    pub fn parse(&mut self) -> Result<ParseResult, ParseError> {
        let mut commands = Vec::new();
        let mut variables = Vec::new();
        let mut constants = Vec::new();
        let mut functions = Vec::new();
        let mut constant_names = std::collections::HashSet::new();

        while self.current_index < self.lines.len() {
            let line = &self.lines[self.current_index];
            let trimmed = line.trim();

            // Check for source file marker: # @source: /path/to/file
            if let Some(source_path) = trimmed.strip_prefix("# @source: ") {
                let source_path = source_path.trim();
                if !source_path.is_empty() {
                    self.current_source_file = Some(std::path::PathBuf::from(source_path));
                }
                self.current_index += 1;
                continue;
            }

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.current_index += 1;
                continue;
            }

            // DEPRECATION CHECK: Detect legacy syntax prefixes
            if trimmed.starts_with('@') {
                let msg = format!(
                    "Legacy syntax '@' is deprecated. Use keywords (var, const, env, import, function) without '@'. The last version supporting this legacy configuration is 3.0.19. Line: {}",
                    trimmed
                );
                return Err(ParseError::DeprecatedSyntax(msg, self.current_line_number()));
            }
            if trimmed.starts_with('>') {
                let msg = format!(
                    "Legacy directive prefix '>' is deprecated. Remove '>' and use 'key: value' style. The last version supporting this legacy configuration is 3.0.19. Line: {}",
                    trimmed
                );
                return Err(ParseError::DeprecatedSyntax(msg, self.current_line_number()));
            }

            // Check for new keywords (import, var, const, env, function)
            if trimmed.starts_with("var ") {
                let var = self.parse_variable()?;
                variables.retain(|v: &Variable| v.name != var.name);
                variables.push(var);
                continue;
            } else if trimmed.starts_with("const ") {
                let const_def = self.parse_constant()?;
                if constant_names.contains(&const_def.name) {
                    return Err(ParseError::InvalidSyntax(
                        format!(
                            "Constant '{}' is already defined and cannot be redefined",
                            const_def.name
                        ),
                        self.current_line_number(),
                    ));
                }
                constant_names.insert(const_def.name.clone());
                constants.push(const_def);
                continue;
            } else if trimmed.starts_with("env ") {
                // Handle 'env KEY = VALUE' or 'env .env'
                self.parse_env_keyword(&mut variables)?;
                continue;
            } else if trimmed.starts_with("import ") {
                self.parse_import()?;
                continue;
            } else if trimmed.starts_with("function ") {
                let func = self.parse_function()?;
                functions.push(func);
                continue;
            }

            // Check if it's a command definition (ends with : or contains opening parenthesis but not closing)
            let is_command = trimmed.ends_with(':') || (trimmed.contains('(') && !trimmed.contains(')'));

            if is_command {
                let command = self.parse_command(0)?;
                commands.push(command);
            } else {
                self.current_index += 1;
            }
        }

        // Merge commands (Last Wins strategy for duplicate names)
        let merged_commands = crate::nestparse::merge::merge_commands(commands);

        Ok(ParseResult {
            commands: merged_commands,
            variables,
            constants,
            functions,
        })
    }

    fn parse_command(&mut self, base_indent: u8) -> Result<super::ast::Command, ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile(self.current_line_number()));
        }

        let line = &self.lines[self.current_index];
        let indent = get_indent_size(line);

        if indent < base_indent {
            return Err(ParseError::InvalidIndent(self.current_line_number()));
        }

        // Parse function signature: name(params): (may be multiline)
        let (name, parameters) = self.parse_function_signature_multiline(indent)?;

        // current_index already incremented in parse_function_signature_multiline

        let mut directives = Vec::new();
        let mut children = Vec::new();
        let mut local_variables = Vec::new();
        let mut local_constants = Vec::new();
        let mut local_constant_names = std::collections::HashSet::new();

        // Parse directives, local variables/constants, and children
        while self.current_index < self.lines.len() {
            let next_line = self.lines[self.current_index].clone();
            let next_indent = get_indent_size(&next_line);
            let next_trimmed = next_line.trim();

            // If indent is less or equal, we're done with this command
            if next_indent <= indent && !next_trimmed.is_empty() {
                break;
            }

            // Check for source file marker: # @source: /path/to/file
            if let Some(source_path) = next_trimmed.strip_prefix("# @source: ") {
                let source_path = source_path.trim();
                if !source_path.is_empty() {
                    self.current_source_file = Some(std::path::PathBuf::from(source_path));
                }
                self.current_index += 1;
                continue;
            }

            // Skip empty lines and comments
            if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                self.current_index += 1;
                continue;
            }

            // DEPRECATION CHECK: Detect legacy syntax prefixes in commands
            if next_trimmed.starts_with('@') {
                let msg = format!(
                    "Legacy syntax '@' is deprecated. Use keywords (var, const, env) without '@'. The last version supporting this legacy configuration is 3.0.19. Line: {}",
                    next_trimmed
                );
                return Err(ParseError::DeprecatedSyntax(msg, self.current_line_number()));
            }
            if next_trimmed.starts_with('>') {
                let msg = format!(
                    "Legacy directive prefix '>' is deprecated. Remove '>' and use 'key: value' style. The last version supporting this legacy configuration is 3.0.19. Line: {}",
                    next_trimmed
                );
                return Err(ParseError::DeprecatedSyntax(msg, self.current_line_number()));
            }

            // Check for local variable, constant, or env definition
            if next_trimmed.starts_with("var ") {
                let var = self.parse_variable()?;
                local_variables.retain(|v: &Variable| v.name != var.name);
                local_variables.push(var);
                continue;
            } else if next_trimmed.starts_with("const ") {
                let const_def = self.parse_constant()?;
                if local_constant_names.contains(&const_def.name) {
                    return Err(ParseError::InvalidSyntax(
                        format!("Constant '{}' is already defined in this command and cannot be redefined", const_def.name),
                        self.current_line_number()
                    ));
                }
                local_constant_names.insert(const_def.name.clone());
                local_constants.push(const_def);
                continue;
            } else if next_trimmed.starts_with("env ") {
                // For local env, we store it as a Directive::Env
                let env_directive = self.parse_env_directive_keyword()?;
                directives.push(env_directive);
                continue;
            }

            // Check if it's a directive (property: value or property.mod: value)
            let is_directive = next_trimmed.contains(':') && {
                let key_part = next_trimmed.split(':').next().unwrap_or("").trim();
                // Key part shouldn't have spaces unless it's a command name with params
                !key_part.contains(' ') || (key_part.contains('(') && key_part.contains(')'))
            };

            if is_directive && !next_trimmed.ends_with(':') {
                 // It's a property: value directive
                 let directive = self.parse_property_directive(&next_line, next_indent)?;
                 directives.push(directive);
            }
            // Check if it's a child command
            else if next_trimmed.ends_with(':') || (next_trimmed.contains('(') && !next_trimmed.contains(')')) {
                let child = self.parse_command(indent)?;
                children.push(child);
            } else {
                return Err(ParseError::InvalidSyntax(
                    format!("Unexpected line in command definition: {}", next_trimmed),
                    self.current_line_number()
                ));
            }
        }

        let has_wildcard = parameters
            .iter()
            .any(|p| matches!(p.kind, super::ast::ParamKind::Wildcard { .. }));

        Ok(super::ast::Command {
            name,
            parameters,
            directives,
            children,
            has_wildcard,
            local_variables,
            local_constants,
            source_file: self.current_source_file.clone(),
        })
    }

    fn parse_function_signature_multiline(
        &mut self,
        base_indent: u8,
    ) -> Result<(String, Vec<Parameter>), ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile(self.current_line_number()));
        }

        let line = &self.lines[self.current_index];
        let trimmed = line.trim();

        // Find opening parenthesis
        if let Some(open_paren) = trimmed.find('(') {
            let name = trimmed[..open_paren].trim().to_string();

            // Check if closing parenthesis is on the same line
            if let Some(close_paren) = trimmed.rfind(')') {
                // Single line signature
                let params_str = &trimmed[open_paren + 1..close_paren];
                let trimmed_params = params_str.trim();

                let parameters = if trimmed_params.is_empty() {
                    Vec::new()
                } else {
                    self.parse_parameters(params_str, self.current_line_number())?
                };
                self.current_index += 1;
                Ok((name, parameters))
            } else {
                // Multiline signature - collect lines until we find closing parenthesis
                let mut params_lines = Vec::new();
                self.current_index += 1; // Move past the line with opening parenthesis

                while self.current_index < self.lines.len() {
                    let next_line = &self.lines[self.current_index];
                    let next_indent = get_indent_size(next_line);
                    let next_trimmed = next_line.trim();

                    // Skip empty lines and comments
                    if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                        self.current_index += 1;
                        continue;
                    }

                    // If we find closing parenthesis, we're done
                    if next_trimmed.contains(')') {
                        // Extract the part before closing parenthesis and ':'
                        let line_without_colon = next_trimmed.trim_end_matches(':').trim();
                        if let Some(close_paren) = line_without_colon.find(')') {
                            let param_part = line_without_colon[..close_paren].trim();
                            if !param_part.is_empty() {
                                params_lines.push(param_part.to_string());
                            }
                        }
                        self.current_index += 1;
                        break;
                    }

                    // If indent is less than base, something's wrong
                    if next_indent <= base_indent && !next_trimmed.is_empty() {
                        return Err(ParseError::InvalidSyntax(
                            "Missing closing parenthesis in function signature".to_string(),
                            self.current_line_number(),
                        ));
                    }

                    // Add this line to params (remove inline comments if any)
                    let line_without_comment = if let Some(comment_pos) = next_trimmed.find('#') {
                        next_trimmed[..comment_pos].trim()
                    } else {
                        next_trimmed
                    };
                    if !line_without_comment.is_empty() {
                        params_lines.push(line_without_comment.to_string());
                    }
                    self.current_index += 1;
                }

                let params_str = params_lines.join(" ");
                let trimmed_params = params_str.trim();

                let parameters = if trimmed_params.is_empty() {
                    Vec::new()
                } else {
                    self.parse_parameters(&params_str, self.current_line_number())?
                };

                Ok((name, parameters))
            }
        } else {
            // No parameters - check if it ends with ':'
            let name = if trimmed.ends_with(':') {
                trimmed.trim_end_matches(':').trim().to_string()
            } else {
                trimmed.to_string()
            };
            self.current_index += 1;
            Ok((name, Vec::new()))
        }
    }

    fn parse_parameters(
        &self,
        params_str: &str,
        line_number: usize,
    ) -> Result<Vec<Parameter>, ParseError> {
        use super::ast::ParamKind;

        let mut parameters = Vec::new();
        let mut current_param = String::new();
        let mut paren_depth = 0;
        let mut param_strings = Vec::new();

        // First, collect all parameter strings
        for ch in params_str.chars() {
            match ch {
                '(' => {
                    paren_depth += 1;
                    current_param.push(ch);
                }
                ')' => {
                    paren_depth -= 1;
                    current_param.push(ch);
                }
                ',' if paren_depth == 0 => {
                    if !current_param.trim().is_empty() {
                        param_strings.push(current_param.trim().to_string());
                    }
                    current_param.clear();
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }

        if !current_param.trim().is_empty() {
            param_strings.push(current_param.trim().to_string());
        }

        // Parse all parameters, including wildcard specifications
        for param_str in param_strings {
            let trimmed = param_str.trim();

            // Wildcard parameter syntaxes:
            // - "*"
            // - "*name"
            // - "*[N]"
            // - "*name[N]"
            if trimmed.starts_with('*') {
                let wildcard_param = self.parse_wildcard_parameter(trimmed, line_number)?;
                parameters.push(wildcard_param);
            } else {
                parameters.push(self.parse_parameter(trimmed, line_number)?);
            }
        }

        // Validate that there are no two wildcard parameters adjacent to each other
        for window in parameters.windows(2) {
            if matches!(window[0].kind, ParamKind::Wildcard { .. })
                && matches!(window[1].kind, ParamKind::Wildcard { .. })
            {
                return Err(ParseError::InvalidSyntax(
                    "Wildcard parameters cannot be adjacent (e.g., \"*, *\" or \"*a, *b\")"
                        .to_string(),
                    line_number,
                ));
            }
        }

        Ok(parameters)
    }

    fn parse_parameter(
        &self,
        param_str: &str,
        line_number: usize,
    ) -> Result<Parameter, ParseError> {
        // Format: [!]name|alias: type = default
        // ! prefix means named argument (uses --name)
        let parts: Vec<&str> = param_str.split(':').collect();

        if parts.len() < 2 {
            return Err(ParseError::InvalidSyntax(
                format!(
                    "Invalid parameter syntax '{}'. Missing type annotation. Expected format: [!]name|alias: type [= default]",
                    param_str
                ),
                line_number,
            ));
        }

        let name_part = parts[0].trim();
        let type_default_str: String = parts[1..].join(":");
        let type_default = type_default_str.trim();

        // Check if it's a named argument (starts with !)
        let (is_named, name_part_clean) = if name_part.starts_with('!') {
            (true, &name_part[1..])
        } else {
            (false, name_part)
        };

        // Parse name and alias
        let (name, alias) = if let Some(pipe_pos) = name_part_clean.find('|') {
            (
                name_part_clean[..pipe_pos].trim().to_string(),
                Some(name_part_clean[pipe_pos + 1..].trim().to_string()),
            )
        } else {
            (name_part_clean.to_string(), None)
        };

        // Parse type and default
        let (param_type, default) = if let Some(eq_pos) = type_default.find('=') {
            let param_type = type_default[..eq_pos].trim().to_string();
            let default_str = type_default[eq_pos + 1..].trim();
            let default_value = self.parse_value(default_str)?;
            (param_type, Some(default_value))
        } else {
            (type_default.to_string(), None)
        };

        Ok(Parameter {
            name,
            alias,
            param_type,
            default,
            is_named,
            kind: super::ast::ParamKind::Normal,
        })
    }

    fn parse_wildcard_parameter(
        &self,
        param_str: &str,
        line_number: usize,
    ) -> Result<Parameter, ParseError> {
        use super::ast::{ParamKind, Parameter};

        let s = param_str.trim();

        // Wildcard parameters cannot specify type or default value
        if s.contains(':') || s.contains('=') {
            return Err(ParseError::InvalidSyntax(
                format!(
                    "Wildcard parameter '{}' cannot have a type annotation or default value",
                    param_str
                ),
                line_number,
            ));
        }

        // Expect leading '*'
        if !s.starts_with('*') {
            return Err(ParseError::InvalidSyntax(
                format!("Invalid wildcard parameter syntax: {}", param_str),
                line_number,
            ));
        }

        let rest = &s[1..];
        let (name_part, count_part) = if let Some(bracket_pos) = rest.find('[') {
            // Expect closing ']'
            if !rest.ends_with(']') {
                return Err(ParseError::InvalidSyntax(
                    format!(
                        "Wildcard parameter '{}' has an opening '[' without matching ']'",
                        param_str
                    ),
                    line_number,
                ));
            }
            let name_part = &rest[..bracket_pos];
            let count_str = &rest[bracket_pos + 1..rest.len() - 1];
            (name_part.trim(), Some(count_str.trim()))
        } else {
            (rest.trim(), None)
        };

        let name_opt = if name_part.is_empty() {
            None
        } else {
            Some(name_part.to_string())
        };

        let count_opt = if let Some(count_str) = count_part {
            if count_str.is_empty() {
                return Err(ParseError::InvalidSyntax(
                    format!(
                        "Wildcard parameter '{}' has empty size specification []",
                        param_str
                    ),
                    line_number,
                ));
            }
            let n: usize = count_str.parse().map_err(|_| {
                ParseError::InvalidSyntax(
                    format!(
                        "Wildcard parameter '{}' has invalid size specification '[{}]'",
                        param_str, count_str
                    ),
                    line_number,
                )
            })?;
            if n == 0 {
                return Err(ParseError::InvalidSyntax(
                    "Wildcard parameter size must be at least 1".to_string(),
                    line_number,
                ));
            }
            Some(n)
        } else {
            None
        };

        // For wildcard parameters, we treat them as positional array-like parameters.
        // Use "arr" as an internal param_type to keep compatibility with type validation.
        // Public name (used in templates and argument map):
        // - anonymous `*`   -> "*"
        // - named `*name`   -> "*name"
        let public_name = match &name_opt {
            Some(n) => format!("*{}", n),
            None => "*".to_string(),
        };

        Ok(Parameter {
            name: public_name,
            alias: None,
            param_type: "arr".to_string(),
            default: None,
            is_named: false,
            kind: ParamKind::Wildcard {
                name: name_opt,
                count: count_opt,
            },
        })
    }

    fn parse_value(&self, value_str: &str) -> Result<Value, ParseError> {
        let trimmed = value_str.trim();

        // Boolean
        if trimmed == BOOL_TRUE {
            return Ok(Value::Bool(true));
        }
        if trimmed == BOOL_FALSE {
            return Ok(Value::Bool(false));
        }

        // Array - support both [item1, item2] and (item1, item2) formats
        if (trimmed.starts_with('[') && trimmed.ends_with(']'))
            || (trimmed.starts_with('(') && trimmed.ends_with(')'))
        {
            let content = &trimmed[1..trimmed.len() - 1];
            let items: Vec<String> = content
                .split(',')
                .map(|s| {
                    let s = s.trim();
                    let s = if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
                        &s[1..s.len() - 1]
                    } else {
                        s
                    };
                    // Command substitution in array items? 
                    // Let's assume yes, for consistency.
                    self.execute_command_substitutions(s, self.current_line_number()).unwrap_or_else(|_| s.to_string())
                })
                .filter(|s| !s.is_empty())
                .collect();
            return Ok(Value::Array(items));
        }

        // Number
        if let Ok(num) = trimmed.parse::<f64>() {
            return Ok(Value::Number(num));
        }

        // String literal or just raw string
        let s = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        // Support command substitution $(...)
        let s = self.execute_command_substitutions(s, self.current_line_number())?;

        Ok(Value::String(s))
    }


    fn parse_variable(&mut self) -> Result<Variable, ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile(self.current_line_number()));
        }

        let line = &self.lines[self.current_index];
        let trimmed = line.trim();

        // Format: var NAME = value
        let var_part = trimmed.strip_prefix("var ").unwrap_or("").trim();

        if let Some(eq_pos) = var_part.find('=') {
            let name = var_part[..eq_pos].trim().to_string();
            let value_str = var_part[eq_pos + 1..].trim();

            if name.is_empty() {
                return Err(ParseError::InvalidSyntax(
                    "Variable name cannot be empty".to_string(),
                    self.current_line_number(),
                ));
            }

            // Parse value: if start with quote, use parse_value, otherwise use as-is
            let mut value = if (value_str.starts_with('"') && value_str.ends_with('"'))
                || (value_str.starts_with('\'') && value_str.ends_with('\''))
            {
                self.parse_value(value_str)?.to_string()
            } else {
                value_str.to_string()
            };

            // Execute command substitutions $(...) if present
            value = self.execute_command_substitutions(&value, self.current_line_number())?;

            self.current_index += 1;
            Ok(Variable { name, value })
        } else {
            Err(ParseError::InvalidSyntax(
                format!(
                    "Invalid variable syntax. Expected: var NAME = value, got: {}",
                    trimmed
                ),
                self.current_line_number(),
            ))
        }
    }

    fn parse_constant(&mut self) -> Result<Constant, ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile(self.current_line_number()));
        }

        let line = &self.lines[self.current_index];
        let trimmed = line.trim();

        // Format: const NAME = value
        let const_part = trimmed.strip_prefix("const ").unwrap_or("").trim();

        if let Some(eq_pos) = const_part.find('=') {
            let name = const_part[..eq_pos].trim().to_string();
            let value_str = const_part[eq_pos + 1..].trim();

            if name.is_empty() {
                return Err(ParseError::InvalidSyntax(
                    "Constant name cannot be empty".to_string(),
                    self.current_line_number(),
                ));
            }

            // Parse value: if start with quote, use parse_value, otherwise use as-is
            let mut value = if (value_str.starts_with('"') && value_str.ends_with('"'))
                || (value_str.starts_with('\'') && value_str.ends_with('\''))
            {
                self.parse_value(value_str)?.to_string()
            } else {
                value_str.to_string()
            };

            // Execute command substitutions $(...) if present
            value = self.execute_command_substitutions(&value, self.current_line_number())?;

            self.current_index += 1;
            Ok(Constant { name, value })
        } else {
            Err(ParseError::InvalidSyntax(
                format!(
                    "Invalid constant syntax. Expected: const NAME = value, got: {}",
                    trimmed
                ),
                self.current_line_number(),
            ))
        }
    }

    fn parse_env_keyword(&mut self, variables: &mut Vec<Variable>) -> Result<(), ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile(self.current_line_number()));
        }

        let line = &self.lines[self.current_index];
        let trimmed = line.trim();

        // Format: env KEY = VALUE or env .env
        let env_part = trimmed.strip_prefix("env ").unwrap_or("").trim();

        if let Some(eq_pos) = env_part.find('=') {
            // env KEY = VALUE
            let name = env_part[..eq_pos].trim().to_string();
            let value_str = env_part[eq_pos + 1..].trim();

            if name.is_empty() {
                return Err(ParseError::InvalidSyntax(
                    "Environment variable name cannot be empty".to_string(),
                    self.current_line_number(),
                ));
            }

            // Parse value
            let mut value = if (value_str.starts_with('"') && value_str.ends_with('"'))
                || (value_str.starts_with('\'') && value_str.ends_with('\''))
            {
                self.parse_value(value_str)?.to_string()
            } else {
                value_str.to_string()
            };

            // Execute command substitutions
            value = self.execute_command_substitutions(&value, self.current_line_number())?;

            // Add to variables (env variables are stored as variables with ENV_ prefix internally sometimes, 
            // but here we might just want to store them so execution logic can find them.
            // Actually, Nest stores env in a separate place in Directive, but global env
            // might be stored as special variables or we might need a GlobalEnv in ParseResult.
            // Let's check ParseResult structure again.)
            
            // Re-checking ParseResult: it doesn't have a global env. 
            // Previous code likely didn't support global env outside of commands except via @env in some versions.
            // Looking at parser.rs, @env wasn't in the main loop before my change.
            // Wait, did the previous version support @env at top level?
            // No, only @var, @const, @function.
            
            // So I should probably add 'env' to ParseResult or handle it by adding to variables with a prefix.
            // But the user wants 'env' to be special.
            
            // Let's look at Directive::Env. It's used inside commands.
            // For global 'env', we could add it to a new field in ParseResult.
            
            // For now, I'll store it in variables with an internal prefix or just add GlobalEnv to ParseResult.
            // Wait, the user said: "var, const, and env variables are automatically exported... while {{}} is reserved for Nest-level composition".
            // So 'env' at top level is just a special kind of 'var' that is EXPECTED to be an environment variable.
            
            // I'll add it as a variable for now, and later ensure 'nest' handles it.
            variables.retain(|v| v.name != name);
            variables.push(Variable { name, value });
            self.current_index += 1;
            Ok(())
        } else {
            // env .env (load from file)
            // We'll store this as a special variable or directive.
            // For now, let's just increment index and maybe store as a special var.
            // Actually, we should probably handle this in include.rs or similar.
            // But let's just store it as a variable with a special name for now.
            variables.push(Variable { 
                name: format!("__env_file_{}", self.current_line_number()), 
                value: env_part.to_string() 
            });
            self.current_index += 1;
            Ok(())
        }
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile(self.current_line_number()));
        }

        let line = &self.lines[self.current_index];
        let indent = get_indent_size(line);
        let trimmed = line.trim();

        // Format: @function name(params):
        // Extract function name and parameters from "@function name(params):"
        let func_part = trimmed.strip_prefix("@function ").unwrap_or("").trim();

        // Parse function signature manually
        let (name, parameters) = if func_part.contains('(') {
            // Has parameters
            let open_paren = func_part.find('(').unwrap();
            let name = func_part[..open_paren].trim().to_string();

            // Find closing parenthesis
            if let Some(close_paren) = func_part.rfind(')') {
                let params_str = &func_part[open_paren + 1..close_paren];
                let parameters = if params_str.trim().is_empty() {
                    Vec::new()
                } else {
                    self.parse_parameters(params_str, self.current_line_number())?
                };
                self.current_index += 1;
                (name, parameters)
            } else {
                return Err(ParseError::InvalidSyntax(
                    "Missing closing parenthesis in function signature".to_string(),
                    self.current_line_number(),
                ));
            }
        } else {
            // No parameters
            let name = if func_part.ends_with(':') {
                func_part[..func_part.len() - 1].trim().to_string()
            } else {
                func_part.to_string()
            };
            self.current_index += 1;
            (name, Vec::new())
        };

        // Parse function body (similar to command parsing)
        let mut body_lines = Vec::new();
        let mut local_variables = Vec::new();

        while self.current_index < self.lines.len() {
            let next_line = self.lines[self.current_index].clone();
            let next_indent = get_indent_size(&next_line);
            let next_trimmed = next_line.trim();

            // If indent is less or equal, we're done with this function
            if next_indent <= indent && !next_trimmed.is_empty() {
                break;
            }

            // Skip empty lines and comments
            if next_trimmed.is_empty() || next_trimmed.starts_with('#') {
                self.current_index += 1;
                continue;
            }

            // Check for local variable definition
            if next_trimmed.starts_with("@var ") {
                let var = self.parse_variable()?;
                local_variables.retain(|v: &Variable| v.name != var.name);
                local_variables.push(var);
                continue;
            }

            // Everything else is part of the function body
            // Remove indentation from the line
            let body_line = if next_indent > indent {
                &next_line[indent as usize..]
            } else {
                &next_line
            };
            body_lines.push(body_line.to_string());
            self.current_index += 1;
        }

        let body = body_lines.join("\n");

        Ok(Function {
            name,
            parameters,
            body,
            local_variables,
        })
    }

    fn parse_multiline_block(&mut self, base_indent: u8) -> Result<String, ParseError> {
        let mut content = String::new();
        let start_line = self.current_line_number();
        self.current_index += 1; // Move past the "> script: |" line

        while self.current_index < self.lines.len() {
            let line = &self.lines[self.current_index];
            let line_indent = get_indent_size(line);
            let trimmed = line.trim();

            // If indent is less or equal to base, block is finished
            if line_indent <= base_indent && !trimmed.is_empty() {
                break;
            }

            // Empty line at base level also ends the block
            if line_indent == base_indent && trimmed.is_empty() {
                break;
            }

            // Add line to content
            if !content.is_empty() {
                content.push('\n');
            }

            // Remove the base indent + one level (4 spaces) from content
            let content_line = if line_indent > base_indent {
                let spaces_to_remove = (base_indent + INDENT_SIZE) as usize;
                if line.len() > spaces_to_remove {
                    &line[spaces_to_remove..]
                } else {
                    line
                }
            } else {
                line
            };

            content.push_str(content_line);
            self.current_index += 1;
        }

        // Validate that multiline block is not empty
        if content.trim().is_empty() {
            return Err(ParseError::InvalidSyntax(
                format!("Multiline script block is empty. Add script content after '|' or use single-line format without '|'."),
                start_line
            ));
        }

        Ok(content)
    }

    /// Executes command substitutions in the format $(command) and replaces them with command output.
    ///
    /// This function finds all occurrences of $(...) in the string and executes the commands,
    /// replacing the $(...) with the command output (trimmed of whitespace).
    ///
    /// # Arguments
    ///
    /// * `value` - The string value that may contain $(...) command substitutions
    /// * `line_number` - The line number for error reporting
    ///
    /// # Returns
    ///
    /// Returns the string with all $(...) replaced by command outputs, or an error if command execution fails.
    ///
    /// # Example
    ///
    /// ```
    /// // Input: "Path: $(which python)"
    /// // Output: "Path: /usr/bin/python"
    /// ```
    fn execute_command_substitutions(
        &self,
        value: &str,
        line_number: usize,
    ) -> Result<String, ParseError> {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        let mut in_substitution = false;
        let mut command = String::new();
        let mut paren_depth = 0;

        while let Some(ch) = chars.next() {
            if !in_substitution {
                if ch == '$' {
                    if let Some(&'(') = chars.peek() {
                        // Found $(
                        chars.next(); // consume '('
                        in_substitution = true;
                        paren_depth = 1;
                        command.clear();
                        continue;
                    }
                }
                result.push(ch);
            } else {
                // We're inside $(...)
                match ch {
                    '(' => {
                        paren_depth += 1;
                        command.push(ch);
                    }
                    ')' => {
                        paren_depth -= 1;
                        if paren_depth == 0 {
                            // End of substitution, execute command
                            let output = self.execute_command(&command.trim(), line_number)?;
                            result.push_str(&output);
                            in_substitution = false;
                            command.clear();
                        } else {
                            command.push(ch);
                        }
                    }
                    _ => {
                        command.push(ch);
                    }
                }
            }
        }

        // If we're still in a substitution at the end, it's an error
        if in_substitution {
            return Err(ParseError::InvalidSyntax(
                format!("Unclosed command substitution $(...) in value: {}", value),
                line_number,
            ));
        }

        Ok(result)
    }

    /// Executes a shell command and returns its output as a string.
    ///
    /// # Arguments
    ///
    /// * `command` - The shell command to execute
    /// * `line_number` - The line number for error reporting
    ///
    /// # Returns
    ///
    /// Returns the command output (stdout) trimmed of whitespace, or an error if execution fails.
    fn execute_command(&self, command: &str, line_number: usize) -> Result<String, ParseError> {
        let output = if cfg!(windows) {
            // On Windows, use cmd /c
            ProcessCommand::new("cmd").arg("/c").arg(command).output()
        } else {
            // On Unix-like systems, use sh -c
            ProcessCommand::new("sh").arg("-c").arg(command).output()
        }
        .map_err(|e| {
            ParseError::InvalidSyntax(
                format!("Failed to execute command '{}': {}", command, e),
                line_number,
            )
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ParseError::InvalidSyntax(
                format!(
                    "Command '{}' failed with exit code {}: {}",
                    command,
                    output.status.code().unwrap_or(-1),
                    stderr.trim()
                ),
                line_number,
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }
}

fn get_indent_size(line: &str) -> u8 {
    let mut spaces = 0;
    for ch in line.chars() {
        if ch == ' ' {
            spaces += 1;
        } else {
            break;
        }
    }
    spaces / INDENT_SIZE
}

impl Parser {
    /// Parses dependencies from a depends directive value.
    ///
    /// Supports syntax like:
    /// - `clean` - simple dependency
    /// - `build(target="x86_64")` - dependency with named argument
    /// - `test(coverage=true)` - dependency with boolean argument
    /// - `build(target="x86_64", release=true)` - multiple arguments
    /// - `dev:build(target="x86_64")` - nested command with arguments
    fn parse_env_directive_keyword(&mut self) -> Result<Directive, ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile(self.current_line_number()));
        }

        let line = &self.lines[self.current_index];
        let trimmed = line.trim();

        // Format: env KEY = VALUE
        let env_part = trimmed.strip_prefix("env ").unwrap_or("").trim();

        if let Some(eq_pos) = env_part.find('=') {
            let name = env_part[..eq_pos].trim().to_string();
            let value_str = env_part[eq_pos + 1..].trim();

            if name.is_empty() {
                return Err(ParseError::InvalidSyntax(
                    "Environment variable name cannot be empty".to_string(),
                    self.current_line_number(),
                ));
            }

            // Parse value
            let value = self.parse_value(value_str)?.to_string();
            self.current_index += 1;
            Ok(Directive::Env(name, value, false))
        } else {
            // env .env
            let value = env_part.to_string();
            self.current_index += 1;
            Ok(Directive::EnvFile(value, false))
        }
    }

    fn parse_property_directive(&mut self, line: &str, indent: u8) -> Result<Directive, ParseError> {
        let trimmed = line.trim();
        
        let colon_pos = trimmed.find(':').ok_or_else(|| {
            ParseError::InvalidSyntax(
                format!("Invalid directive format. Expected 'key: value', got: {}", trimmed),
                self.current_line_number(),
            )
        })?;

        let key_with_mods = trimmed[..colon_pos].trim();
        let value_str = trimmed[colon_pos + 1..].trim();

        // Handle dot-notation modifiers: script.hide: ...
        let key_parts: Vec<&str> = key_with_mods.split('.').collect();
        let key = key_parts[0];
        let hide = key_parts.contains(&"hide");

        // Common directives
        match key {
            "desc" => {
                let value = self.parse_value(value_str)?.to_string();
                Ok(Directive::Desc(value))
            }
            "script" => {
                let script = if value_str == "|" {
                    self.parse_multiline_block(indent)?
                } else {
                    value_str.to_string()
                };
                if hide {
                    Ok(Directive::ScriptHide(script))
                } else {
                    Ok(Directive::Script(script))
                }
            }
            "before" => {
                let script = if value_str == "|" {
                    self.parse_multiline_block(indent)?
                } else {
                    value_str.to_string()
                };
                if hide {
                    Ok(Directive::BeforeHide(script))
                } else {
                    Ok(Directive::Before(script))
                }
            }
            "after" => {
                let script = if value_str == "|" {
                    self.parse_multiline_block(indent)?
                } else {
                    value_str.to_string()
                };
                if hide {
                    Ok(Directive::AfterHide(script))
                } else {
                    Ok(Directive::After(script))
                }
            }
            "finally" => {
                let script = if value_str == "|" {
                    self.parse_multiline_block(indent)?
                } else {
                    value_str.to_string()
                };
                if hide {
                    Ok(Directive::FinallyHide(script))
                } else {
                    Ok(Directive::Finally(script))
                }
            }
            "depends" => {
                let is_parallel = key_parts.contains(&"parallel");
                let deps = self.parse_dependencies(value_str)?;
                Ok(Directive::Depends(deps, is_parallel))
            }
            "fallback" => {
                let script = if value_str == "|" {
                    self.parse_multiline_block(indent)?
                } else {
                    value_str.to_string()
                };
                if hide {
                    Ok(Directive::FallbackHide(script))
                } else {
                    Ok(Directive::Fallback(script))
                }
            }
            "env" => {
                if let Some(eq_pos) = value_str.find('=') {
                     let name = value_str[..eq_pos].trim().to_string();
                     let val = self.parse_value(value_str[eq_pos + 1..].trim())?.to_string();
                     Ok(Directive::Env(name, val, hide))
                } else {
                     Ok(Directive::EnvFile(value_str.to_string(), hide))
                }
            }
            _ => {
                Err(ParseError::InvalidSyntax(
                    format!("Unknown directive property: {}", key),
                    self.current_line_number(),
                ))
            }
        }.map(|d| {
            if !value_str.starts_with('|') {
                self.current_index += 1;
            }
            d
        })
    }

    fn parse_import(&mut self) -> Result<(), ParseError> {
        self.current_index += 1;
        Ok(())
    }

    fn parse_dependencies(&self, value: &str) -> Result<Vec<Dependency>, ParseError> {
        let mut dependencies = Vec::new();
        let mut current = value.trim();

        while !current.is_empty() {
            // Find the next dependency (accounting for parentheses and quotes)
            let (dep_str, remainder) = self.split_next_dependency(current)?;

            if dep_str.is_empty() {
                break;
            }

            let dep = self.parse_single_dependency(dep_str.trim())?;
            dependencies.push(dep);

            current = remainder.trim();
        }

        Ok(dependencies)
    }

    /// Splits the next dependency from the string, handling nested parentheses and quotes.
    fn split_next_dependency<'a>(&self, s: &'a str) -> Result<(&'a str, &'a str), ParseError> {
        let mut depth = 0;
        let mut in_quotes = false;
        let mut quote_char = '\0';
        let mut start = 0;

        // Skip leading whitespace
        while start < s.len() && s.chars().nth(start).unwrap().is_whitespace() {
            start += 1;
        }

        for (i, ch) in s.char_indices() {
            if i < start {
                continue;
            }

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
                }
                ',' if !in_quotes && depth == 0 => {
                    // Found a top-level comma, split here
                    return Ok((&s[start..i], &s[i + 1..]));
                }
                _ => {}
            }
        }

        // No comma found, this is the last dependency
        if start < s.len() {
            Ok((&s[start..], ""))
        } else {
            Ok(("", ""))
        }
    }

    /// Parses a single dependency string into a Dependency struct.
    fn parse_single_dependency(&self, dep_str: &str) -> Result<Dependency, ParseError> {
        // Check if dependency has arguments
        if let Some(open_paren) = dep_str.find('(') {
            let command_path = dep_str[..open_paren].trim().to_string();

            // Find matching closing parenthesis
            let mut depth = 0;
            let mut in_quotes = false;
            let mut quote_char = '\0';
            let mut close_paren = None;

            for (i, ch) in dep_str[open_paren..].char_indices() {
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

            let close_paren = close_paren.ok_or_else(|| {
                ParseError::InvalidSyntax(
                    format!("Unclosed parentheses in dependency: {}", dep_str),
                    self.current_line_number(),
                )
            })?;

            let args_str = &dep_str[open_paren + 1..close_paren];
            let args = self.parse_dependency_args(args_str)?;

            Ok(Dependency { command_path, args })
        } else {
            // No arguments
            Ok(Dependency {
                command_path: dep_str.to_string(),
                args: HashMap::new(),
            })
        }
    }

    /// Parses arguments from a dependency argument string.
    /// Format: `name="value", name2=true, name3=123`
    fn parse_dependency_args(&self, args_str: &str) -> Result<HashMap<String, String>, ParseError> {
        let mut args = HashMap::new();

        if args_str.trim().is_empty() {
            return Ok(args);
        }

        // Split by comma, but respect quotes
        let mut current = args_str.trim();
        while !current.is_empty() {
            let (arg_str, remainder) = self.split_next_arg(current)?;

            if arg_str.is_empty() {
                break;
            }

            // Parse name=value
            let equals_pos = arg_str.find('=').ok_or_else(|| {
                ParseError::InvalidSyntax(
                    format!("Invalid argument format (expected name=value): {}", arg_str),
                    self.current_line_number(),
                )
            })?;

            let name = arg_str[..equals_pos].trim().to_string();
            let value_str = arg_str[equals_pos + 1..].trim();

            // Parse value (string, bool, or number)
            let value = self.parse_dependency_value(value_str)?;

            args.insert(name, value);

            current = remainder.trim();
        }

        Ok(args)
    }

    /// Splits the next argument from the string, handling quotes.
    fn split_next_arg<'a>(&self, s: &'a str) -> Result<(&'a str, &'a str), ParseError> {
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

    /// Parses a dependency argument value (string, bool, or number).
    fn parse_dependency_value(&self, value_str: &str) -> Result<String, ParseError> {
        let trimmed = value_str.trim();

        // String value (quoted)
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            // Remove quotes
            let unquoted = &trimmed[1..trimmed.len() - 1];
            // Unescape quotes
            let unescaped = unquoted
                .replace("\\\"", "\"")
                .replace("\\'", "'")
                .replace("\\\\", "\\");
            Ok(unescaped)
        }
        // Boolean value
        else if trimmed == "true" || trimmed == "false" {
            Ok(trimmed.to_string())
        }
        // Number value
        else if trimmed.parse::<f64>().is_ok() {
            Ok(trimmed.to_string())
        }
        // Unquoted string (treat as string)
        else {
            Ok(trimmed.to_string())
        }
    }

}
