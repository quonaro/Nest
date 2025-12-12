//! Include directive processing for Nestfile.
//!
//! This module handles the `@include` directive which allows including
//! commands from other files or directories.

use super::file::read_file_unchecked;
use super::path::is_config_file;
use std::fs;
use std::path::{Path, PathBuf};

/// Errors that can occur during include processing.
#[derive(Debug)]
pub enum IncludeError {
    /// File or directory not found
    NotFound(String),
    /// I/O error while reading files
    IoError(String),
    /// Invalid include path
    InvalidPath(String),
    /// Circular include detected
    CircularInclude(String),
}

impl std::fmt::Display for IncludeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncludeError::NotFound(msg) => write!(f, "Include not found: {}", msg),
            IncludeError::IoError(msg) => write!(f, "I/O error: {}", msg),
            IncludeError::InvalidPath(msg) => write!(f, "Invalid include path: {}", msg),
            IncludeError::CircularInclude(msg) => write!(f, "Circular include detected: {}", msg),
        }
    }
}

/// Processes include directives in the content and returns merged content.
///
/// This function:
/// 1. Finds all `@include` directives in the content
/// 2. Resolves paths relative to the base file
/// 3. Loads and merges content from included files
/// 4. Returns the merged content with includes replaced
///
/// # Arguments
///
/// * `content` - The original file content
/// * `base_path` - Path to the base configuration file (for resolving relative paths)
/// * `visited` - Set of already visited files (to detect circular includes)
///
/// # Returns
///
/// - `Ok(merged_content)` - Content with all includes processed
/// - `Err(error)` - Error if include processing fails
pub fn process_includes(
    content: &str,
    base_path: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<String, IncludeError> {
    // Normalize base_path for comparison and path resolution
    let normalized_base = base_path.canonicalize()
        .map_err(|e| IncludeError::IoError(format!("Cannot canonicalize base path: {}", e)))?;
    
    if visited.contains(&normalized_base) {
        return Err(IncludeError::CircularInclude(
            base_path.display().to_string(),
        ));
    }
    visited.insert(normalized_base.clone());

    // Get base directory from canonicalized path
    let base_dir = normalized_base
        .parent()
        .ok_or_else(|| IncludeError::InvalidPath("Base path has no parent".to_string()))?;

    let mut result = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        
        // Check if this is an include directive
        if trimmed.starts_with("@include ") {
            let include_path_str = trimmed[9..].trim(); // Skip "@include "
            
            if include_path_str.is_empty() {
                return Err(IncludeError::InvalidPath("Empty include path".to_string()));
            }

            // Resolve the include path
            let include_path = base_dir.join(include_path_str);
            
            // Process the include
            let included_content = match resolve_and_load_include(&include_path, base_dir, visited)? {
                Some(content) => content,
                None => {
                    // Include path didn't match any files, skip this include
                    continue;
                }
            };

            // Add the included content
            result.push_str(&included_content);
            result.push('\n');
        } else {
            // Regular line, add it as-is
            result.push_str(line);
            result.push('\n');
        }
    }

    Ok(result)
}

/// Resolves an include path and loads the content.
///
/// Handles three types of includes:
/// 1. Specific file: `@include app1/nestfile`
/// 2. Pattern with wildcard: `@include app2/*.nest`
/// 3. Directory: `@include app3/`
///
/// # Arguments
///
/// * `include_path` - The include path to resolve
/// * `base_dir` - Base directory for resolving relative paths
/// * `visited` - Set of visited files for circular dependency detection
///
/// # Returns
///
/// - `Ok(Some(content))` - Content from included files
/// - `Ok(None)` - No files matched (not an error)
/// - `Err(error)` - Error if processing fails
fn resolve_and_load_include(
    include_path: &Path,
    base_dir: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<Option<String>, IncludeError> {
    // Check if it's a wildcard pattern
    let path_str = include_path.to_string_lossy();
    
    if path_str.contains('*') {
        // Pattern matching: app2/*.nest
        return load_pattern_files(include_path, base_dir, visited);
    }

    // Check if it's a directory (ends with /)
    if path_str.ends_with('/') || path_str.ends_with('\\') {
        // Directory include: app3/
        let dir_path = if include_path.is_absolute() {
            include_path.to_path_buf()
        } else {
            base_dir.join(include_path)
        };
        
        // Remove trailing slash
        let dir_path = if dir_path.to_string_lossy().ends_with('/') {
            PathBuf::from(dir_path.to_string_lossy().trim_end_matches('/'))
        } else {
            dir_path
        };
        
        return load_directory_files(&dir_path, visited);
    }

    // Specific file: app1/nestfile
    let file_path = if include_path.is_absolute() {
        include_path.to_path_buf()
    } else {
        base_dir.join(include_path)
    };

    // Check if path exists and is a file
    if file_path.exists() && file_path.is_file() {
        return load_single_file(&file_path, visited);
    }

    // If path doesn't exist or is a directory, try to resolve it
    if file_path.is_dir() {
        return load_directory_files(&file_path, visited);
    }

    // If path doesn't have extension, try to find a config file
    if file_path.extension().is_none() {
        // Try to find a config file with this name
        if let Some(file_name) = file_path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if is_config_file(name_str) {
                    // It's already a config file name
                    return load_single_file(&file_path, visited);
                }
            }
        }
        
        // Try common config file names
        for config_name in ["nestfile", "Nestfile", "nest", "Nest"] {
            let config_path = file_path.join(config_name);
            if config_path.exists() && config_path.is_file() {
                return load_single_file(&config_path, visited);
            }
        }
        
        // If the path itself doesn't exist, try it as a directory
        if !file_path.exists() {
            return load_directory_files(&file_path, visited);
        }
    }

    // File doesn't exist
    Err(IncludeError::NotFound(format!(
        "File not found: {}",
        file_path.display()
    )))
}

/// Loads content from a single file.
fn load_single_file(
    file_path: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<Option<String>, IncludeError> {
    let canonical_path = file_path
        .canonicalize()
        .map_err(|e| IncludeError::NotFound(format!("{}: {}", file_path.display(), e)))?;

    if visited.contains(&canonical_path) {
        return Err(IncludeError::CircularInclude(
            file_path.display().to_string(),
        ));
    }

    // Use unchecked read for include files (they may have any name)
    let content = read_file_unchecked(&canonical_path)
        .map_err(|e| IncludeError::IoError(format!("{}: {}", file_path.display(), e)))?;

    // Recursively process includes in the included file
    let processed_content = process_includes(&content, &canonical_path, visited)?;

    // Add source file marker at the beginning of included content
    // This allows the parser to track which file each command came from
    let mut result = String::new();
    result.push_str(&format!("# @source: {}\n", canonical_path.display()));
    result.push_str(&processed_content);

    Ok(Some(result))
}

/// Loads content from files matching a pattern.
fn load_pattern_files(
    pattern_path: &Path,
    base_dir: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<Option<String>, IncludeError> {
    let pattern_str = pattern_path.to_string_lossy();
    
    // Find the directory and pattern
    let (dir_path, file_pattern) = if let Some(last_slash) = pattern_str.rfind('/') {
        let dir_part = &pattern_str[..last_slash];
        let file_part = &pattern_str[last_slash + 1..];
        (base_dir.join(dir_part), file_part)
    } else if let Some(last_backslash) = pattern_str.rfind('\\') {
        let dir_part = &pattern_str[..last_backslash];
        let file_part = &pattern_str[last_backslash + 1..];
        (base_dir.join(dir_part), file_part)
    } else {
        // No directory, just pattern in current dir
        (base_dir.to_path_buf(), pattern_str.as_ref())
    };

    if !dir_path.exists() {
        return Ok(None); // Directory doesn't exist, not an error
    }

    // Use pattern as-is (matches_pattern handles * directly)
    let pattern = file_pattern;
    
    let mut merged_content = String::new();
    let mut found_any = false;

    let entries = fs::read_dir(&dir_path)
        .map_err(|e| IncludeError::IoError(format!("Cannot read directory {}: {}", dir_path.display(), e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| IncludeError::IoError(format!("Error reading directory entry: {}", e)))?;
        let file_path = entry.path();
        
        if !file_path.is_file() {
            continue;
        }

        if let Some(file_name) = file_path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                // Simple pattern matching (supports * wildcard)
                if matches_pattern(name_str, &pattern) {
                    if let Some(content) = load_single_file(&file_path, visited)? {
                        merged_content.push_str(&content);
                        merged_content.push('\n');
                        found_any = true;
                    }
                }
            }
        }
    }

    if found_any {
        Ok(Some(merged_content))
    } else {
        Ok(None)
    }
}

/// Loads all config files from a directory.
fn load_directory_files(
    dir_path: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<Option<String>, IncludeError> {
    if !dir_path.exists() {
        return Ok(None); // Directory doesn't exist, not an error
    }

    if !dir_path.is_dir() {
        return Err(IncludeError::InvalidPath(format!(
            "Path is not a directory: {}",
            dir_path.display()
        )));
    }

    let mut merged_content = String::new();
    let mut found_any = false;

    let entries = fs::read_dir(dir_path)
        .map_err(|e| IncludeError::IoError(format!("Cannot read directory {}: {}", dir_path.display(), e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| IncludeError::IoError(format!("Error reading directory entry: {}", e)))?;
        let file_path = entry.path();
        
        if !file_path.is_file() {
            continue;
        }

        if let Some(file_name) = file_path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if is_config_file(name_str) {
                    if let Some(content) = load_single_file(&file_path, visited)? {
                        merged_content.push_str(&content);
                        merged_content.push('\n');
                        found_any = true;
                    }
                }
            }
        }
    }

    if found_any {
        Ok(Some(merged_content))
    } else {
        Ok(None)
    }
}

/// Checks if a filename matches a pattern.
///
/// Supports simple wildcard matching where * matches any sequence of characters.
fn matches_pattern(filename: &str, pattern: &str) -> bool {
    // Convert pattern with * to regex-like parts
    // Pattern like "*.nest" becomes ["", ".nest"]
    // Pattern like "test*.nest" becomes ["test", ".nest"]
    
    let parts: Vec<&str> = pattern.split('*').collect();
    
    if parts.len() == 1 {
        // No wildcard, exact match
        return filename == pattern;
    }

    // First part must match the beginning
    if !parts[0].is_empty() && !filename.starts_with(parts[0]) {
        return false;
    }

    // Last part must match the end
    if !parts.last().unwrap().is_empty() && !filename.ends_with(parts.last().unwrap()) {
        return false;
    }

    // For patterns with multiple wildcards, check that all parts appear in order
    if parts.len() > 2 {
        let mut search_start = parts[0].len();
        for part in parts.iter().skip(1).take(parts.len() - 1) {
            if part.is_empty() {
                continue;
            }
            if let Some(pos) = filename[search_start..].find(part) {
                search_start += pos + part.len();
            } else {
                return false;
            }
        }
    }

    true
}

