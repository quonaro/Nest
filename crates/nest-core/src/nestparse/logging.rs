//! File logging functionality.
//!
//! This module handles writing command execution logs to files in various formats (txt, json).

use super::template::{TemplateContext, TemplateProcessor};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

/// Writes a log entry to a file in the specified format.
pub fn write_log_entry(
    log_path: &str,
    log_format: &str,
    command_path: Option<&[String]>,
    args: &HashMap<String, String>,
    result: &Result<(), String>,
) -> Result<(), String> {
    // Process template in log path
    // Log path doesn't need parent args (it's just a path)
    let empty_parent_args: HashMap<String, String> = HashMap::new();
    let processed_path = TemplateProcessor::process(
        log_path,
        args,
        &TemplateContext::default(),
        &empty_parent_args,
        None,
    );

    // Create parent directories if needed
    if let Some(parent) = std::path::Path::new(&processed_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create log directory: {}", e))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&processed_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    let command_name = command_path
        .map(|p| p.join(" "))
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = Utc::now().to_rfc3339();
    let success = result.is_ok();
    let error_msg = result.as_ref().err().map(|e| e.to_string());

    match log_format {
        "json" => {
            let log_entry = json!({
                "timestamp": timestamp,
                "command": command_name,
                "args": args,
                "success": success,
                "error": error_msg,
            });
            writeln!(file, "{}", serde_json::to_string(&log_entry).unwrap())
                .map_err(|e| format!("Failed to write log: {}", e))?;
        }
        "txt" => {
            writeln!(file, "[{}] Command: {}", timestamp, command_name)
                .map_err(|e| format!("Failed to write log: {}", e))?;
            if !args.is_empty() {
                let args_str: Vec<String> =
                    args.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                writeln!(file, "  Args: {}", args_str.join(", "))
                    .map_err(|e| format!("Failed to write log: {}", e))?;
            }
            writeln!(
                file,
                "  Status: {}",
                if success { "SUCCESS" } else { "FAILED" }
            )
            .map_err(|e| format!("Failed to write log: {}", e))?;
            if let Some(err) = error_msg {
                writeln!(file, "  Error: {}", err)
                    .map_err(|e| format!("Failed to write log: {}", e))?;
            }
            writeln!(file).map_err(|e| format!("Failed to write log: {}", e))?;
        }
        _ => {
            return Err(format!("Unknown log format: {}", log_format));
        }
    }

    Ok(())
}
