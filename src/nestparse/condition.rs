//! Condition parsing and evaluation for if/elif directives.
//!
//! This module handles parsing and evaluating conditional expressions
//! with support for comparison operators (==, !=, <=, >=) and logical operators (&&, ||, !).

use super::ast::{Constant, Variable};
use super::template::TemplateProcessor;
use std::collections::HashMap;

/// Represents a condition that can be evaluated.
#[derive(Debug, Clone)]
enum ConditionExpr {
    /// Comparison: left operand, operator, right operand
    Comparison(String, ComparisonOp, String),
    /// Logical NOT: operand
    Not(Box<ConditionExpr>),
    /// Logical AND: left operand, right operand
    And(Box<ConditionExpr>, Box<ConditionExpr>),
    /// Logical OR: left operand, right operand
    Or(Box<ConditionExpr>, Box<ConditionExpr>),
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq)]
enum ComparisonOp {
    Equal,
    NotEqual,
    LessOrEqual,
    GreaterOrEqual,
}

/// Evaluates a condition expression.
///
/// Supports:
/// - Comparison operators: ==, !=, <=, >=
/// - Logical operators: &&, ||, !
/// - Template variable substitution in operands
///
/// # Arguments
///
/// * `condition` - The condition string to evaluate
/// * `args` - Command arguments for template substitution
/// * `global_variables` - Global variables for template substitution
/// * `global_constants` - Global constants for template substitution
/// * `local_variables` - Local variables for template substitution
/// * `local_constants` - Local constants for template substitution
/// * `parent_variables` - Parent variables for template substitution
/// * `parent_constants` - Parent constants for template substitution
/// * `parent_args` - Parent command arguments for template substitution
///
/// # Returns
///
/// Returns `Ok(true)` if condition is true, `Ok(false)` if false,
/// or `Err(message)` if condition cannot be parsed or evaluated.
pub fn evaluate_condition(
    condition: &str,
    args: &HashMap<String, String>,
    global_variables: &[Variable],
    global_constants: &[Constant],
    local_variables: &[Variable],
    local_constants: &[Constant],
    parent_variables: &[Variable],
    parent_constants: &[Constant],
    parent_args: &HashMap<String, String>,
) -> Result<bool, String> {
    let trimmed = condition.trim();
    if trimmed.is_empty() {
        return Err("Empty condition".to_string());
    }

    // Parse the condition expression
    let expr = parse_condition(trimmed)?;

    // Evaluate the expression
    evaluate_expr(
        &expr,
        args,
        global_variables,
        global_constants,
        local_variables,
        local_constants,
        parent_variables,
        parent_constants,
        parent_args,
    )
}

/// Parses a condition string into a ConditionExpr.
fn parse_condition(condition: &str) -> Result<ConditionExpr, String> {
    // Remove outer parentheses if present
    let mut trimmed = condition.trim();
    while trimmed.starts_with('(') && trimmed.ends_with(')') {
        // Check if parentheses are balanced
        let mut depth = 0;
        let mut found_outer = false;
        for (i, ch) in trimmed.chars().enumerate() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && i < trimmed.len() - 1 {
                        found_outer = false;
                        break;
                    }
                    if depth == 0 && i == trimmed.len() - 1 {
                        found_outer = true;
                    }
                }
                _ => {}
            }
        }
        if found_outer {
            trimmed = &trimmed[1..trimmed.len() - 1].trim();
        } else {
            break;
        }
    }

    // Parse logical operators (lowest precedence first)
    // OR has lowest precedence
    if let Some(pos) = find_logical_operator(trimmed, "||") {
        let left = parse_condition(&trimmed[..pos])?;
        let right = parse_condition(&trimmed[pos + 2..])?;
        return Ok(ConditionExpr::Or(Box::new(left), Box::new(right)));
    }

    // AND has higher precedence than OR
    if let Some(pos) = find_logical_operator(trimmed, "&&") {
        let left = parse_condition(&trimmed[..pos])?;
        let right = parse_condition(&trimmed[pos + 2..])?;
        return Ok(ConditionExpr::And(Box::new(left), Box::new(right)));
    }

    // NOT has highest precedence
    if trimmed.starts_with('!') {
        let operand = parse_condition(&trimmed[1..])?;
        return Ok(ConditionExpr::Not(Box::new(operand)));
    }

    // Parse comparison operators
    if let Some((op, pos)) = find_comparison_operator(trimmed) {
        let op_str = match op {
            ComparisonOp::Equal => "==",
            ComparisonOp::NotEqual => "!=",
            ComparisonOp::LessOrEqual => "<=",
            ComparisonOp::GreaterOrEqual => ">=",
        };
        let left = trimmed[..pos].trim().to_string();
        let right = trimmed[pos + op_str.len()..].trim().to_string();
        return Ok(ConditionExpr::Comparison(left, op, right));
    }

    // If no operator found, treat as boolean value
    let lower = trimmed.to_lowercase();
    if lower == "true" {
        Ok(ConditionExpr::Comparison(
            "true".to_string(),
            ComparisonOp::Equal,
            "true".to_string(),
        ))
    } else if lower == "false" {
        Ok(ConditionExpr::Comparison(
            "false".to_string(),
            ComparisonOp::Equal,
            "false".to_string(),
        ))
    } else {
        Err(format!("Invalid condition: {}", condition))
    }
}

/// Finds a logical operator in the string, respecting parentheses and quotes.
fn find_logical_operator(s: &str, op: &str) -> Option<usize> {
    let mut depth = 0;
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
            '(' if !in_quotes => depth += 1,
            ')' if !in_quotes => depth -= 1,
            _ => {}
        }

        if depth == 0 && !in_quotes && i + op.len() <= s.len() {
            if s[i..].starts_with(op) {
                return Some(i);
            }
        }
    }

    None
}

/// Finds a comparison operator in the string, respecting parentheses and quotes.
/// Returns the operator and its position.
fn find_comparison_operator(s: &str) -> Option<(ComparisonOp, usize)> {
    let operators = [
        ("!=", ComparisonOp::NotEqual),
        ("==", ComparisonOp::Equal),
        ("<=", ComparisonOp::LessOrEqual),
        (">=", ComparisonOp::GreaterOrEqual),
    ];

    let mut depth = 0;
    let mut in_quotes = false;
    let mut quote_char = '\0';

    // Find all operator positions
    let mut positions = Vec::new();

    for (i, ch) in s.char_indices() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            ch if ch == quote_char && in_quotes => {
                in_quotes = false;
            }
            '(' if !in_quotes => depth += 1,
            ')' if !in_quotes => depth -= 1,
            _ => {}
        }

        if depth == 0 && !in_quotes {
            for (op_str, op) in &operators {
                if i + op_str.len() <= s.len() && s[i..].starts_with(op_str) {
                    positions.push((i, *op));
                }
            }
        }
    }

    // Return the first (leftmost) operator
    positions.first().map(|(pos, op)| (*op, *pos))
}

/// Evaluates a condition expression.
fn evaluate_expr(
    expr: &ConditionExpr,
    args: &HashMap<String, String>,
    global_variables: &[Variable],
    global_constants: &[Constant],
    local_variables: &[Variable],
    local_constants: &[Constant],
    parent_variables: &[Variable],
    parent_constants: &[Constant],
    parent_args: &HashMap<String, String>,
) -> Result<bool, String> {
    match expr {
        ConditionExpr::Comparison(left, op, right) => {
            // Process templates in operands
            let left_val = TemplateProcessor::process(
                left,
                args,
                global_variables,
                global_constants,
                local_variables,
                local_constants,
                parent_variables,
                parent_constants,
                parent_args,
            );
            let right_val = TemplateProcessor::process(
                right,
                args,
                global_variables,
                global_constants,
                local_variables,
                local_constants,
                parent_variables,
                parent_constants,
                parent_args,
            );

            // Try to parse as numbers for <= and >=
            let left_num = left_val.trim().parse::<f64>();
            let right_num = right_val.trim().parse::<f64>();

            match op {
                ComparisonOp::Equal => Ok(left_val.trim() == right_val.trim()),
                ComparisonOp::NotEqual => Ok(left_val.trim() != right_val.trim()),
                ComparisonOp::LessOrEqual | ComparisonOp::GreaterOrEqual => {
                    if let (Ok(l), Ok(r)) = (left_num, right_num) {
                        match op {
                            ComparisonOp::LessOrEqual => Ok(l <= r),
                            ComparisonOp::GreaterOrEqual => Ok(l >= r),
                            _ => unreachable!(),
                        }
                    } else {
                        Err(format!(
                            "Cannot compare non-numeric values with {}: '{}' and '{}'",
                            match op {
                                ComparisonOp::LessOrEqual => "<=",
                                ComparisonOp::GreaterOrEqual => ">=",
                                _ => unreachable!(),
                            },
                            left_val.trim(),
                            right_val.trim()
                        ))
                    }
                }
            }
        }
        ConditionExpr::Not(operand) => {
            let result = evaluate_expr(
                operand,
                args,
                global_variables,
                global_constants,
                local_variables,
                local_constants,
                parent_variables,
                parent_constants,
                parent_args,
            )?;
            Ok(!result)
        }
        ConditionExpr::And(left, right) => {
            let left_result = evaluate_expr(
                left,
                args,
                global_variables,
                global_constants,
                local_variables,
                local_constants,
                parent_variables,
                parent_constants,
                parent_args,
            )?;
            // Short-circuit evaluation
            if !left_result {
                return Ok(false);
            }
            let right_result = evaluate_expr(
                right,
                args,
                global_variables,
                global_constants,
                local_variables,
                local_constants,
                parent_variables,
                parent_constants,
                parent_args,
            )?;
            Ok(left_result && right_result)
        }
        ConditionExpr::Or(left, right) => {
            let left_result = evaluate_expr(
                left,
                args,
                global_variables,
                global_constants,
                local_variables,
                local_constants,
                parent_variables,
                parent_constants,
                parent_args,
            )?;
            // Short-circuit evaluation
            if left_result {
                return Ok(true);
            }
            let right_result = evaluate_expr(
                right,
                args,
                global_variables,
                global_constants,
                local_variables,
                local_constants,
                parent_variables,
                parent_constants,
                parent_args,
            )?;
            Ok(left_result || right_result)
        }
    }
}
