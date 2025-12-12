//! Command execution orchestration.
//!
//! This module handles routing and executing different types of commands:
//! - Group commands without default subcommands (show help)
//! - Group commands with default subcommands (execute default)
//! - Regular commands (execute directly)

use super::args::ArgumentExtractor;
use super::ast::Command;
use super::cli::CliGenerator;
use super::help::HelpFormatter;
use crate::constants::{DEFAULT_SUBCOMMAND, FLAG_DRY_RUN, FLAG_VERBOSE};
use clap::ArgMatches;

/// Handles command execution routing and orchestration.
///
/// This is a utility struct with static methods for command handling.
pub struct CommandHandler;

impl CommandHandler {
    /// Handles a group command that doesn't have a default subcommand.
    ///
    /// When a user calls a group command (like `nest dev`) without specifying
    /// a subcommand and there's no default subcommand, this shows the help
    /// message for the group.
    ///
    /// # Arguments
    ///
    /// * `command` - The group command
    /// * `command_path` - The full path to the command
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if help was displayed successfully,
    /// `Err(())` otherwise.
    pub fn handle_group_without_default(
        command: &Command,
        command_path: &[String],
    ) -> Result<(), ()> {
        HelpFormatter::print_group_help(command, command_path);
        Ok(())
    }

    /// Handles execution of a default subcommand.
    ///
    /// When a user calls a group command (like `nest dev`) that has a
    /// `default` subcommand, this function finds and executes that default
    /// subcommand with the arguments passed to the parent group.
    ///
    /// # Arguments
    ///
    /// * `matches` - The parsed CLI arguments from clap
    /// * `command_path` - The path to the parent group command
    /// * `generator` - CLI generator for finding and executing commands
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if execution succeeded,
    /// `Err(message)` if execution failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The default subcommand is not found
    /// - Command execution fails
    pub fn handle_default_command(
        matches: &ArgMatches,
        command_path: &[String],
        generator: &CliGenerator,
        root_matches: &ArgMatches,
    ) -> Result<(), String> {
        let default_path = {
            let mut path = command_path.to_vec();
            path.push(DEFAULT_SUBCOMMAND.to_string());
            path
        };

        let default_cmd = generator
            .find_command(&default_path)
            .ok_or_else(|| "Default command not found".to_string())?;

        let matches_for_args = Self::get_group_matches(matches);
        let args = match ArgumentExtractor::extract_for_default_command(
            matches_for_args,
            &default_cmd.parameters,
            generator,
            &default_path,
        ) {
            Ok(args) => args,
            Err(validation_errors) => {
                use super::output::OutputFormatter;
                for error in &validation_errors {
                    OutputFormatter::error(error);
                }
                return Err("Type validation failed".to_string());
            }
        };

        // Extract parent command arguments (from the group command)
        let parent_args = Self::extract_parent_args(root_matches, command_path, generator);

        // Get flags from root matches (they're global)
        let dry_run = root_matches.get_flag(FLAG_DRY_RUN);
        let verbose = root_matches.get_flag(FLAG_VERBOSE);

        // Execute with parent args - we need to modify execute_command to accept parent_args
        // For now, we'll pass empty parent args and handle it in execute_command_with_deps
        generator.execute_command_with_parent_args(default_cmd, &args, Some(&default_path), dry_run, verbose, &parent_args)
    }

    /// Handles execution of a regular (non-group) command.
    ///
    /// This function extracts arguments from the CLI matches and executes
    /// the command's script with those arguments.
    ///
    /// # Arguments
    ///
    /// * `matches` - The parsed CLI arguments from clap
    /// * `command` - The command to execute
    /// * `generator` - CLI generator for executing commands
    /// * `command_path` - The full path to the command
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if execution succeeded,
    /// `Err(message)` if execution failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Command has no script directive
    /// - Script execution fails
    pub fn handle_regular_command(
        matches: &ArgMatches,
        command: &Command,
        generator: &CliGenerator,
        command_path: &[String],
        root_matches: &ArgMatches,
    ) -> Result<(), String> {
        let args = if command.has_wildcard {
            // For wildcard commands, extract all remaining arguments
            ArgumentExtractor::extract_wildcard_args(matches)
        } else {
            match ArgumentExtractor::extract_from_matches(
                matches,
                &command.parameters,
                generator,
                command_path,
            ) {
                Ok(args) => args,
                Err(validation_errors) => {
                    use super::output::OutputFormatter;
                    for error in &validation_errors {
                        OutputFormatter::error(error);
                    }
                    return Err("Type validation failed".to_string());
                }
            }
        };

        // Extract parent command arguments (if this is a nested command)
        // Use root_matches to access parent command arguments
        let parent_args = Self::extract_parent_args(root_matches, command_path, generator);

        // Get flags from root matches (they're global)
        let dry_run = root_matches.get_flag(FLAG_DRY_RUN);
        let verbose = root_matches.get_flag(FLAG_VERBOSE);

        // Execute with parent args
        generator.execute_command_with_parent_args(command, &args, Some(command_path), dry_run, verbose, &parent_args)
    }

    fn get_group_matches(matches: &ArgMatches) -> &ArgMatches {
        matches
            .subcommand()
            .map(|(_, sub_matches)| sub_matches)
            .unwrap_or(matches)
    }

    /// Extracts arguments from parent commands in the command path.
    ///
    /// For a command path like ["grp", "start"], this extracts arguments
    /// from the "grp" parent command.
    ///
    /// # Arguments
    ///
    /// * `root_matches` - Root ArgMatches containing all command information
    /// * `command_path` - Full path to the command (e.g., ["grp", "start"])
    /// * `generator` - CLI generator for finding commands
    fn extract_parent_args(
        root_matches: &ArgMatches,
        command_path: &[String],
        generator: &CliGenerator,
    ) -> std::collections::HashMap<String, String> {
        let mut parent_args = std::collections::HashMap::new();

        // If command_path has only one element, there are no parents
        if command_path.len() <= 1 {
            return parent_args;
        }

        // Traverse up the command path to extract parent arguments
        // For path ["grp", "start"], we need to get args from "grp"
        let mut current_matches = root_matches;
        let mut current_path = Vec::new();

        // Navigate to each parent command's matches
        for name in command_path.iter().take(command_path.len() - 1) {
            current_path.push(name.clone());
            
            // Find the parent command to get its parameters
            if let Some(parent_cmd) = generator.find_command(&current_path) {
                // Navigate to this parent command's matches
                if let Some((_, sub_matches)) = current_matches.subcommand() {
                    current_matches = sub_matches;
                } else {
                    // If we can't find subcommand matches, skip this parent
                    continue;
                }

                // Extract arguments from parent command
                let parent_cmd_args = if parent_cmd.has_wildcard {
                    ArgumentExtractor::extract_wildcard_args(current_matches)
                } else {
                    match ArgumentExtractor::extract_from_matches(
                        current_matches,
                        &parent_cmd.parameters,
                        generator,
                        &current_path,
                    ) {
                        Ok(args) => args,
                        Err(_) => std::collections::HashMap::new(), // Skip if extraction fails
                    }
                };

                // Merge parent args (later parents override earlier ones)
                for (key, value) in parent_cmd_args {
                    parent_args.insert(key, value);
                }
            }
        }

        parent_args
    }
}
