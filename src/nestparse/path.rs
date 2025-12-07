//! Utilities for finding and identifying configuration files.

use crate::constants::CONFIG_NAMES;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;

/// Checks if a file name matches one of the valid configuration file names.
///
/// # Arguments
///
/// * `file_name` - The file name to check
///
/// # Returns
///
/// Returns `true` if the file name is a valid configuration file name,
/// `false` otherwise.
pub fn is_config_file(file_name: &str) -> bool {
    CONFIG_NAMES.iter().any(|&name| name == file_name)
}

/// Searches for a configuration file in the current directory.
///
/// This function looks for files matching the names in `CONFIG_NAMES`
/// (nestfile, Nestfile, nest, Nest) in the current working directory.
///
/// # Returns
///
/// - `Some(path)` - Path to the found configuration file
/// - `None` - No configuration file found, or error reading directory
///
/// # Errors
///
/// Prints error messages to stderr if:
/// - Current directory cannot be determined
/// - Directory cannot be read
pub fn find_config_file() -> Option<PathBuf> {
    let current_dir = match current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            use super::output::OutputFormatter;
            OutputFormatter::error(&format!("Error getting current directory: {}", e));
            return None;
        }
    };

    let entries = match fs::read_dir(&current_dir) {
        Ok(entries) => entries,
        Err(e) => {
            use super::output::OutputFormatter;
            OutputFormatter::error(&format!(
                "Error reading directory {}: {}",
                current_dir.display(),
                e
            ));
            return None;
        }
    };

    for entry in entries.flatten() {
        if entry.file_type().ok()?.is_file() {
            let file_path = entry.path();
            if let Some(file_name) = file_path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    if is_config_file(name_str) {
                        return Some(file_path);
                    }
                }
            }
        }
    }

    None
}
