//! Command handlers for special flags and utility commands.
//!
//! This module deals with flags like --version, --json, --init, --example, and --update.

use super::ast::Command;
use super::output::OutputFormatter;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

pub fn handle_version() {
    use super::output::colors;
    let libc = detect_libc();
    let libc_info = if libc.to_lowercase() == "musl" {
        "musl"
    } else {
        "glibc"
    };

    println!(
        "{}nest{} {} ({})",
        colors::BRIGHT_BLUE,
        colors::RESET,
        OutputFormatter::value(env!("CARGO_PKG_VERSION")),
        libc_info
    );
    std::process::exit(0);
}

/// Handles the --show json flag.
///
/// Converts commands to JSON format and prints them.
///
/// # Arguments
///
/// * `commands` - The list of commands to serialize
///
/// # Returns
///
/// Returns `Ok(())` if successful, `Err(error)` if serialization fails.
pub fn handle_json(commands: &[Command]) -> Result<(), Box<dyn std::error::Error>> {
    use super::json::to_json;
    let json = to_json(commands)?;
    println!("{}", json);
    Ok(())
}

/// Handles the --show ast flag.
///
/// Prints commands in a tree format showing the AST structure.
///
/// # Arguments
///
/// * `commands` - The list of commands to display
pub fn handle_show_ast(commands: &[Command]) {
    use super::display::print_command;
    use super::output::colors;
    println!(
        "{}🌳{} {}AST Structure:{}\n",
        colors::BRIGHT_GREEN,
        colors::RESET,
        colors::BRIGHT_CYAN,
        colors::RESET
    );
    for command in commands {
        print_command(command, 0);
        println!();
    }
}

/// Handles the --example flag.
///
/// Prompts user for confirmation, then downloads the examples folder from GitHub
/// and changes directory into it.
///
/// # Errors
///
/// Exits with code 1 if:
/// - User declines confirmation
/// - Git is not available
/// - Clone fails
/// - Directory change fails
pub fn handle_example() {
    use std::io::{self, Write};

    // Ask for confirmation
    print!("Do you want to download the examples folder? (y/N): ");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let trimmed = input.trim().to_lowercase();
            if trimmed != "y" && trimmed != "yes" {
                OutputFormatter::info("Download cancelled.");
                std::process::exit(0);
            }
        }
        Err(e) => {
            OutputFormatter::error(&format!("Error reading input: {}", e));
            std::process::exit(1);
        }
    }

    // Get current directory
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            OutputFormatter::error(&format!("Error getting current directory: {}", e));
            std::process::exit(1);
        }
    };

    let examples_dir = current_dir.join("examples");

    // Check if examples directory already exists
    if examples_dir.exists() {
        OutputFormatter::error("Examples directory already exists in the current directory");
        OutputFormatter::info("Please remove it first or choose a different location.");
        std::process::exit(1);
    }

    OutputFormatter::info("Downloading examples folder from GitHub Releases...");

    // Try to download from GitHub Releases first
    let version = env!("CARGO_PKG_VERSION");
    let release_url = format!(
        "https://github.com/quonaro/nest/releases/download/v{}/examples.tar.gz",
        version
    );
    let latest_url = "https://github.com/quonaro/nest/releases/latest/download/examples.tar.gz";

    if download_examples_from_release(&current_dir, &examples_dir, &release_url, latest_url) {
        return;
    }

    // Fallback to repository clone method
    OutputFormatter::info("Release download failed, trying repository clone method...");
    download_examples_from_repo(&current_dir, &examples_dir);
}

/// Handles the --init flag.
///
/// Creates a basic nestfile in the current directory with example commands.
///
/// # Arguments
///
/// * `force` - If true, overwrite existing nestfile without confirmation
///
/// # Errors
///
/// Exits with code 1 if:
/// - File cannot be created
/// - File cannot be written
pub fn handle_init(force: bool) {
    use super::path::find_config_file;

    // Get current directory
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            OutputFormatter::error(&format!("Error getting current directory: {}", e));
            std::process::exit(1);
        }
    };

    // Check if nestfile already exists
    if let Some(existing_file) = find_config_file() {
        if !force {
            OutputFormatter::info(&format!(
                "Configuration file already exists: {}",
                existing_file.display()
            ));
            OutputFormatter::info("Use --force or -f to overwrite it.");
            std::process::exit(0);
        }
        // Force mode: overwrite without confirmation
        OutputFormatter::info(&format!(
            "Overwriting existing configuration file: {}",
            existing_file.display()
        ));
    }

    // Create basic nestfile template
    let nestfile_content = r#"# Nestfile - Task Runner Configuration
# This file defines commands that can be executed with: nest <command>

# ============================================================================
# BASIC COMMANDS
# ============================================================================

hello():
    > desc: Print a greeting message
    > script: |
        echo "Hello from Nest!"

build():
    > desc: Build the project
    > script: |
        echo "Building project..."
        # Add your build commands here

test():
    > desc: Run tests
    > script: |
        echo "Running tests..."
        # Add your test commands here

clean():
    > desc: Clean build artifacts
    > script: |
        echo "Cleaning build artifacts..."
        # Add your clean commands here

# ============================================================================
# COMMANDS WITH PARAMETERS
# ============================================================================

# Example command with parameters
# deploy(version: str, !env|e: str = "production"):
#     > desc: Deploy application
#     > script: |
#         echo "Deploying version {{version}} to {{env}}"
#         # Add your deployment commands here

# ============================================================================
# VARIABLES AND CONSTANTS
# ============================================================================

# @var APP_NAME = "myapp"
# @var VERSION = "1.0.0"
# @const BUILD_DIR = "./dist"

# ============================================================================
# NESTED COMMANDS (GROUPS)
# ============================================================================

# dev:
#     > desc: Development commands
#     
#     dev start():
#         > desc: Start development server
#         > script: |
#             echo "Starting development server..."
#     
#     dev test():
#         > desc: Run development tests
#         > script: |
#             echo "Running development tests..."

# For more examples, see: nest --example
"#;

    let nestfile_path = current_dir.join("nestfile");

    // Write nestfile
    match fs::write(&nestfile_path, nestfile_content) {
        Ok(_) => {
            OutputFormatter::info(&format!("Created nestfile at: {}", nestfile_path.display()));
            OutputFormatter::info(
                "You can now add commands to your nestfile and run them with: nest <command>",
            );
        }
        Err(e) => {
            OutputFormatter::error(&format!("Failed to create nestfile: {}", e));
            std::process::exit(1);
        }
    }
}

/// Downloads examples folder from GitHub Releases.
/// Returns true if successful, false otherwise.
fn download_examples_from_release(
    current_dir: &std::path::Path,
    examples_dir: &std::path::Path,
    versioned_url: &str,
    latest_url: &str,
) -> bool {
    let archive_name = "examples.tar.gz";
    let temp_archive = current_dir.join(archive_name);

    // Clean up temp archive if it exists
    if temp_archive.exists() {
        let _ = fs::remove_file(&temp_archive);
    }

    // Try downloading from versioned release first, then latest
    let download_urls = vec![versioned_url, latest_url];
    let mut download_success = false;

    for url in download_urls {
        OutputFormatter::info(&format!("Trying to download from: {}", url));

        // Try curl first
        let curl_result = ProcessCommand::new("curl")
            .args([
                "-fsSL",
                "-o",
                temp_archive.to_str().unwrap_or(archive_name),
                url,
            ])
            .output();

        match curl_result {
            Ok(output) if output.status.success() => {
                download_success = true;
                break;
            }
            _ => {
                // Try wget
                let wget_result = ProcessCommand::new("wget")
                    .args([
                        "-q",
                        "-O",
                        temp_archive.to_str().unwrap_or(archive_name),
                        url,
                    ])
                    .output();

                match wget_result {
                    Ok(output) if output.status.success() => {
                        download_success = true;
                        break;
                    }
                    _ => continue,
                }
            }
        }
    }

    if !download_success {
        OutputFormatter::info("Failed to download from GitHub Releases");
        if temp_archive.exists() {
            let _ = fs::remove_file(&temp_archive);
        }
        return false;
    }

    // Verify archive exists
    if !temp_archive.exists() {
        OutputFormatter::error("Downloaded archive not found");
        return false;
    }

    // Extract archive
    OutputFormatter::info("Extracting archive...");
    let extract_output = ProcessCommand::new("tar")
        .args([
            "xzf",
            temp_archive.to_str().unwrap_or(archive_name),
            "-C",
            current_dir.to_str().unwrap_or("."),
        ])
        .output();

    match extract_output {
        Ok(output) if output.status.success() => {
            // Verify examples directory was extracted
            if examples_dir.exists() {
                // Clean up archive
                let _ = fs::remove_file(&temp_archive);

                use super::output::colors;
                OutputFormatter::success("Examples folder downloaded successfully!");
                println!(
                    "  {}Location:{} {}",
                    OutputFormatter::help_label("Location:"),
                    colors::RESET,
                    OutputFormatter::path(&examples_dir.display().to_string())
                );
                println!(
                    "\n{}Changing to examples directory...{}",
                    colors::BRIGHT_CYAN,
                    colors::RESET
                );
                println!("Run: cd examples");
                true
            } else {
                OutputFormatter::error("Examples directory not found after extraction");
                let _ = fs::remove_file(&temp_archive);
                false
            }
        }
        Ok(_) => {
            OutputFormatter::error("Failed to extract archive");
            let _ = fs::remove_file(&temp_archive);
            false
        }
        Err(_) => {
            OutputFormatter::error("tar command not available. Please install tar.");
            let _ = fs::remove_file(&temp_archive);
            false
        }
    }
}

/// Downloads examples folder from repository (fallback method).
fn download_examples_from_repo(current_dir: &std::path::Path, examples_dir: &std::path::Path) {
    // Try to clone the repository (just the examples folder)
    // We'll clone into a temp directory, then move the examples folder
    let temp_dir = current_dir.join(".nest_examples_temp");

    // Clean up temp directory if it exists
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    OutputFormatter::info("Downloading examples folder from GitHub repository...");

    // Clone repository (depth 1 for faster download)
    let clone_output = ProcessCommand::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            "--sparse",
            "https://github.com/quonaro/nest.git",
            temp_dir.to_str().unwrap_or(".nest_examples_temp"),
        ])
        .output();

    match clone_output {
        Ok(output) if output.status.success() => {
            // Set sparse checkout to only get examples folder
            let sparse_output = ProcessCommand::new("git")
                .args(["sparse-checkout", "set", "cli/examples"])
                .current_dir(&temp_dir)
                .output();

            match sparse_output {
                Ok(sparse_result) if sparse_result.status.success() => {
                    // Checkout files after sparse checkout configuration
                    let checkout_output = ProcessCommand::new("git")
                        .args(["checkout"])
                        .current_dir(&temp_dir)
                        .output();

                    if checkout_output.is_err() {
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        OutputFormatter::error("Failed to checkout files after sparse checkout");
                        std::process::exit(1);
                    }
                }
                _ => {
                    // If sparse checkout fails, try full checkout
                    OutputFormatter::info("Sparse checkout failed, using full checkout...");
                    let checkout_output = ProcessCommand::new("git")
                        .args(["checkout"])
                        .current_dir(&temp_dir)
                        .output();

                    if checkout_output.is_err() {
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        OutputFormatter::error("Failed to checkout files");
                        std::process::exit(1);
                    }
                }
            }

            // Move examples folder from temp/cli/examples to current_dir/examples
            let source_examples = temp_dir.join("cli").join("examples");

            if source_examples.exists() {
                match std::fs::rename(&source_examples, examples_dir) {
                    Ok(_) => {
                        // Clean up temp directory
                        let _ = std::fs::remove_dir_all(&temp_dir);

                        use super::output::colors;
                        OutputFormatter::success("Examples folder downloaded successfully!");
                        println!(
                            "  {}Location:{} {}",
                            OutputFormatter::help_label("Location:"),
                            colors::RESET,
                            OutputFormatter::path(&examples_dir.display().to_string())
                        );

                        // Change directory to examples
                        println!(
                            "\n{}Changing to examples directory...{}",
                            colors::BRIGHT_CYAN,
                            colors::RESET
                        );
                        println!("Run: cd examples");
                    }
                    Err(e) => {
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        OutputFormatter::error(&format!("Error moving examples folder: {}", e));
                        std::process::exit(1);
                    }
                }
            } else {
                let _ = std::fs::remove_dir_all(&temp_dir);
                OutputFormatter::error("Examples folder not found in repository");
                std::process::exit(1);
            }
        }
        Ok(_) => {
            let _ = std::fs::remove_dir_all(&temp_dir);
            OutputFormatter::error("Git clone failed");
            std::process::exit(1);
        }
        Err(_) => {
            // Git not available, try alternative method: download archive
            let _ = std::fs::remove_dir_all(&temp_dir);
            OutputFormatter::info("Git not available, trying alternative download method...");

            // Try downloading as archive using curl/wget
            download_examples_archive(current_dir, examples_dir);
        }
    }
}

/// Downloads examples folder as archive (fallback method when git is not available).
fn download_examples_archive(current_dir: &std::path::Path, examples_dir: &std::path::Path) {
    let archive_url = "https://github.com/quonaro/nest/archive/refs/heads/main.zip";
    let temp_zip = current_dir.join(".nest_examples_temp.zip");
    let temp_extract = current_dir.join(".nest_examples_temp_extract");

    // Download archive
    OutputFormatter::info("Downloading archive...");
    let _download_output = match ProcessCommand::new("curl")
        .args([
            "-fsSL",
            "-o",
            temp_zip.to_str().unwrap_or(".nest_examples_temp.zip"),
            archive_url,
        ])
        .output()
    {
        Ok(output) if output.status.success() => output,
        Ok(_) => {
            // Try wget
            match ProcessCommand::new("wget")
                .args([
                    "-q",
                    "-O",
                    temp_zip.to_str().unwrap_or(".nest_examples_temp.zip"),
                    archive_url,
                ])
                .output()
            {
                Ok(output) if output.status.success() => output,
                Ok(_) => {
                    OutputFormatter::error("Both curl and wget failed to download archive");
                    std::process::exit(1);
                }
                Err(_) => {
                    OutputFormatter::error("Neither curl nor wget is available");
                    OutputFormatter::info("Please install git, curl, or wget to use this feature.");
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            // curl not found, try wget
            match ProcessCommand::new("wget")
                .args([
                    "-q",
                    "-O",
                    temp_zip.to_str().unwrap_or(".nest_examples_temp.zip"),
                    archive_url,
                ])
                .output()
            {
                Ok(output) if output.status.success() => output,
                Ok(_) => {
                    OutputFormatter::error("wget failed to download archive");
                    std::process::exit(1);
                }
                Err(_) => {
                    OutputFormatter::error("Neither curl nor wget is available");
                    OutputFormatter::info("Please install git, curl, or wget to use this feature.");
                    std::process::exit(1);
                }
            }
        }
    };

    // Extract archive (requires unzip or tar)
    OutputFormatter::info("Extracting archive...");
    let extract_output = ProcessCommand::new("unzip")
        .args([
            "-q",
            temp_zip.to_str().unwrap_or(".nest_examples_temp.zip"),
            "-d",
            temp_extract
                .to_str()
                .unwrap_or(".nest_examples_temp_extract"),
        ])
        .output();

    match extract_output {
        Ok(output) if output.status.success() => {
            // Move examples folder
            let source_examples = temp_extract.join("nest-main").join("cli").join("examples");

            if source_examples.exists() {
                match std::fs::rename(&source_examples, examples_dir) {
                    Ok(_) => {
                        // Clean up
                        let _ = fs::remove_file(&temp_zip);
                        let _ = fs::remove_dir_all(&temp_extract);

                        use super::output::colors;
                        OutputFormatter::success("Examples folder downloaded successfully!");
                        println!(
                            "  {}Location:{} {}",
                            OutputFormatter::help_label("Location:"),
                            colors::RESET,
                            OutputFormatter::path(&examples_dir.display().to_string())
                        );
                        println!(
                            "\n{}Changing to examples directory...{}",
                            colors::BRIGHT_CYAN,
                            colors::RESET
                        );
                        println!("Run: cd examples");
                    }
                    Err(e) => {
                        let _ = fs::remove_file(&temp_zip);
                        let _ = fs::remove_dir_all(&temp_extract);
                        OutputFormatter::error(&format!("Error moving examples folder: {}", e));
                        std::process::exit(1);
                    }
                }
            } else {
                let _ = fs::remove_file(&temp_zip);
                let _ = fs::remove_dir_all(&temp_extract);
                OutputFormatter::error("Examples folder not found in archive");
                std::process::exit(1);
            }
        }
        Ok(_) => {
            let _ = fs::remove_file(&temp_zip);
            OutputFormatter::error("Failed to extract archive. Please install unzip.");
            std::process::exit(1);
        }
        Err(_) => {
            let _ = fs::remove_file(&temp_zip);
            OutputFormatter::error("unzip is not available. Please install unzip or use git.");
            std::process::exit(1);
        }
    }
}

/// # Arguments
///
/// * `recreate` - If true, run the official installation script instead of updating the current binary.
///
/// # Errors
///
/// Exits with code 1 if:
/// - OS or architecture is not supported
/// - curl or wget is not available
/// - Download fails
/// - Archive extraction fails
/// - Binary replacement fails
pub fn handle_update(recreate: bool) {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let try_sudo_retry = || {
        if !env::args().any(|a| a == "--sudo-retry") {
            #[cfg(unix)]
            {
                OutputFormatter::info("Permission denied. Retrying with sudo...");
                let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("nest"));
                let mut args: Vec<String> = env::args().collect();
                args.push("--sudo-retry".to_string());

                // We use sudo to run the SAME command again.
                // The --sudo-retry flag prevents infinite loops if sudo also fails.
                let status = ProcessCommand::new("sudo")
                    .arg(current_exe)
                    .args(&args[1..])
                    .status();

                if let Ok(s) = status {
                    if s.success() {
                        std::process::exit(0);
                    }
                }
            }
        }
    };

    // Handle --recreate
    if recreate {
        OutputFormatter::info("Recreating Nest... Running official installation script.");
        if ProcessCommand::new("curl")
            .arg("--version")
            .output()
            .is_ok()
        {
            let status = ProcessCommand::new("bash")
                .arg("-c")
                .arg("curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash")
                .status();
            match status {
                Ok(s) if s.success() => {
                    OutputFormatter::success("Nest successfully recreated.");
                    std::process::exit(0);
                }
                _ => {
                    OutputFormatter::error("Failed to run installation script.");
                    std::process::exit(1);
                }
            }
        } else {
            OutputFormatter::error("curl is required for --recreate.");
            std::process::exit(1);
        }
    }

    // Detect OS and architecture
    let (platform, architecture) = match detect_platform() {
        Ok((p, a)) => (p, a),
        Err(e) => {
            OutputFormatter::error(&e);
            std::process::exit(1);
        }
    };

    // libc / flavor selection for Linux x86_64:
    // - default: glibc (asset: nest-linux-x86_64.tar.gz)
    // - NEST_LIBC=musl -> static musl (asset: nest-linux-musl-x86_64.tar.gz)
    let libc_flavor = match env::var("NEST_LIBC") {
        Ok(v) => v,
        Err(_) => {
            if platform == "linux" && architecture == "x86_64" {
                detect_libc()
            } else {
                "glibc".to_string()
            }
        }
    };

    // Archive platform name (differs for linux glibc vs musl)
    let platform_archive = if platform == "linux" && architecture == "x86_64" {
        match libc_flavor.to_lowercase().as_str() {
            "musl" => "linux-musl".to_string(),
            "glibc" | "" => "linux".to_string(),
            other => {
                OutputFormatter::info(&format!(
                    "Unknown NEST_LIBC='{}', falling back to glibc (linux archive)",
                    other
                ));
                "linux".to_string()
            }
        }
    } else {
        platform.clone()
    };

    // Determine binary name and installation path
    let binary_name = "nest";
    let current_exe = env::current_exe().ok();

    let install_dir = current_exe
        .as_ref()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            env::var("HOME")
                .map(|home| PathBuf::from(home).join(".local").join("bin"))
                .unwrap_or_else(|_| PathBuf::from("/usr/local/bin"))
        });

    let binary_path = current_exe.unwrap_or_else(|| install_dir.join(binary_name));

    // GitHub repository
    let repo = "quonaro/nest";
    let version = "latest";

    // Print header
    OutputFormatter::info("Updating Nest CLI...");
    println!("  Platform: {}-{}", platform, architecture);
    if platform == "linux" && architecture == "x86_64" {
        if platform_archive == "linux-musl" {
            println!("  Libc: musl (static)");
        } else {
            println!("  Libc: glibc");
        }
    }
    println!("  Install directory: {}", install_dir.display());

    // Create install directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&install_dir) {
        OutputFormatter::error(&format!("Failed to create install directory: {}", e));
        std::process::exit(1);
    }

    // Build download URL
    let url = if version == "latest" {
        format!(
            "https://github.com/{}/releases/latest/download/nest-{}-{}.tar.gz",
            repo, platform_archive, architecture
        )
    } else {
        format!(
            "https://github.com/{}/releases/download/v{}/nest-{}-{}.tar.gz",
            repo, version, platform_archive, architecture
        )
    };

    // Create temporary directory
    let temp_dir = env::temp_dir().join(format!("nest-update-{}", std::process::id()));
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        OutputFormatter::error(&format!("Failed to create temporary directory: {}", e));
        std::process::exit(1);
    }
    let temp_file = temp_dir.join(format!("nest-{}-{}.tar.gz", platform_archive, architecture));

    // Download binary
    OutputFormatter::info("Downloading Nest CLI...");
    println!("  URL: {}", url);

    // Convert paths to strings with proper error handling
    let temp_file_str = match temp_file.to_str() {
        Some(s) => s,
        None => {
            OutputFormatter::error("Invalid temporary file path encoding");
            std::process::exit(1);
        }
    };

    let download_success = if ProcessCommand::new("curl")
        .arg("--version")
        .output()
        .is_ok()
    {
        // Use curl
        let output = ProcessCommand::new("curl")
            .args([
                "-L",
                "-s",
                "-S",
                "--show-error",
                "-w",
                "%{http_code}",
                "-o",
                temp_file_str,
                &url,
            ])
            .output();

        match output {
            Ok(result) => {
                // HTTP code is in stdout (last line)
                let stdout = String::from_utf8_lossy(&result.stdout);
                let http_code = stdout.trim();

                if http_code == "200" {
                    true
                } else {
                    // Print stderr if available
                    if !result.stderr.is_empty() {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        OutputFormatter::error(&format!(
                            "Failed to download binary (HTTP {}): {}",
                            http_code, stderr
                        ));
                    } else {
                        OutputFormatter::error(&format!(
                            "Failed to download binary (HTTP {})",
                            http_code
                        ));
                    }
                    false
                }
            }
            Err(e) => {
                OutputFormatter::error(&format!("curl failed: {}", e));
                false
            }
        }
    } else if ProcessCommand::new("wget")
        .arg("--version")
        .output()
        .is_ok()
    {
        // Use wget
        let output = ProcessCommand::new("wget")
            .args(["-O", temp_file_str, &url])
            .output();

        match output {
            Ok(result) if result.status.success() => true,
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                if !stderr.is_empty() {
                    OutputFormatter::error(&format!("wget failed: {}", stderr));
                } else {
                    OutputFormatter::error("wget failed to download file");
                }
                false
            }
            Err(e) => {
                OutputFormatter::error(&format!("wget failed: {}", e));
                false
            }
        }
    } else {
        OutputFormatter::error("Neither curl nor wget is available");
        OutputFormatter::info("Please install curl or wget to use this feature.");
        false
    };

    if !download_success {
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Verify downloaded file exists and is not empty
    match fs::metadata(&temp_file) {
        Ok(meta) if meta.len() > 0 => {}
        Ok(_) => {
            OutputFormatter::error("Downloaded file is empty");
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
        Err(e) => {
            OutputFormatter::error(&format!("Failed to verify downloaded file: {}", e));
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
    }

    // Extract archive
    OutputFormatter::info("Extracting archive...");
    let extract_dir = temp_dir.join("extract");
    if let Err(e) = fs::create_dir_all(&extract_dir) {
        OutputFormatter::error(&format!("Failed to create extract directory: {}", e));
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Convert extract directory path to string with proper error handling
    let extract_dir_str = match extract_dir.to_str() {
        Some(s) => s,
        None => {
            OutputFormatter::error("Invalid extract directory path encoding");
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
    };

    let extract_output = ProcessCommand::new("tar")
        .args(["-xzf", temp_file_str, "-C", extract_dir_str])
        .output();

    match extract_output {
        Ok(result) if result.status.success() => {}
        Ok(_) => {
            OutputFormatter::error("Failed to extract archive");
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
        Err(e) => {
            OutputFormatter::error(&format!("tar failed: {}", e));
            let _ = fs::remove_dir_all(&temp_dir);
            std::process::exit(1);
        }
    }

    // Check if binary exists in extracted archive
    let extracted_binary = extract_dir.join(binary_name);
    if !extracted_binary.exists() {
        OutputFormatter::error(&format!("Binary '{}' not found in archive", binary_name));
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Replace binary using atomic rename to avoid "Text file busy" error
    OutputFormatter::info("Installing binary...");

    // Copy new binary to temporary file in the same directory as target
    // This allows atomic rename operation
    let new_binary_path = binary_path.with_extension("new");
    if let Err(e) = fs::copy(&extracted_binary, &new_binary_path) {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            try_sudo_retry();
        }
        OutputFormatter::error(&format!("Failed to copy new binary: {}", e));
        let _ = fs::remove_dir_all(&temp_dir);
        std::process::exit(1);
    }

    // Make new binary executable before renaming
    // On Unix systems, set explicit permissions; on Windows, permissions are handled automatically
    #[cfg(unix)]
    {
        let mut perms = match fs::metadata(&new_binary_path) {
            Ok(meta) => meta.permissions(),
            Err(e) => {
                OutputFormatter::error(&format!("Failed to get file permissions: {}", e));
                let _ = fs::remove_dir_all(&temp_dir);
                let _ = fs::remove_file(&new_binary_path);
                std::process::exit(1);
            }
        };
        perms.set_mode(0o755);
        if let Err(e) = fs::set_permissions(&new_binary_path, perms) {
            OutputFormatter::error(&format!("Failed to set executable permissions: {}", e));
            let _ = fs::remove_dir_all(&temp_dir);
            let _ = fs::remove_file(&new_binary_path);
            std::process::exit(1);
        }
    }

    // Try to replace the binary.
    // On Linux, if the binary is running, we MUST move the old one out of the way (unlink it)
    // instead of trying to remove it directly if rename fails.
    let mut replaced = false;

    // 1. Try direct rename (atomically replace)
    if fs::rename(&new_binary_path, &binary_path).is_ok() {
        replaced = true;
    } else {
        // 2. If rename failed (likely "Text file busy"), try moving the OLD binary to a backup first
        let backup_path = binary_path.with_extension("old");
        if fs::rename(&binary_path, &backup_path).is_ok() {
            // Now that the old binary is moved (unlinked from the name 'nest'), we can rename the new one in
            if fs::rename(&new_binary_path, &binary_path).is_ok() {
                replaced = true;
                // Try to remove the backup, but don't fail if we can't (it might be in use)
                let _ = fs::remove_file(&backup_path).ok();
            } else {
                // If this still fails, try to restore the original
                let _ = fs::rename(&backup_path, &binary_path);
            }
        }
    }

    if !replaced {
        // If failed, try sudo as a last resort (if not already tried)
        try_sudo_retry();

        OutputFormatter::error("Failed to install binary: Permission denied or Text file busy");
        OutputFormatter::info("Please try running with sudo or close running instances.");
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::remove_file(&new_binary_path);
        std::process::exit(1);
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    // Success message
    OutputFormatter::success("Nest CLI updated successfully!");

    // Attempt to run the new version to show it
    if let Ok(output) = ProcessCommand::new(&binary_path).arg("--version").output() {
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!(
                "  Current version: {}",
                OutputFormatter::value(&version_str)
            );
        }
    }
}

/// Detects the libc flavor of the currently running binary.
/// Returns "musl" or "glibc".
fn detect_libc() -> String {
    // Check ldd on the current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if let Ok(output) = ProcessCommand::new("ldd").arg(current_exe).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("musl") {
                return "musl".to_string();
            }
            if stdout.contains("libc.so") || stdout.contains("ld-linux") {
                return "glibc".to_string();
            }
            // If it's a static binary, ldd might say "not a dynamic executable"
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not a dynamic executable") || stdout.contains("statically linked") {
                // We assume musl for our static builds
                return "musl".to_string();
            }
        }
    }

    "glibc".to_string()
}

/// Detects the platform and architecture.
///
/// # Returns
///
/// Returns `Ok((platform, architecture))` if detection succeeds,
/// or `Err(error_message)` if the OS or architecture is not supported.
fn detect_platform() -> Result<(String, String), String> {
    // Check if uname is available (Unix systems)
    let os_output = match ProcessCommand::new("uname").arg("-s").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => {
            // On Windows or if uname is not available
            #[cfg(windows)]
            return Err("Update command is not supported on Windows. Please use the PowerShell install script (install.ps1) instead.".to_string());

            #[cfg(not(windows))]
            return Err("Failed to detect OS. The 'uname' command is required.".to_string());
        }
    };

    let platform = match os_output.as_str() {
        "Linux" => "linux",
        "Darwin" => "macos",
        _ => {
            return Err(format!(
                "Unsupported OS: {}. Update command currently supports Linux and macOS only.",
                os_output
            ))
        }
    };

    // Detect architecture
    let arch_output = ProcessCommand::new("uname").arg("-m").output();
    let arch = match arch_output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => return Err("Failed to detect architecture".to_string()),
    };

    let architecture = match arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" | "arm64" => "aarch64",
        _ => return Err(format!("Unsupported architecture: {}", arch)),
    };

    Ok((platform.to_string(), architecture.to_string()))
}
