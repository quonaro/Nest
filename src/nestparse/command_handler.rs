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
        let args = ArgumentExtractor::extract_for_default_command(
            matches_for_args,
            &default_cmd.parameters,
            generator,
        );

        // Get flags from root matches (they're global)
        let dry_run = root_matches.get_flag(FLAG_DRY_RUN);
        let verbose = root_matches.get_flag(FLAG_VERBOSE);

        generator.execute_command(default_cmd, &args, Some(&default_path), dry_run, verbose)
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
        let args = ArgumentExtractor::extract_from_matches(matches, &command.parameters, generator);

        // Get flags from root matches (they're global)
        let dry_run = root_matches.get_flag(FLAG_DRY_RUN);
        let verbose = root_matches.get_flag(FLAG_VERBOSE);

        generator.execute_command(command, &args, Some(command_path), dry_run, verbose)
    }

    fn get_group_matches(matches: &ArgMatches) -> &ArgMatches {
        matches
            .subcommand()
            .map(|(_, sub_matches)| sub_matches)
            .unwrap_or(matches)
    }
}
