//! File watcher implementation using the `notify` crate.
//!
//! This module provides functionality to watch files and directories for changes
//! and trigger callbacks. It handles debouncing to prevent multiple triggers
//! for a single change event.

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

/// Configuration for the file watcher.
pub struct WatcherConfig {
    /// List of paths or glob patterns to watch
    pub patterns: Vec<String>,
    /// Duration to wait before triggering the callback (debounce)
    pub debounce_ms: u64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            debounce_ms: 100,
        }
    }
}

/// Runs a watch loop that executes the callback when changes are detected.
///
/// This function blocks until interrupted (Ctrl+C).
///
/// # Arguments
///
/// * `config` - Watcher configuration
/// * `callback` - Function to execute when changes are detected
pub fn run_watch_loop<F>(config: WatcherConfig, mut callback: F) -> Result<(), String>
where
    F: FnMut() -> Result<(), String>,
{
    let (tx, rx) = channel();

    // Create a watcher with default config
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Expand globs and add paths to watcher
    for pattern in &config.patterns {
        for entry in glob::glob(pattern).map_err(|e| format!("Invalid glob pattern '{}': {}", pattern, e))? {
            match entry {
                Ok(path) => {
                    // Watch parent directory for files to handle deletion/recreation better,
                    // or watch the file itself.
                    // For now, let's watch the path recursively if it's a directory,
                    // or the file itself (RecursiveMode::NonRecursive for files is implied).
                    
                    // Simple approach: watch the path.
                    // If it's a file, watch it. If it's a dir, watch recursively.
                    let mode = if path.is_dir() {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    };

                    if let Err(e) = watcher.watch(&path, mode) {
                        eprintln!("Warning: Failed to watch path '{}': {}", path.display(), e);
                    } else {
                        // println!("Watching: {}", path.display());
                    }
                }
                Err(e) => eprintln!("Warning: Glob error: {}", e),
            }
        }
    }

    println!("Watcher started. Waiting for changes...");
    println!("Press Ctrl+C to stop.");

    // Initial run
    if let Err(e) = callback() {
        eprintln!("Error during execution: {}", e);
    }

    // Debounce logic
    let debounce_duration = Duration::from_millis(config.debounce_ms);
    let mut last_event_time = std::time::Instant::now(); 

    loop {
        match rx.recv() {
            Ok(Ok(_event)) => {
                // Simple debounce: if less than X ms passed since last event, ignore
                // ideally we should wait until silence, but immediate debounce is easier for first pass
                let now = std::time::Instant::now();
                if now.duration_since(last_event_time) < debounce_duration {
                    continue;
                }
                last_event_time = now;

                // Clear terminal
                print!("\x1B[2J\x1B[1;1H");
                println!("Change detected. Restarting...");
                
                if let Err(e) = callback() {
                    eprintln!("Error during execution: {}", e);
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {}", e),
            Err(e) => return Err(format!("Watcher channel error: {}", e)),
        }
    }
}
