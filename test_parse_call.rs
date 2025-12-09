fn parse_command_call(line: &str) -> Option<(String, std::collections::HashMap<String, String>)> {
    let trimmed = line.trim();
    
    // Check if line looks like a command call
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    
    // Check for shell operators
    let shell_operators = ["|", "&&", "||", ";", ">", "<", ">>", "<<", "&", "$", "`"];
    if shell_operators.iter().any(|&op| trimmed.contains(op)) {
        return None;
    }
    
    // Check if there are parentheses (arguments)
    if let Some(open_paren) = trimmed.find('(') {
        let command_path = trimmed[..open_paren].trim().to_string();
        println!("Found command_path: '{}'", command_path);
        Some((command_path, std::collections::HashMap::new()))
    } else {
        let command_path = trimmed.to_string();
        println!("Found command_path (no args): '{}'", command_path);
        Some((command_path, std::collections::HashMap::new()))
    }
}

fn main() {
    let test = "setup_env(env_name=\"test\")";
    println!("Testing: '{}'", test);
    parse_command_call(test);
}
