//! File reading utilities for configuration files.

use super::path::is_config_file;
use std::fs;
use std::io;
use std::path::Path;

/// Reads the contents of a configuration file.
///
/// This function validates that:
/// 1. The file exists
/// 2. The file name matches a valid configuration file name
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// - `Ok(content)` - The file contents as a string
/// - `Err(error)` - An I/O error if the file cannot be read or is invalid
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist
/// - The file name is not a valid configuration file name
/// - The file cannot be read
pub fn read_config_file(path: &Path) -> io::Result<String> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    }
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            if !is_config_file(file_name_str) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Not a config file",
                ));
            }
        }
    }
    fs::read_to_string(path)
}
