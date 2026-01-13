//! Implementation of standard lifecycle and diagnostic commands.
//!
//! This module contains handlers for:
//! - `list` (ls): List available commands
//! - `check`: Validate configuration
//! - `doctor`: Diagnose environment issues
//! - `clean`: Remove temporary files
//! - `uninstall`: Remove Nest CLI

use crate::constants::APP_DESCRIPTION;
use super::ast::Command;
use super::output::OutputFormatter;
use super::output::colors;
use std::process;

/// Handles the `--std` flag.
///
/// Prints help for standard lifecycle and diagnostic commands.
pub fn handle_std_help() {
    println!("{}", APP_DESCRIPTION);
    println!();
    println!("Standard Commands (Available without nestfile):");
    println!("  {:<12} {}", "list", "List available commands in current nestfile");
    println!("  {:<12} {}", "check", "Validate configuration file");
    println!("  {:<12} {}", "doctor", "Diagnose environment issues");
    println!("  {:<12} {}", "clean", "Remove temporary files");
    println!("  {:<12} {}", "uninstall", "Uninstall Nest CLI");
    println!("  {:<12} {}", "update", "Update Nest CLI to the latest version");
    println!();
    println!("Flags:");
    println!("  {:<12} {}", "--init", "Initialize a new nestfile");
    println!("  {:<12} {}", "--example", "Download example nestfiles");
    println!("  {:<12} {}", "--version", "Show version");
    println!("  {:<12} {}", "--std", "Show this help message");
}

/// Handles the `check` command.
///
/// Validates the configuration file and prints a success message if valid.
/// The actual validation logic is already performed in `main.rs` before 
/// calling this, so if we reach here, it's valid.
pub fn handle_check(config_path: &std::path::Path) {
    OutputFormatter::success("Configuration file is valid!");
    println!(
        "  {}Path:{} {}",
        OutputFormatter::help_label("Path:"),
        colors::RESET,
        OutputFormatter::path(&config_path.display().to_string())
    );
    // TODO: Add more advanced checks here (unused variables, circular dependencies, etc.)
}

/// Handles the `list` command.
///
/// Lists all available commands in a readable format.
pub fn handle_list(commands: &[Command]) {
    println!("{}", APP_DESCRIPTION);
    println!();
    println!("Available commands:");

    for command in commands {
        print_command(command, 0);
    }
}

fn print_command(command: &Command, indent: usize) {
    let padding = " ".repeat(indent * 2);
    let name = if indent == 0 {
        format!("{}{}{}", colors::BRIGHT_GREEN, command.name, colors::RESET)
    } else {
        format!("{}{}{}", colors::CYAN, command.name, colors::RESET)
    };
    
    // Get description from directives
    let desc = command.directives.iter().find_map(|d| match d {
        super::ast::Directive::Desc(s) => Some(s.clone()),
        _ => None,
    }).unwrap_or_default();

    if !desc.is_empty() {
        println!("{}{} - {}", padding, name, desc);
    } else {
        println!("{}{} ", padding, name);
    }

    for child in &command.children {
        if child.name != "default" {
            print_command(child, indent + 1);
        }
    }
}

/// Handles the `clean` command.
///
/// Removes temporary files created by Nest.
pub fn handle_clean() {
    use std::env;
    use std::fs;

    let mut cleaned_count = 0;
    
    // Clean system temp dir for `nest-update-*`
    let temp_dir = env::temp_dir();
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("nest-update-") {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(_) = fs::remove_dir_all(&path) {
                            println!("Removed: {}", path.display());
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }
    }

    // Clean current dir for `.nest_examples_temp` or `.nest_examples_temp.zip`
    if let Ok(current_dir) = env::current_dir() {
         let targets = [".nest_examples_temp", ".nest_examples_temp.zip", ".nest_examples_temp_extract"];
         for target in targets {
             let path = current_dir.join(target);
             if path.exists() {
                 if path.is_dir() {
                     if let Ok(_) = fs::remove_dir_all(&path) {
                         println!("Removed: {}", path.display());
                         cleaned_count += 1;
                     }
                 } else {
                     if let Ok(_) = fs::remove_file(&path) {
                         println!("Removed: {}", path.display());
                         cleaned_count += 1;
                     }
                 }
             }
         }
    }

    if cleaned_count > 0 {
        OutputFormatter::success(&format!("Cleaned {} items.", cleaned_count));
    } else {
        OutputFormatter::info("Nothing to clean.");
    }
}

/// Handles the `doctor` command.
///
/// Checks for common issues.
pub fn handle_doctor() {


    println!("{}Doctor Check:{}", colors::BRIGHT_CYAN, colors::RESET);
    println!("----------------------------------------");

    // Check OS/Arch
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    println!("OS: {} ({})", os, arch);

    // Check external tools
    check_tool("git");
    check_tool("curl");
    check_tool("wget");
    check_tool("tar");
    check_tool("unzip");

    // Check home dir
    if let Ok(home) = std::env::var("HOME") {
        println!("HOME: {}", home);
        let local_bin = std::path::Path::new(&home).join(".local").join("bin");
        // Check if in PATH
        if let Ok(path_var) = std::env::var("PATH") {
            if path_var.split(':').any(|p| std::path::Path::new(p) == local_bin) {
                println!("PATH: Includes ~/.local/bin {}", check_mark(true));
            } else {
                println!("PATH: Missing ~/.local/bin {}", check_mark(false));
                println!("  {}Tip: Add ~/.local/bin to your PATH{}", colors::YELLOW, colors::RESET);
            }
        }
    } else {
         println!("HOME: Not set {}", check_mark(false));
    }
}

fn check_tool(name: &str) {
    let result = std::process::Command::new(name).arg("--version").output();
    let installed = result.is_ok();
    println!("Tool: {:<10} {}", name, check_mark(installed));
}

fn check_mark(ok: bool) -> String {
    if ok {
        format!("{}[OK]{}", colors::GREEN, colors::RESET)
    } else {
        format!("{}[MISSING]{}", colors::RED, colors::RESET)
    }
}

/// Handles the `uninstall` command.
pub fn handle_uninstall() {
    use std::io::{self, Write};
    
    // Confirm
    print!("Are you sure you want to uninstall Nest CLI? This will remove the binary. [y/N]: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted.");
        return;
    }

    if let Ok(path) = std::env::current_exe() {
        println!("Removing binary: {}", path.display());
         match std::fs::remove_file(&path) {
             Ok(_) => {
                 OutputFormatter::success("Nest CLI uninstalled successfully.");
                 println!("Note: Configuration files were not removed.");
             }
             Err(e) => {
                 OutputFormatter::error(&format!("Failed to remove binary: {}", e));
                 process::exit(1);
             }
         }
    } else {
        OutputFormatter::error("Could not locate executable to remove.");
        process::exit(1);
    }
}
