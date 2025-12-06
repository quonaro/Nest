//! Parser for Nestfile configuration files.
//!
//! This module parses the Nestfile syntax into an Abstract Syntax Tree (AST).
//! It handles nested commands, parameters, directives, and multiline constructs.

use crate::constants::{BOOL_FALSE, BOOL_TRUE, INDENT_SIZE};
use super::ast::{Command, Parameter, Value, Directive};

/// Parser state for processing Nestfile content.
///
/// The parser maintains its position in the file and processes commands
/// recursively based on indentation levels.
pub struct Parser {
    /// All lines of the configuration file
    lines: Vec<String>,
    /// Current position in the file (line index)
    current_index: usize,
}

/// Errors that can occur during parsing.
#[derive(Debug)]
pub enum ParseError {
    /// Unexpected end of file (e.g., incomplete command definition)
    UnexpectedEndOfFile,
    /// Invalid syntax in the configuration file
    #[allow(dead_code)]
    InvalidSyntax(String),
    /// Invalid indentation (e.g., child command not properly indented)
    InvalidIndent,
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
        }
    }

    /// Parses the entire configuration file into a list of commands.
    ///
    /// This is the main entry point for parsing. It processes all top-level
    /// commands and their nested structure.
    ///
    /// # Returns
    ///
    /// Returns `Ok(commands)` with the parsed command structure,
    /// or `Err(error)` if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File structure is invalid
    /// - Indentation is incorrect
    /// - Unexpected end of file
    pub fn parse(&mut self) -> Result<Vec<Command>, ParseError> {
        let mut commands = Vec::new();
        
        while self.current_index < self.lines.len() {
            let line = &self.lines[self.current_index];
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                self.current_index += 1;
                continue;
            }
            
            // Check if it's a command definition (ends with : or contains opening parenthesis but not closing)
            // A line like "):" should not be recognized as a command
            let is_command = !trimmed.starts_with('>') && 
                (trimmed.ends_with(':') || (trimmed.contains('(') && !trimmed.contains(')')));
            
            if is_command {
                let command = self.parse_command(0)?;
                commands.push(command);
            } else {
                self.current_index += 1;
            }
        }
        
        Ok(commands)
    }

    fn parse_command(&mut self, base_indent: u8) -> Result<Command, ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile);
        }

        let line = &self.lines[self.current_index];
        let indent = get_indent_size(line);
        
        if indent < base_indent {
            return Err(ParseError::InvalidIndent);
        }

        // Parse function signature: name(params): (may be multiline)
        let (name, parameters) = self.parse_function_signature_multiline(indent)?;
        
        // current_index already incremented in parse_function_signature_multiline
        
        let mut directives = Vec::new();
        let mut children = Vec::new();
        
        // Parse directives and children
        while self.current_index < self.lines.len() {
            let next_line = self.lines[self.current_index].clone();
            let next_indent = get_indent_size(&next_line);
            let next_trimmed = next_line.trim();
            
            // If indent is less or equal, we're done with this command
            if next_indent <= indent && !next_trimmed.is_empty() {
                break;
            }
            
            // Skip empty lines
            if next_trimmed.is_empty() {
                self.current_index += 1;
                continue;
            }
            
            // Check if it's a directive (> desc:, > env:, etc.)
            if next_trimmed.starts_with('>') {
                let (directive, is_multiline) = self.parse_directive(&next_line, next_indent)?;
                directives.push(directive);
                // Only increment if it's not a multiline block (multiline already increments inside)
                if !is_multiline {
                    self.current_index += 1;
                }
            }
            // Check if it's a child command
            // A line like "):" should not be recognized as a command
            else if !next_trimmed.starts_with('>') && 
                    (next_trimmed.ends_with(':') || (next_trimmed.contains('(') && !next_trimmed.contains(')'))) {
                let child = self.parse_command(indent)?;
                children.push(child);
            }
            else {
                self.current_index += 1;
            }
        }
        
        Ok(Command {
            name,
            parameters,
            directives,
            children,
        })
    }

    fn parse_function_signature_multiline(&mut self, base_indent: u8) -> Result<(String, Vec<Parameter>), ParseError> {
        if self.current_index >= self.lines.len() {
            return Err(ParseError::UnexpectedEndOfFile);
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
                let parameters = if params_str.trim().is_empty() {
                    Vec::new()
                } else {
                    self.parse_parameters(params_str)?
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
                        return Err(ParseError::InvalidSyntax("Missing closing parenthesis in function signature".to_string()));
                    }
                    
                    // Add this line to params
                    params_lines.push(next_trimmed.to_string());
                    self.current_index += 1;
                }
                
                let params_str = params_lines.join(" ");
                let parameters = if params_str.trim().is_empty() {
                    Vec::new()
                } else {
                    self.parse_parameters(&params_str)?
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

    fn parse_parameters(&self, params_str: &str) -> Result<Vec<Parameter>, ParseError> {
        let mut parameters = Vec::new();
        let mut current_param = String::new();
        let mut paren_depth = 0;
        
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
                        parameters.push(self.parse_parameter(current_param.trim())?);
                    }
                    current_param.clear();
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }
        
        if !current_param.trim().is_empty() {
            parameters.push(self.parse_parameter(current_param.trim())?);
        }
        
        Ok(parameters)
    }

    fn parse_parameter(&self, param_str: &str) -> Result<Parameter, ParseError> {
        // Format: [!]name|alias: type = default
        // ! prefix means named argument (uses --name)
        let parts: Vec<&str> = param_str.split(':').collect();
        
        if parts.len() < 2 {
            return Err(ParseError::InvalidSyntax(format!("Invalid parameter: {}", param_str)));
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
                Some(name_part_clean[pipe_pos + 1..].trim().to_string())
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
        })
    }

    fn parse_value(&self, value_str: &str) -> Result<Value, ParseError> {
        let trimmed = value_str.trim();
        
        // String literal
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            let s = trimmed[1..trimmed.len() - 1].to_string();
            return Ok(Value::String(s));
        }
        
        // Boolean
        if trimmed == BOOL_TRUE {
            return Ok(Value::Bool(true));
        }
        if trimmed == BOOL_FALSE {
            return Ok(Value::Bool(false));
        }
        
        // Array
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let content = &trimmed[1..trimmed.len() - 1];
            let items: Vec<String> = content.split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            return Ok(Value::Array(items));
        }
        
        // Number
        if let Ok(num) = trimmed.parse::<f64>() {
            return Ok(Value::Number(num));
        }
        
        // Default to string
        Ok(Value::String(trimmed.to_string()))
    }

    fn parse_directive(&mut self, line: &str, indent: u8) -> Result<(Directive, bool), ParseError> {
        let trimmed = line.trim();
        
        // Remove '>' prefix
        let content = trimmed.strip_prefix('>').unwrap_or(trimmed).trim();
        
        if let Some(colon_pos) = content.find(':') {
            let directive_name = content[..colon_pos].trim();
            let directive_value = content[colon_pos + 1..].trim();
            
            match directive_name {
                "desc" => Ok((Directive::Desc(directive_value.to_string()), false)),
                "cwd" => Ok((Directive::Cwd(directive_value.to_string()), false)),
                "env" => Ok((Directive::Env(directive_value.to_string()), false)),
                "script" => {
                    // Check if it's multiline (ends with |)
                    if directive_value == "|" {
                        // Parse multiline block
                        let script_content = self.parse_multiline_block(indent)?;
                        Ok((Directive::Script(script_content), true))
                    } else {
                        // Single line script
                        Ok((Directive::Script(directive_value.to_string()), false))
                    }
                }
                _ => Err(ParseError::InvalidSyntax(format!("Unknown directive: {}", directive_name)))
            }
        } else {
            Err(ParseError::InvalidSyntax(format!("Invalid directive format: {}", trimmed)))
        }
    }

    fn parse_multiline_block(&mut self, base_indent: u8) -> Result<String, ParseError> {
        let mut content = String::new();
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
        
        Ok(content)
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

