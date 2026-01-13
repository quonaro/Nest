//! Include directive processing for Nestfile.
//!
//! This module handles the `@include` directive which allows including
//! commands from other files or directories.

use super::file::read_file_unchecked;
use super::path::is_config_file;
use super::parser::Parser;
use super::codegen;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;

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
            let rest = trimmed[9..].trim(); // Skip "@include "
            
            // Check for "from" syntax: ... from cmd1, cmd2
            let (rest_before_from, from_clause) = if let Some(from_pos) = rest.rfind(" from ") {
                let before = rest[..from_pos].trim();
                let after = rest[from_pos + 6..].trim();
                (before, Some(after))
            } else {
                (rest, None)
            };

            // Check for "into" syntax: @include path/to/file into group_name
            let (include_path_str, into_group) = if let Some(into_pos) = rest_before_from.find(" into ") {
                let path_part = rest_before_from[..into_pos].trim();
                let group_part = rest_before_from[into_pos + 6..].trim();
                (path_part, Some(group_part))
            } else {
                (rest_before_from, None)
            };
            
            if include_path_str.is_empty() {
                return Err(IncludeError::InvalidPath("Empty include path".to_string()));
            }

            // Parse filter list if present
            let filter_commands: Option<Vec<&str>> = from_clause.map(|s| {
                s.split(',')
                    .map(|cmd| cmd.trim())
                    .filter(|cmd| !cmd.is_empty())
                    .collect()
            });
            let filter_slice = filter_commands.as_deref();

            // Resolve the include path
            let include_path = base_dir.join(include_path_str);
            
            // Process the include
            let included_content = match resolve_and_load_include(&include_path, base_dir, visited, filter_slice)? {
                Some(content) => content,
                None => {
                    // Include path didn't match any files, skip this include
                    continue;
                }
            };

            // Add the included content
            if let Some(group_name) = into_group {
                // If importing into a group, we need to:
                // 1. Create the group command
                // 2. Indent the included content
                
                if group_name.is_empty() {
                    return Err(IncludeError::InvalidPath("Empty group name in 'into' clause".to_string()));
                }
                
                // Add group definition
                result.push_str(line.split("@include").next().unwrap_or("")); // Preserve original indentation
                result.push_str(group_name);
                result.push_str(":\n");
                
                // Indent content
                for content_line in included_content.lines() {
                    // Preserve original indentation of the @include line for the content
                    let base_indent = line.split("@include").next().unwrap_or("");
                    result.push_str(base_indent);
                    result.push_str("    "); // Add 4 spaces indentation
                    result.push_str(content_line);
                    result.push('\n');
                }
            } else {
                // Regular include, just append content
                result.push_str(&included_content);
                result.push('\n');
            }
        } else {
            // Regular line, add it as-is
            result.push_str(line);
            result.push('\n');
        }
    }

    visited.remove(&normalized_base);
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
    filter: Option<&[&str]>,
) -> Result<Option<String>, IncludeError> {
    // Check if it's a wildcard pattern
    let path_str = include_path.to_string_lossy();
    
    if path_str.contains('*') {
        // Pattern matching: app2/*.nest
        return load_pattern_files(include_path, base_dir, visited, filter);
    }

    // Check for remote URL
    if path_str.starts_with("http://") || path_str.starts_with("https://") {
        return load_remote_file(&path_str, visited, filter);
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
        
        return load_directory_files(&dir_path, visited, filter);
    }

    // Specific file: app1/nestfile
    let file_path = if include_path.is_absolute() {
        include_path.to_path_buf()
    } else {
        base_dir.join(include_path)
    };

    // Check if path exists and is a file
    if file_path.exists() && file_path.is_file() {
        return load_single_file(&file_path, visited, filter);
    }

    // If path doesn't exist or is a directory, try to resolve it
    if file_path.is_dir() {
        return load_directory_files(&file_path, visited, filter);
    }

    // If path doesn't have extension, try to find a config file
    if file_path.extension().is_none() {
        // Try to find a config file with this name
        if let Some(file_name) = file_path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if is_config_file(name_str) {
                    // It's already a config file name
                    return load_single_file(&file_path, visited, filter);
                }
            }
        }
        
        // Try common config file names
        for config_name in ["nestfile", "Nestfile", "nest", "Nest"] {
            let config_path = file_path.join(config_name);
            if config_path.exists() && config_path.is_file() {
                return load_single_file(&config_path, visited, filter);
            }
        }
        
        // If the path itself doesn't exist, try it as a directory
        if !file_path.exists() {
            return load_directory_files(&file_path, visited, filter);
        }
    }

    // File doesn't exist
    Err(IncludeError::NotFound(format!(
        "File not found: {}",
        file_path.display()
    )))
}

/// Loads content from a remote URL.
fn load_remote_file(
    url: &str,
    visited: &mut std::collections::HashSet<PathBuf>,
    filter: Option<&[&str]>,
) -> Result<Option<String>, IncludeError> {
    // Use the URL as the "canonical path" for cycle detection
    // We treat the URL string as a dummy PathBuf
    let url_path = PathBuf::from(url);
    
    if visited.contains(&url_path) {
        return Err(IncludeError::CircularInclude(url.to_string()));
    }

    // Fetch the content
    let content = match ureq::get(url).call() {
        Ok(response) => {
            if response.status() != 200 {
                return Err(IncludeError::IoError(format!(
                    "Failed to fetch remote include {}: status {}", 
                    url, 
                    response.status()
                )));
            }
            let mut body = String::new();
            response.into_body().as_reader().read_to_string(&mut body)
                .map_err(|e| IncludeError::IoError(format!("Failed to read remote content: {}", e)))?;
            body
        },
        Err(e) => return Err(IncludeError::IoError(format!("Failed to fetch remote include {}: {}", url, e))),
    };

    // Recursively process includes in the included file
    // Note: Relative includes in a remote file will fail because we can't resolve them easily
    // unless we track the base URL. For now, we assume remote files only have absolute includes or no includes.
    // If we want to support relative includes in remote files, we need to pass a "Base URI" instead of PathBuf.
    // Given the current architecture uses PathBuf, let's pass a dummy path but maybe warn about relative includes?
    // Actually, process_includes takes a Path. If we pass the URL as Path, resolving relative paths will likely fail 
    // or produce weird paths. 
    // Ideally, we shouldn't allow relative includes in remote files unless we properly resolve URLs.
    // For simplicity: treat remote url as a "file" in current dir? No, that breaks relative paths.
    // Let's just process includes but use a dummy base path that indicates it's remote.
    
    // We can't really support relative includes inside remote files without a significant refactor 
    // to separate Filesystem vs URL resolution. 
    // For now, let's just process it with the current directory as base, which means relative includes 
    // will look in the local file system. This is probably "safe" but maybe not what user expects.
    // Let's rely on absolute includes or remote includes inside remote files.
    
    // Actually, let's just reuse the current logic, but mark visited.
    // We use a dummy path for "base_path" so process_includes doesn't crash?
    // process_includes uses canonicalize() on base_path. Attempting to canonicalize a URL or non-existent path will fail.
    
    // HACK: To avoid refactoring everything, let's skip recursive include processing for remote files for now,
    // OR we can create a temporary file? No, that's messy.
    // Let's check process_includes implementation again.
    // It canonicalizes base_path.
    
    // If we simply return the content without recursive processing, then `@include` inside remote file won't work.
    // This is acceptable for a first version.
    
    // If a filter is provided, parse and filter the commands
    let final_content = if let Some(filter_paths) = filter {
        let mut parser = Parser::new(&content);
        let parse_result = parser.parse()
            .map_err(|e| IncludeError::InvalidPath(format!("Error parsing remote included file for filtering: {:?}", e)))?;
        
        let filtered_commands = filter_commands(parse_result.commands, filter_paths)
            .map_err(|e| IncludeError::InvalidPath(e))?;

        let mut filtered_content = String::new();
        for cmd in filtered_commands {
            filtered_content.push_str(&codegen::to_nestfile_string(&cmd, 0));
            filtered_content.push('\n');
        }
        filtered_content
    } else {
        content
    };

    let mut result = String::new();
    result.push_str(&format!("# @source: {}\n", url));
    result.push_str(&final_content);

    Ok(Some(result))
}

/// Loads content from a single file.
fn load_single_file(
    file_path: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
    filter: Option<&[&str]>,
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
    // Note: We normally don't pass the filter recursively because the filter applies to the *result* of the file.
    let processed_content = process_includes(&content, &canonical_path, visited)?;

    // If a filter is provided, parse and filter the commands
    let final_content = if let Some(filter_paths) = filter {
        let mut parser = Parser::new(&processed_content);
        let parse_result = parser.parse()
            .map_err(|e| IncludeError::InvalidPath(format!("Error parsing included file for filtering: {:?}", e)))?;
        
        let filtered_commands = filter_commands(parse_result.commands, filter_paths)
            .map_err(|e| IncludeError::InvalidPath(e))?;

        let mut filtered_content = String::new();
        for cmd in filtered_commands {
            filtered_content.push_str(&codegen::to_nestfile_string(&cmd, 0));
            filtered_content.push('\n');
        }
        filtered_content
    } else {
        processed_content
    };

    // Add source file marker at the beginning of included content
    // This allows the parser to track which file each command came from
    let mut result = String::new();
    result.push_str(&format!("# @source: {}\n", canonical_path.display()));
    result.push_str(&final_content);

    Ok(Some(result))
}

/// Loads content from files matching a pattern.
fn load_pattern_files(
    pattern_path: &Path,
    base_dir: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
    filter: Option<&[&str]>,
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
                    if let Some(content) = load_single_file(&file_path, visited, filter)? {
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
    filter: Option<&[&str]>,
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
                    if let Some(content) = load_single_file(&file_path, visited, filter)? {
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

/// Helper struct for building the selection tree
#[derive(Debug, Default)]
struct SelectionNode {
    /// Children nodes (subcommands)
    children: std::collections::HashMap<String, SelectionNode>,
    /// Whether this node is explicitly selected as a leaf (implies deep import)
    is_leaf_selection: bool,
}

impl SelectionNode {
    fn insert(&mut self, path_parts: &[&str]) {
        if path_parts.is_empty() {
            self.is_leaf_selection = true;
            return;
        }

        let head = path_parts[0];
        let tail = &path_parts[1..];
        
        self.children
            .entry(head.to_string())
            .or_default()
            .insert(tail);
    }
}

/// Filters a list of commands based on a list of selection paths.
/// 
/// Paths can be:
/// - "cmd" -> Selects "cmd" and all its children (deep)
/// - "group.cmd" -> Selects "group" (structure only) then "cmd" (deep)
fn filter_commands(
    commands: Vec<super::ast::Command>,
    filter_paths: &[&str],
) -> Result<Vec<super::ast::Command>, String> {
    // 1. Build selection tree
    let mut root = SelectionNode::default();
    for path in filter_paths {
        let parts: Vec<&str> = path.split('.').collect();
        root.insert(&parts);
    }

    // 2. Filter recursively
    filter_commands_recursive(commands, &root)
}

fn filter_commands_recursive(
    commands: Vec<super::ast::Command>,
    selection: &SelectionNode,
) -> Result<Vec<super::ast::Command>, String> {
    let mut result = Vec::new();

    for mut cmd in commands {
        // Check if this command is selected
        if let Some(child_selection) = selection.children.get(&cmd.name) {
            // It is selected!
            
            // If explicit leaf selection ("group"), keep it fully (deep import)
            if child_selection.is_leaf_selection {
                // Keep the command as is (with all children)
                // But wait, what if "group" AND "group.sub" are specified?
                // "group" implies everything. "group.sub" implies specific.
                // Union is "everything". So we just take it.
                result.push(cmd);
            } else {
                // It's a partial selection (branch). 
                // We need to filter its children.
                
                // Recursively filter children
                // We must take ownership of children, filter them, and put them back.
                let children = std::mem::take(&mut cmd.children);
                let filtered_children = filter_commands_recursive(children, child_selection)?;
                
                // If no children selected but it was a branch selection, 
                // it implies we wanted something inside but found nothing?
                // Or maybe we want the empty group?
                // User asked: "import a group without nested commands".
                // If I say "from group.nonexistent", I get error?
                // If I say "from group" (leaf) -> Deep.
                // How to get shallow?
                // Currently, if I don't select any children, I get empty children list.
                // So if `child_selection` has no leaf selections in its subtree that match, `filtered_children` is empty.
                // This effectively gives us "Empty Group" if we select partially but match nothing?
                // But `filter_commands_recursive` doesn't error on "not found" except if we want strictness.
                // Let's implement strictness check at top level if needed.
                
                cmd.children = filtered_children;
                result.push(cmd);
            }
        }
    }
    
    // Validation: Did we find everything we asked for?
    // This is tricky with the recursive tree. 
    // Ideally, we should validate that every path in `filter_paths` matched something.
    // For now, let's return what we found. Strict validation can be added if needed.
    
    Ok(result)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::nestparse::ast::{Command, ParamKind};

    fn create_dummy_command(name: &str) -> Command {
        Command {
            name: name.to_string(),
            parameters: vec![],
            directives: vec![],
            children: vec![],
            has_wildcard: false,
            local_variables: vec![],
            local_constants: vec![],
            source_file: None,
        }
    }

    fn create_group(name: &str, children: Vec<Command>) -> Command {
        let mut cmd = create_dummy_command(name);
        cmd.children = children;
        cmd
    }

    #[test]
    fn test_filter_commands_deep() {
        // defined: group1 -> sub1, sub2
        let sub1 = create_dummy_command("sub1");
        let sub2 = create_dummy_command("sub2");
        let group1 = create_group("group1", vec![sub1, sub2]);
        let commands = vec![group1];

        // test: from group1
        let filter = vec!["group1"];
        let result = filter_commands(commands, &filter).expect("Filter failed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "group1");
        assert_eq!(result[0].children.len(), 2); // Should have both children
    }

    #[test]
    fn test_filter_commands_partial() {
        // defined: group1 -> sub1, sub2
        let sub1 = create_dummy_command("sub1");
        let sub2 = create_dummy_command("sub2");
        let group1 = create_group("group1", vec![sub1, sub2]);
        let commands = vec![group1];

        // test: from group1.sub1
        let filter = vec!["group1.sub1"];
        let result = filter_commands(commands, &filter).expect("Filter failed");

        assert_eq!(result[0].children.len(), 1);
        assert_eq!(result[0].children[0].name, "sub1");
    }

    #[test]
    fn test_filter_commands_nested() {
         // defined: group1 -> nested -> deep
        let deep = create_dummy_command("deep");
        let nested = create_group("nested", vec![deep]);
        let group1 = create_group("group1", vec![nested]);
        let commands = vec![group1];

        // test: from group1.nested.deep
        let filter = vec!["group1.nested.deep"];
        let result = filter_commands(commands, &filter).expect("Filter failed");

        let group1_res = &result[0];
        let nested_res = &group1_res.children[0];
        let deep_res = &nested_res.children[0];
        
        assert_eq!(group1_res.name, "group1");
        assert_eq!(nested_res.name, "nested");
        assert_eq!(deep_res.name, "deep");
    }

    #[test]
    fn test_filter_commands_multiple() {
        // defined: group1 -> sub1, sub2; group2 -> sub3
        let sub1 = create_dummy_command("sub1");
        let sub2 = create_dummy_command("sub2");
        let group1 = create_group("group1", vec![sub1, sub2]);
        
        let sub3 = create_dummy_command("sub3");
        let group2 = create_group("group2", vec![sub3]);
        
        let commands = vec![group1, group2];

        // test: from group1.sub1, group2
        let filter = vec!["group1.sub1", "group2"];
        let result = filter_commands(commands, &filter).expect("Filter failed");

        assert_eq!(result.len(), 2);
        
        // Check group1
        let r_group1 = result.iter().find(|c| c.name == "group1").expect("group1 missing");
        assert_eq!(r_group1.children.len(), 1);
        assert_eq!(r_group1.children[0].name, "sub1");

        // Check group2
        let r_group2 = result.iter().find(|c| c.name == "group2").expect("group2 missing");
        assert_eq!(r_group2.children.len(), 1); // Full deep import
        assert_eq!(r_group2.children[0].name, "sub3");
    }
}
