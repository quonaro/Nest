//! Shell completion generation and management.
//!
//! This module handles automatic generation and installation of shell completion scripts
//! for Nest CLI. It provides the following features:
//!
//! ## Features
//!
//! - **Automatic generation**: Completion scripts are automatically generated when nestfile changes
//! - **Multi-shell support**: Supports bash, zsh, fish, PowerShell, and elvish
//! - **Auto-installation**: Automatically installs completion in shell configuration files
//! - **Smart caching**: Uses SHA256 hashing to detect nestfile changes and regenerate only when needed
//! - **Manual control**: Use `nest --complete <shell>` to manually generate and install completion
//!
//! ## Usage
//!
//! ### Automatic Installation
//!
//! Completion is automatically installed when you run any `nest` command. The system:
//! 1. Detects your current shell
//! 2. Generates completion scripts for all supported shells
//! 3. Installs completion in your shell's configuration file
//! 4. Sources the completion in the current terminal session (if possible)
//!
//! ### Manual Installation
//!
//! ```bash
//! # Generate and install completion for current shell
//! nest --complete zsh
//!
//! # View completion script content
//! nest --complete zsh -V
//!
//! # Generate for specific shell
//! nest --complete bash
//! nest --complete fish
//! ```
//!
//! ## File Locations
//!
//! - **Completion scripts**: `~/.cache/nest/completions/`
//! - **Hash file**: `~/.cache/nest/completions/nestfile.hash`
//! - **Shell configs**: Automatically added to `.zshrc`, `.bashrc`, etc.
//!
//! ## Supported Shells
//!
//! - **Bash**: Completion added to `~/.bashrc` or `~/.bash_profile`
//! - **Zsh**: Completion added to `~/.zshrc`
//! - **Fish**: Completion copied to `~/.config/fish/completions/nest.fish`
//! - **PowerShell**: Completion added to PowerShell profile
//! - **Elvish**: Completion added to `~/.elvish/rc.elv`

use crate::constants::APP_NAME;
use clap::Command as ClapCommand;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Shell types supported for completion generation
#[derive(Debug, Clone, Copy)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl Shell {
    /// Parse shell name from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            "powershell" | "ps1" => Some(Shell::PowerShell),
            "elvish" => Some(Shell::Elvish),
            _ => None,
        }
    }

    /// Get shell name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
            Shell::Elvish => "elvish",
        }
    }
}

/// Manages completion script generation and caching
pub struct CompletionManager {
    pub(crate) cache_dir: PathBuf,
}

impl CompletionManager {
    /// Create a new CompletionManager
    pub fn new() -> Result<Self, String> {
        let cache_dir = Self::get_cache_dir()?;
        
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }

        Ok(Self { cache_dir })
    }

    /// Get the cache directory path
    fn get_cache_dir() -> Result<PathBuf, String> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "HOME or USERPROFILE environment variable not set")?;
        
        let mut cache_dir = PathBuf::from(home);
        cache_dir.push(".cache");
        cache_dir.push("nest");
        cache_dir.push("completions");
        
        Ok(cache_dir)
    }

    /// Calculate SHA256 hash of the nestfile content
    fn calculate_file_hash(file_path: &Path) -> Result<String, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file for hashing: {}", e))?;
        
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        
        Ok(format!("{:x}", hash))
    }

    /// Get the path to the hash file for a given nestfile
    fn get_hash_file_path(&self, nestfile_path: &Path) -> PathBuf {
        let file_name = nestfile_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("nestfile");
        
        let mut hash_path = self.cache_dir.clone();
        hash_path.push(format!("{}.hash", file_name));
        hash_path
    }

    /// Get the path to the completion script for a given shell and nestfile
    fn get_completion_script_path(&self, shell: Shell, nestfile_path: &Path) -> PathBuf {
        let file_name = nestfile_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("nestfile");
        
        let mut script_path = self.cache_dir.clone();
        script_path.push(format!("{}.{}", file_name, shell.as_str()));
        script_path
    }

    /// Check if completion script needs to be regenerated
    pub fn needs_regeneration(&self, nestfile_path: &Path) -> Result<bool, String> {
        let current_hash = Self::calculate_file_hash(nestfile_path)?;
        let hash_file_path = self.get_hash_file_path(nestfile_path);
        
        // If hash file doesn't exist, need to regenerate
        if !hash_file_path.exists() {
            return Ok(true);
        }
        
        // Read stored hash
        let stored_hash = fs::read_to_string(&hash_file_path)
            .map_err(|e| format!("Failed to read hash file: {}", e))?;
        
        // Need regeneration if hashes don't match
        Ok(current_hash.trim() != stored_hash.trim())
    }

    /// Generate completion script for a specific shell
    pub fn generate_completion(
        &self,
        shell: Shell,
        cli: &mut ClapCommand,
        nestfile_path: &Path,
    ) -> Result<PathBuf, String> {
        let script_path = self.get_completion_script_path(shell, nestfile_path);
        
        // Generate completion script
        let mut buffer = Vec::new();
        match shell {
            Shell::Bash => {
                clap_complete::generate(clap_complete::shells::Bash, cli, APP_NAME, &mut buffer);
            }
            Shell::Zsh => {
                clap_complete::generate(clap_complete::shells::Zsh, cli, APP_NAME, &mut buffer);
            }
            Shell::Fish => {
                clap_complete::generate(clap_complete::shells::Fish, cli, APP_NAME, &mut buffer);
            }
            Shell::PowerShell => {
                clap_complete::generate(clap_complete::shells::PowerShell, cli, APP_NAME, &mut buffer);
            }
            Shell::Elvish => {
                clap_complete::generate(clap_complete::shells::Elvish, cli, APP_NAME, &mut buffer);
            }
        }
        
        // Write script to file
        fs::write(&script_path, buffer)
            .map_err(|e| format!("Failed to write completion script: {}", e))?;
        
        // Save hash for future comparison
        let current_hash = Self::calculate_file_hash(nestfile_path)?;
        let hash_file_path = self.get_hash_file_path(nestfile_path);
        fs::write(&hash_file_path, current_hash)
            .map_err(|e| format!("Failed to write hash file: {}", e))?;
        
        Ok(script_path)
    }

    /// Generate completion scripts for all supported shells
    pub fn generate_all_completions(
        &self,
        cli: &mut ClapCommand,
        nestfile_path: &Path,
    ) -> Result<Vec<(Shell, PathBuf)>, String> {
        let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell, Shell::Elvish];
        let mut generated = Vec::new();
        
        for shell in &shells {
            // Clone CLI for each shell to avoid borrowing issues
            let mut cli_clone = cli.clone();
            let path = self.generate_completion(*shell, &mut cli_clone, nestfile_path)?;
            generated.push((*shell, path));
        }
        
        Ok(generated)
    }

    /// Handle completion request (for --complete flag)
    pub fn handle_completion_request(
        cli: &mut ClapCommand,
        shell_name: &str,
        verbose: bool,
        nestfile_path: &Path,
    ) -> Result<(), String> {
        let shell = Shell::from_str(shell_name)
            .ok_or_else(|| format!("Unsupported shell: {}. Supported: bash, zsh, fish, powershell, elvish", shell_name))?;
        
        let manager = CompletionManager::new()?;
        
        // Generate completion script if needed
        let script_path = manager.get_completion_script_path(shell, nestfile_path);
        if !script_path.exists() || manager.needs_regeneration(nestfile_path)? {
            manager.generate_completion(shell, cli, nestfile_path)?;
        }
        
        // If verbose, output the script content
        if verbose {
            match shell {
                Shell::Bash => {
                    clap_complete::generate(clap_complete::shells::Bash, cli, APP_NAME, &mut std::io::stdout());
                }
                Shell::Zsh => {
                    clap_complete::generate(clap_complete::shells::Zsh, cli, APP_NAME, &mut std::io::stdout());
                }
                Shell::Fish => {
                    clap_complete::generate(clap_complete::shells::Fish, cli, APP_NAME, &mut std::io::stdout());
                }
                Shell::PowerShell => {
                    clap_complete::generate(clap_complete::shells::PowerShell, cli, APP_NAME, &mut std::io::stdout());
                }
                Shell::Elvish => {
                    clap_complete::generate(clap_complete::shells::Elvish, cli, APP_NAME, &mut std::io::stdout());
                }
            }
            return Ok(());
        }
        
        // Otherwise, show informational message and install
        Self::print_completion_info(&manager, shell, &script_path, nestfile_path)?;
        
        // Try to install automatically
        if let Ok(true) = Self::install_completion(shell, &script_path) {
            // Installation successful, try to source in current terminal
            Self::source_completion_in_current_shell(shell, &script_path)?;
        }
        
        Ok(())
    }
    
    /// Print informational message about completion installation
    fn print_completion_info(
        manager: &CompletionManager,
        shell: Shell,
        script_path: &Path,
        nestfile_path: &Path,
    ) -> Result<(), String> {
        use crate::nestparse::output::OutputFormatter;
        
        OutputFormatter::info("Shell completion information:");
        println!();
        println!("  Shell: {}", shell.as_str());
        println!("  Completion script: {}", script_path.display());
        println!("  Based on nestfile: {}", nestfile_path.display());
        println!();
        
        // Show all generated completion scripts
        let cache_dir = manager.cache_dir.clone();
        if cache_dir.exists() {
            println!("  All generated completion scripts:");
            let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell, Shell::Elvish];
            for s in &shells {
                let path = manager.get_completion_script_path(*s, nestfile_path);
                if path.exists() {
                    println!("    {}: {}", s.as_str(), path.display());
                }
            }
            println!();
        }
        
        // Show installation status
        if Self::is_completion_installed(shell, script_path) {
            let config_path = Self::get_shell_config_path(shell)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            OutputFormatter::info(&format!("  ✓ Completion already installed in: {}", config_path));
        } else {
            OutputFormatter::info("  → Completion will be installed automatically");
        }
        println!();
        
        // Show instructions
        println!("  To view the completion script, use:");
        println!("    nest --complete {} -V", shell.as_str());
        println!();
        
        Ok(())
    }
    
    /// Source completion script in current shell (if possible)
    fn source_completion_in_current_shell(shell: Shell, script_path: &Path) -> Result<(), String> {
        // Only try to source for bash/zsh in interactive shells
        match shell {
            Shell::Bash | Shell::Zsh => {
                // Check if we're in an interactive shell
                if std::env::var("PS1").is_ok() || std::env::var("ZSH").is_ok() {
                    // Try to source the script
                    let source_cmd = format!("source {}", script_path.display());
                    let output = std::process::Command::new(shell.as_str())
                        .arg("-c")
                        .arg(&source_cmd)
                        .output();
                    
                    if let Ok(_) = output {
                        use crate::nestparse::output::OutputFormatter;
                        OutputFormatter::info("  ✓ Completion loaded in current shell session");
                    }
                }
            }
            _ => {
                // For other shells, just inform user
                use crate::nestparse::output::OutputFormatter;
                OutputFormatter::info(&format!("  → Restart your {} shell or run: source {}", shell.as_str(), script_path.display()));
            }
        }
        Ok(())
    }

    /// Get instructions for setting up completion in user's shell
    #[allow(dead_code)]
    pub fn get_setup_instructions(shell: Shell, script_path: &Path) -> String {
        match shell {
            Shell::Bash => {
                format!(
                    "Add to your ~/.bashrc:\n  source {}\nThen reload: source ~/.bashrc",
                    script_path.display()
                )
            }
            Shell::Zsh => {
                format!(
                    "Add to your ~/.zshrc:\n  source {}\nThen reload: source ~/.zshrc",
                    script_path.display()
                )
            }
            Shell::Fish => {
                format!(
                    "Copy to fish completions:\n  cp {} ~/.config/fish/completions/nest.fish",
                    script_path.display()
                )
            }
            Shell::PowerShell => {
                format!(
                    "Run in PowerShell:\n  . {}\nOr add to your PowerShell profile",
                    script_path.display()
                )
            }
            Shell::Elvish => {
                format!(
                    "Add to your ~/.elvish/rc.elv:\n  eval (slurp < {})",
                    script_path.display()
                )
            }
        }
    }

    /// Detect current shell from environment
    pub fn detect_shell() -> Option<Shell> {
        // Try SHELL environment variable first
        if let Ok(shell_path) = std::env::var("SHELL") {
            let shell_name = std::path::Path::new(&shell_path)
                .file_name()
                .and_then(|n| n.to_str())?
                .to_lowercase();
            
            if shell_name.contains("zsh") {
                return Some(Shell::Zsh);
            } else if shell_name.contains("bash") {
                return Some(Shell::Bash);
            } else if shell_name.contains("fish") {
                return Some(Shell::Fish);
            } else if shell_name.contains("elvish") {
                return Some(Shell::Elvish);
            }
        }
        
        // Try PowerShell on Windows
        #[cfg(windows)]
        {
            if std::env::var("PSModulePath").is_ok() {
                return Some(Shell::PowerShell);
            }
        }
        
        None
    }

    /// Get shell configuration file path
    pub fn get_shell_config_path(shell: Shell) -> Option<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;
        
        let home_path = PathBuf::from(home);
        
        match shell {
            Shell::Bash => {
                // Try .bashrc first, then .bash_profile
                let bashrc = home_path.join(".bashrc");
                if bashrc.exists() {
                    return Some(bashrc);
                }
                let bash_profile = home_path.join(".bash_profile");
                if bash_profile.exists() {
                    return Some(bash_profile);
                }
                // Return .bashrc even if it doesn't exist (will be created)
                Some(bashrc)
            }
            Shell::Zsh => {
                Some(home_path.join(".zshrc"))
            }
            Shell::Fish => {
                // Fish uses a different approach - completions go to completions directory
                Some(home_path.join(".config/fish/completions/nest.fish"))
            }
            Shell::PowerShell => {
                // PowerShell profile location
                #[cfg(windows)]
                {
                    if let Ok(profile) = std::env::var("PROFILE") {
                        return Some(PathBuf::from(profile));
                    }
                }
                #[cfg(not(windows))]
                {
                    // On Unix, PowerShell might be installed via PowerShell Core
                    if let Ok(home) = std::env::var("HOME") {
                        return Some(PathBuf::from(home).join(".config/powershell/Microsoft.PowerShell_profile.ps1"));
                    }
                }
                None
            }
            Shell::Elvish => {
                Some(home_path.join(".elvish/rc.elv"))
            }
        }
    }

    /// Check if completion is already installed in shell config
    pub fn is_completion_installed(shell: Shell, script_path: &Path) -> bool {
        let config_path = match Self::get_shell_config_path(shell) {
            Some(path) => path,
            None => return false,
        };

        if !config_path.exists() {
            return false;
        }

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        // Check for different patterns depending on shell
        match shell {
            Shell::Bash | Shell::Zsh => {
                // Check for source command pointing to our script
                let script_str = script_path.to_string_lossy();
                content.contains(script_str.as_ref()) || 
                content.contains("nest/completions") ||
                content.contains("Nest CLI completion")
            }
            Shell::Fish => {
                // Fish completions are separate files, check if file exists
                config_path.exists()
            }
            Shell::PowerShell => {
                content.contains("nest") && content.contains("completion")
            }
            Shell::Elvish => {
                content.contains("nest") && (content.contains("completion") || content.contains("eval"))
            }
        }
    }

    /// Automatically install completion in shell configuration file
    pub fn install_completion(shell: Shell, script_path: &Path) -> Result<bool, String> {
        // Check if already installed
        if Self::is_completion_installed(shell, script_path) {
            return Ok(false); // Already installed, skip
        }

        match shell {
            Shell::Bash | Shell::Zsh => {
                let config_path = Self::get_shell_config_path(shell)
                    .ok_or_else(|| "Could not determine shell config path".to_string())?;
                
                // Ensure parent directory exists
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create config directory: {}", e))?;
                }

                // Read existing content
                let mut content = if config_path.exists() {
                    fs::read_to_string(&config_path)
                        .map_err(|e| format!("Failed to read config file: {}", e))?
                } else {
                    String::new()
                };

                // Add completion setup
                if !content.ends_with('\n') && !content.is_empty() {
                    content.push('\n');
                }
                content.push_str("\n# Nest CLI completion (auto-generated)\n");
                content.push_str(&format!("if [ -f {} ]; then\n", script_path.display()));
                content.push_str(&format!("    source {}\n", script_path.display()));
                content.push_str("fi\n");

                // Write back
                fs::write(&config_path, content)
                    .map_err(|e| format!("Failed to write config file: {}", e))?;

                Ok(true)
            }
            Shell::Fish => {
                // Fish uses completions directory
                let completion_file = Self::get_shell_config_path(shell)
                    .ok_or_else(|| "Could not determine fish completions path".to_string())?;
                
                // Ensure directory exists
                if let Some(parent) = completion_file.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create fish completions directory: {}", e))?;
                }

                // Copy completion script
                fs::copy(script_path, &completion_file)
                    .map_err(|e| format!("Failed to copy completion script: {}", e))?;

                Ok(true)
            }
            Shell::PowerShell => {
                let config_path = Self::get_shell_config_path(shell)
                    .ok_or_else(|| "Could not determine PowerShell profile path".to_string())?;
                
                // Ensure parent directory exists
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create PowerShell profile directory: {}", e))?;
                }

                // Read existing content
                let mut content = if config_path.exists() {
                    fs::read_to_string(&config_path)
                        .map_err(|e| format!("Failed to read PowerShell profile: {}", e))?
                } else {
                    String::new()
                };

                // Add completion setup
                if !content.ends_with('\n') && !content.is_empty() {
                    content.push('\n');
                }
                content.push_str("\n# Nest CLI completion (auto-generated)\n");
                content.push_str(&format!(". {}\n", script_path.display()));

                // Write back
                fs::write(&config_path, content)
                    .map_err(|e| format!("Failed to write PowerShell profile: {}", e))?;

                Ok(true)
            }
            Shell::Elvish => {
                let config_path = Self::get_shell_config_path(shell)
                    .ok_or_else(|| "Could not determine elvish config path".to_string())?;
                
                // Ensure parent directory exists
                if let Some(parent) = config_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create elvish config directory: {}", e))?;
                }

                // Read existing content
                let mut content = if config_path.exists() {
                    fs::read_to_string(&config_path)
                        .map_err(|e| format!("Failed to read elvish config: {}", e))?
                } else {
                    String::new()
                };

                // Add completion setup
                if !content.ends_with('\n') && !content.is_empty() {
                    content.push('\n');
                }
                content.push_str("\n# Nest CLI completion (auto-generated)\n");
                content.push_str(&format!("eval (slurp < {})\n", script_path.display()));

                // Write back
                fs::write(&config_path, content)
                    .map_err(|e| format!("Failed to write elvish config: {}", e))?;

                Ok(true)
            }
        }
    }

    /// Automatically install completion for current shell
    pub fn auto_install_completion(&self, nestfile_path: &Path) -> Result<Option<Shell>, String> {
        let shell = match Self::detect_shell() {
            Some(s) => s,
            None => return Ok(None), // Shell not detected, skip installation
        };

        let script_path = self.get_completion_script_path(shell, nestfile_path);
        
        // Check if script exists (should be generated first)
        if !script_path.exists() {
            return Ok(None); // Script not generated yet, skip
        }

        match Self::install_completion(shell, &script_path) {
            Ok(true) => Ok(Some(shell)), // Installed
            Ok(false) => Ok(Some(shell)), // Already installed
            Err(e) => Err(e),
        }
    }
}

impl Default for CompletionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create CompletionManager")
    }
}

