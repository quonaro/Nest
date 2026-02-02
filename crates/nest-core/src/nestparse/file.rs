//! File reading utilities for configuration files.

use std::fs;
use std::io;
use std::path::Path;
/// Reads the contents of a file without validating the file name.
///
/// This function is used for include directives where files may have
/// any name (e.g., `*.nest` files).
///
/// # Arguments
///
/// * `path` - Path to the file
///
/// # Returns
///
/// - `Ok(content)` - The file contents as a string
/// - `Err(error)` - An I/O error if the file cannot be read
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist
/// - The file cannot be read
pub fn read_file_unchecked(path: &Path) -> io::Result<String> {
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    }
    fs::read_to_string(path)
}
