use super::ast::Command;
use super::args::ArgumentExtractor;
use super::cli::CliGenerator;
use super::help::HelpFormatter;
use clap::ArgMatches;

pub struct CommandHandler;

impl CommandHandler {
    pub fn handle_group_without_default(
        command: &Command,
        command_path: &[String],
    ) -> Result<(), ()> {
        HelpFormatter::print_group_help(command, command_path);
        Ok(())
    }

    pub fn handle_default_command(
        matches: &ArgMatches,
        command_path: &[String],
        generator: &CliGenerator,
    ) -> Result<(), String> {
        let default_path = {
            let mut path = command_path.to_vec();
            path.push("default".to_string());
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

        generator.execute_command(default_cmd, &args, Some(&default_path))
    }

    pub fn handle_regular_command(
        matches: &ArgMatches,
        command: &Command,
        generator: &CliGenerator,
        command_path: &[String],
    ) -> Result<(), String> {
        let args = ArgumentExtractor::extract_from_matches(matches, &command.parameters, generator);
        generator.execute_command(command, &args, Some(command_path))
    }

    fn get_group_matches(matches: &ArgMatches) -> &ArgMatches {
        matches
            .subcommand()
            .map(|(_, sub_matches)| sub_matches)
            .unwrap_or(matches)
    }
}

