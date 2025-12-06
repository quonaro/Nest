use super::ast::{Command, Directive};

pub struct HelpFormatter;

impl HelpFormatter {
    pub fn print_group_help(command: &Command, command_path: &[String]) {
        println!("Usage: nest {} [COMMAND]", command_path.join(" "));
        println!();

        if let Some(desc) = Self::extract_description(&command.directives) {
            println!("{}", desc);
            println!();
        }

        println!("Available commands:");
        for child in &command.children {
            let child_desc = Self::extract_description(&child.directives);
            if let Some(desc) = child_desc {
                println!("  {}  {}", child.name, desc);
            } else {
                println!("  {}", child.name);
            }
        }
    }

    fn extract_description(directives: &[Directive]) -> Option<&str> {
        directives.iter().find_map(|d| {
            if let Directive::Desc(s) = d {
                Some(s.as_str())
            } else {
                None
            }
        })
    }
}

