use super::ast::{Command, Directive};

pub fn print_command(command: &Command, indent: usize) {
    let indent_str = "  ".repeat(indent);
    println!("{}└─ {}", indent_str, command);

    // Print directives
    for directive in &command.directives {
        match directive {
            Directive::Desc(s) => {
                println!("{}    > desc: {}", indent_str, s);
            }
            Directive::Cwd(s) => {
                println!("{}    > cwd: {}", indent_str, s);
            }
            Directive::Env(s) => {
                println!("{}    > env: {}", indent_str, s);
            }
            Directive::Script(s) => {
                if s.contains('\n') {
                    println!("{}    > script: |", indent_str);
                    for line in s.lines() {
                        println!("{}        {}", indent_str, line);
                    }
                } else {
                    println!("{}    > script: {}", indent_str, s);
                }
            }
        }
    }

    // Print children
    for child in &command.children {
        print_command(child, indent + 1);
    }
}

