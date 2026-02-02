use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nest_core::nestparse::ast::Command;
use nest_core::nestparse::validator::{print_validation_errors, validate_commands};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, path::Path, process};

enum InputMode {
    Normal,
    Editing,
    Search,
    EditingArg,
}

#[derive(PartialEq)]
enum ViewMode {
    Tree,
    Flat,
    History,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Focus {
    CommandList,
    ArgumentList,
    History,
}

struct App {
    mode: InputMode,
    view_mode: ViewMode,
    focus: Focus,
    input_buffer: String,

    // Tree Navigation State
    root_commands: Vec<Command>,
    flat_commands: Vec<(String, Command)>, // Cache for Flat View
    breadcrumbs: Vec<String>,              // Function/Command names path
    selection_history: Vec<usize>,         // To restore selection when going up
    args_map: std::collections::HashMap<String, String>,

    // Feature States
    show_source: bool,
    source_code: Option<String>,
    search_query: String,
    filtered_commands: Vec<(String, Command)>,

    // History State
    history: Vec<String>,
    history_state: ListState,
    history_path: std::path::PathBuf,

    state: ListState,
    arg_state: ListState,
    should_quit: bool,
    nestfile_path: std::path::PathBuf,
}

impl App {
    fn new(commands: Vec<Command>, nestfile_path: std::path::PathBuf) -> App {
        let history_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("nest")
            .join("history");

        let mut app = App {
            mode: InputMode::Normal,
            view_mode: ViewMode::Tree,
            focus: Focus::CommandList,
            input_buffer: String::new(),
            root_commands: commands,
            flat_commands: Vec::new(),
            breadcrumbs: Vec::new(),
            selection_history: Vec::new(),
            args_map: std::collections::HashMap::new(),
            show_source: false,
            source_code: None,
            search_query: String::new(),
            filtered_commands: Vec::new(),

            history: Vec::new(),
            history_state: ListState::default(),
            history_path,

            state: ListState::default(),
            arg_state: ListState::default(),
            should_quit: false,
            nestfile_path,
        };
        app.load_history();
        app.flatten_commands();
        // Select first item by default
        if !app.root_commands.is_empty() {
            app.state.select(Some(0));
        }
        app
    }

    fn flatten_commands(&mut self) {
        self.flat_commands.clear();
        let commands = self.root_commands.clone();
        for cmd in &commands {
            self.flatten_recursive(cmd, &[]);
        }
    }

    fn flatten_recursive(&mut self, cmd: &Command, parent_path: &[String]) {
        let mut current_path = parent_path.to_vec();
        current_path.push(cmd.name.clone());

        // Only add leaf commands to flat view? Or all?
        // Typically flat view is for running commands, so mostly leafs.
        // But if groups have scripts attached, they are also commands.
        // Let's add everything that is runnable.
        // For simplicity, let's add everything for now, or just leafs if user prefers "all commands at once".
        // Let's add everything.
        let display_path = current_path.join(" ");
        self.flat_commands.push((display_path.clone(), cmd.clone()));

        for child in &cmd.children {
            self.flatten_recursive(child, &current_path);
        }
    }

    fn load_history(&mut self) {
        if let Ok(content) = std::fs::read_to_string(&self.history_path) {
            self.history = content.lines().map(|s| s.to_string()).collect();
            // Reverse to show newest at top? Or standard?
            // Usually history file is append, so newest at bottom.
            // But UI might want newest at top.
            // Let's keep order as file (oldest first) but render or scrolling properly.
            // Actually, recent at top is better for quick access.
            self.history.reverse();
        }
    }

    fn add_history(&mut self, cmd: String) {
        // Remove duplicates
        if let Some(pos) = self.history.iter().position(|x| x == &cmd) {
            self.history.remove(pos);
        }
        self.history.insert(0, cmd.clone());
        if self.history.len() > 50 {
            self.history.pop();
        }
        self.save_history();
    }

    fn save_history(&self) {
        if let Some(parent) = self.history_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Save in reverse (oldest first) for standard history files
        let mut to_save = self.history.clone();
        to_save.reverse();
        let content = to_save.join("\n");
        let _ = std::fs::write(&self.history_path, content);
    }

    /// Returns the list of commands at the current depth
    fn get_current_items(&self) -> Vec<&Command> {
        let mut current_level = &self.root_commands;

        for step in &self.breadcrumbs {
            if let Some(cmd) = current_level.iter().find(|c| &c.name == step) {
                current_level = &cmd.children;
            } else {
                // path invalid, fallback to empty or root?
                return Vec::new();
            }
        }
        current_level.iter().collect()
    }

    fn resolve_command_from_string(&self, cmd_str: &str) -> Option<&Command> {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        let mut best_match: Option<&Command> = None;
        let mut current_level = &self.root_commands;

        for part in parts {
            if let Some(cmd) = current_level.iter().find(|c| c.name == part) {
                best_match = Some(cmd);
                current_level = &cmd.children;
            } else {
                // Once we stop matching (hit an argument), we stop traversing
                break;
            }
        }
        best_match
    }

    fn get_selected_command(&self) -> Option<&Command> {
        // If searching, use filtered commands
        if !self.search_query.is_empty() {
            if let Some(idx) = self.state.selected() {
                if let Some((_, cmd)) = self.filtered_commands.get(idx) {
                    return Some(cmd);
                }
            }
            return None;
        }

        match self.view_mode {
            ViewMode::Tree => {
                let items = self.get_current_items();
                if let Some(idx) = self.state.selected() {
                    if idx < items.len() {
                        return Some(items[idx]);
                    }
                }
            }
            ViewMode::Flat => {
                if let Some(idx) = self.state.selected() {
                    if let Some((_, cmd)) = self.flat_commands.get(idx) {
                        return Some(cmd);
                    }
                }
            }
            ViewMode::History => {
                if let Some(idx) = self.history_state.selected() {
                    if let Some(cmd_str) = self.history.get(idx) {
                        return self.resolve_command_from_string(cmd_str);
                    }
                }
                return None;
            }
        }
        None
    }

    fn load_selected_source(&mut self) {
        if let Some(cmd) = self.get_selected_command().cloned() {
            if let Some(path) = &cmd.source_file {
                if let Ok(content) = std::fs::read_to_string(path) {
                    // Heuristic extraction: find line with "name:" or "name("
                    // This is simple and improves later.
                    // For now, let's just show the first 20 lines matching the command pattern or just the whole file?
                    // Whole file is too big.
                    // Let's try to find the start line.
                    let lines: Vec<&str> = content.lines().collect();
                    let mut start_line = 0;
                    let mut found = false;

                    // Search pattern: "command_name" followed by various potential chars
                    for (i, line) in lines.iter().enumerate() {
                        let trimmed = line.trim();
                        if trimmed.starts_with(&cmd.name) {
                            start_line = i;
                            found = true;
                            break;
                        }
                    }

                    if found {
                        // Take ~20 lines
                        let end_line = std::cmp::min(start_line + 20, lines.len());
                        self.source_code = Some(lines[start_line..end_line].join("\n"));
                    } else {
                        self.source_code = Some("Could not find definition in file.".to_string());
                    }
                }
            } else {
                self.source_code = Some("No source file information available.".to_string());
            }
        }
    }

    fn current_args(&self) -> Vec<nest_core::nestparse::ast::Parameter> {
        self.get_selected_command()
            .map(|cmd| cmd.parameters.clone())
            .unwrap_or_default()
    }

    fn update_input_buffer(&mut self) {
        if let Some(cmd) = self.get_selected_command() {
            let mut full_cmd = if self.view_mode != ViewMode::History {
                self.breadcrumbs.join(" ")
            } else {
                // In History mode, we might want to respect the history command string base?
                // But for now, let's treat it as rebuilding from the resolved command.
                // Actually, if we are in History mode, get_selected_command resolves the command struct.
                // We should probably reconstruct the path from the resolved command if possible,
                // or just use the history string?
                // Issue: History string has args. args_map might be empty initially.
                // Simple approach: Always rebuild from scratch using breadcrumbs if available?
                // But History command might not match current breadcrumbs.
                // Let's assume breadcrumbs are correct for Tree/Flat.
                // For History, we might need a way to get the full path of the resolved command.
                // Since we don't store parent pointers, we can't easily walk up.
                // Fallback: Use cmd.name.
                cmd.name.clone()
            };

            if self.view_mode != ViewMode::History {
                if !full_cmd.is_empty() {
                    full_cmd.push(' ');
                }
                full_cmd.push_str(&cmd.name);
            }

            // Append arguments from args_map
            for param in &cmd.parameters {
                if let Some(val) = self.args_map.get(&param.name) {
                    // Check if bool flag
                    if param.param_type == "bool" {
                        if val == "true" {
                            full_cmd.push(' ');
                            if param.is_named {
                                full_cmd.push_str("--");
                                full_cmd.push_str(&param.name);
                            }
                        }
                    } else {
                        // String/Num/etc
                        if !val.is_empty() {
                            full_cmd.push(' ');
                            if param.is_named {
                                full_cmd.push_str("--");
                                full_cmd.push_str(&param.name);
                                full_cmd.push(' ');
                                full_cmd.push_str(val);
                            } else {
                                full_cmd.push_str(val);
                            }
                        }
                    }
                }
            }
            self.input_buffer = full_cmd;
        }
    }

    fn toggle_view(&mut self) {
        match self.view_mode {
            ViewMode::Tree => self.view_mode = ViewMode::Flat,
            ViewMode::Flat => self.view_mode = ViewMode::Tree,
            ViewMode::History => {
                self.view_mode = ViewMode::Tree;
                self.focus = Focus::CommandList;
            }
        }
        self.state.select(Some(0));
        self.arg_state.select(None);
        self.focus = Focus::CommandList;
    }

    fn next(&mut self) {
        let count = if !self.search_query.is_empty() {
            self.filtered_commands.len()
        } else {
            match self.view_mode {
                ViewMode::Tree => self.get_current_items().len(),
                ViewMode::Flat => self.flat_commands.len(),
                ViewMode::History => 0,
            }
        };
        if count == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.args_map.clear();
    }

    fn previous(&mut self) {
        let count = if !self.search_query.is_empty() {
            self.filtered_commands.len()
        } else {
            match self.view_mode {
                ViewMode::Tree => self.get_current_items().len(),
                ViewMode::Flat => self.flat_commands.len(),
                ViewMode::History => 0,
            }
        };
        if count == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.args_map.clear();
    }

    fn update_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_commands.clear();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_commands = self
                .flat_commands
                .iter()
                .filter(|(path, _)| path.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }
        if !self.filtered_commands.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    fn next_arg(&mut self) {
        let args_len = self.current_args().len();
        if args_len == 0 {
            return;
        }

        let i = match self.arg_state.selected() {
            Some(i) => {
                if i >= args_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.arg_state.select(Some(i));
    }

    fn previous_arg(&mut self) {
        let args_len = self.current_args().len();
        if args_len == 0 {
            return;
        }

        let i = match self.arg_state.selected() {
            Some(i) => {
                if i == 0 {
                    args_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.arg_state.select(Some(i));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Load Nestfile
    // Logic similar to nest-cli/main.rs but simplified for now
    // 1. Parse arguments for --config / -c, --version, --help
    let args: Vec<String> = std::env::args().collect();
    let mut config_path_arg: Option<String> = None;

    // Check for flags that don't need config
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("nest-ui v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Nest UI - TUI for Nest task runner");
        println!("");
        println!("Usage: nestui [OPTIONS]");
        println!("");
        println!("Options:");
        println!("  -c, --config <PATH>    Path to Nestfile");
        println!("  -V, --version          Show version information");
        println!("  -h, --help             Show this help message");
        println!("");
        return Ok(());
    }

    for (idx, arg) in args.iter().enumerate().skip(1) {
        if arg == "--config" || arg == "-c" {
            if let Some(path) = args.get(idx + 1) {
                config_path_arg = Some(path.clone());
            }
            break;
        }
    }

    let nestfile_path = if let Some(path_str) = config_path_arg {
        let path = Path::new(&path_str).to_path_buf();
        if !path.exists() {
            nest_core::nestparse::output::OutputFormatter::error(&format!(
                "Configuration file not found: {}",
                path_str
            ));
            process::exit(1);
        }
        path
    } else {
        match nest_core::nestparse::path::find_config_file() {
            Some(path) => path,
            None => {
                println!("nestfile not found in current directory");
                println!("Run 'nest --init' to create one, or use '--config <path>'.");
                process::exit(1);
            }
        }
    };

    let content = match nest_core::nestparse::file::read_file_unchecked(&nestfile_path) {
        Ok(c) => c,
        Err(e) => {
            nest_core::nestparse::output::OutputFormatter::error(&format!(
                "Error reading file: {}",
                e
            ));
            process::exit(1);
        }
    };

    // Process includes logic
    let mut visited = std::collections::HashSet::new();
    let processed_content = match nest_core::nestparse::include::process_includes(
        &content,
        &nestfile_path,
        &mut visited,
    ) {
        Ok(c) => c,
        Err(e) => {
            nest_core::nestparse::output::OutputFormatter::error(&format!(
                "Error processing includes: {}",
                e
            ));
            process::exit(1);
        }
    };

    // Prepend root source marker
    let mut full_content = String::new();
    if let Ok(canonical_path) = nestfile_path.canonicalize() {
        full_content.push_str(&format!("# @source: {}\n", canonical_path.display()));
    } else {
        full_content.push_str(&format!("# @source: {}\n", nestfile_path.display()));
    }
    full_content.push_str(&processed_content);

    let mut parser = nest_core::nestparse::parser::Parser::new(&full_content);
    let mut parse_result = match parser.parse() {
        Ok(res) => res,
        Err(e) => {
            let msg = match e {
                nest_core::nestparse::parser::ParseError::UnexpectedEndOfFile(line) => {
                    format!("Parse error at line {}: Unexpected end of file.", line)
                }
                nest_core::nestparse::parser::ParseError::InvalidSyntax(msg, line) => {
                    format!("Parse error at line {}: {}", line, msg)
                }
                nest_core::nestparse::parser::ParseError::InvalidIndent(line) => {
                    format!("Parse error at line {}: Invalid indentation.", line)
                }
                nest_core::nestparse::parser::ParseError::DeprecatedSyntax(msg, line) => {
                    format!("Parse error at line {}: {} (deprecated syntax)", line, msg)
                }
            };
            nest_core::nestparse::output::OutputFormatter::error(&msg);
            process::exit(1);
        }
    };

    // Merge duplicate commands (same as CLI)
    parse_result.commands = nest_core::nestparse::merge::merge_commands(parse_result.commands);

    // Validate configuration using standard nest-core validator
    if let Err(validation_errors) = validate_commands(&parse_result.commands, &nestfile_path) {
        print_validation_errors(&validation_errors, &nestfile_path);
        process::exit(1);
    }

    // Populate source file path recursively
    resolve_command_sources(&mut parse_result.commands, &full_content, &nestfile_path);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App
    let mut app = App::new(parse_result.commands, nestfile_path);

    // Run loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn resolve_command_sources(commands: &mut Vec<Command>, content: &str, initial_path: &Path) {
    let mut line_map = Vec::new();
    let mut current_path = initial_path.to_path_buf();

    // 1. Build map of line index -> source file
    for line in content.lines() {
        if line.starts_with("# @source: ") {
            let path_str = line[11..].trim();
            current_path = std::path::PathBuf::from(path_str);
        }
        line_map.push(current_path.clone());
    }

    // 2. Resolve recursively
    let indexed_lines: Vec<(usize, &str)> = content.lines().enumerate().collect();
    resolve_scope(commands, &indexed_lines, &line_map);
}

fn resolve_scope(
    commands: &mut Vec<Command>,
    lines: &[(usize, &str)],
    line_map: &[std::path::PathBuf],
) {
    for cmd in commands {
        // Find definition line
        for (idx, (line_num, line)) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for name match at start of line
            if trimmed.starts_with(&cmd.name) {
                let remainder = &trimmed[cmd.name.len()..];
                // Check valid endings: ":", "(...", " :"
                if remainder.trim_start().starts_with(':') || remainder.starts_with('(') {
                    // Found it!
                    if let Some(path) = line_map.get(*line_num) {
                        cmd.source_file = Some(path.clone());
                    }

                    // Recurse children with lines AFTER definition
                    if !cmd.children.is_empty() {
                        let next_lines = &lines[idx + 1..];
                        resolve_scope(&mut cmd.children, next_lines, line_map);
                    }
                    break; // Found this command, move to next sibling
                }
            }
        }
    }
}

use std::io::Write;

fn execute_shell_command<B: Backend + Write>(
    terminal: &mut Terminal<B>,
    command_str: &str,
    nestfile_path: &Path,
) -> io::Result<()> {
    // Suspend TUI
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let absolute_config = nestfile_path
        .canonicalize()
        .unwrap_or_else(|_| nestfile_path.to_path_buf());
    let working_dir = absolute_config.parent().unwrap_or(Path::new("."));

    println!(
        "Executing: nest --config {} {}",
        absolute_config.display(),
        command_str
    );

    // Run command
    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if !parts.is_empty() {
        let mut final_args = vec!["--config", absolute_config.to_str().unwrap_or("")];
        final_args.extend_from_slice(&parts);

        let status = std::process::Command::new("nest")
            .current_dir(working_dir)
            .args(&final_args)
            .status();

        match status {
            Ok(s) => {
                if !s.success() {
                    println!("Command failed with exit code: {:?}", s.code());
                }
            }
            Err(e) => println!("Failed to execute command: {}", e),
        }
    }

    println!("\nPress Enter to return...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Restore TUI
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    Ok(())
}

fn run_app<B: Backend + Write>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                            KeyCode::Tab => match app.focus {
                                Focus::CommandList => {
                                    app.focus = Focus::History;
                                    if app.history_state.selected().is_none()
                                        && !app.history.is_empty()
                                    {
                                        app.history_state.select(Some(0));
                                    }
                                }
                                Focus::History => {
                                    app.focus = Focus::ArgumentList;
                                    if app.arg_state.selected().is_none()
                                        && !app.current_args().is_empty()
                                    {
                                        app.arg_state.select(Some(0));
                                    }
                                    app.update_input_buffer();
                                }
                                Focus::ArgumentList => app.focus = Focus::CommandList,
                            },
                            KeyCode::Right => {
                                if app.view_mode == ViewMode::Tree {
                                    if let Some(cmd) = app.get_selected_command() {
                                        if !cmd.children.is_empty() {
                                            app.breadcrumbs.push(cmd.name.clone());
                                            app.selection_history
                                                .push(app.state.selected().unwrap_or(0));
                                            app.state.select(Some(0));
                                            app.args_map.clear();
                                        } else {
                                            app.focus = Focus::ArgumentList;
                                            app.update_input_buffer();
                                        }
                                    }
                                } else {
                                    app.focus = Focus::ArgumentList;
                                }
                            }
                            KeyCode::Left | KeyCode::Backspace => {
                                match app.focus {
                                    Focus::CommandList => {
                                        if app.view_mode == ViewMode::Tree {
                                            if !app.breadcrumbs.is_empty() {
                                                app.breadcrumbs.pop();
                                                if let Some(prev_idx) = app.selection_history.pop()
                                                {
                                                    app.state.select(Some(prev_idx));
                                                } else {
                                                    app.state.select(Some(0));
                                                }
                                                app.args_map.clear();
                                            }
                                        }
                                    }
                                    Focus::History => app.focus = Focus::CommandList,
                                    Focus::ArgumentList => app.focus = Focus::CommandList, // command list is central
                                }
                            }
                            KeyCode::Char('v') => {
                                app.toggle_view();
                            }
                            KeyCode::Char('h') => {
                                if app.view_mode == ViewMode::History {
                                    app.view_mode = ViewMode::Tree;
                                    app.focus = Focus::CommandList;
                                } else {
                                    app.view_mode = ViewMode::History;
                                    app.focus = Focus::History;
                                    // Select first item if none selected
                                    if app.history_state.selected().is_none()
                                        && !app.history.is_empty()
                                    {
                                        app.history_state.select(Some(0));
                                    }
                                }
                            }
                            KeyCode::Char('/') => {
                                app.mode = InputMode::Search;
                                app.search_query.clear();
                                app.update_search();
                            }
                            KeyCode::Char('d') => {
                                app.show_source = !app.show_source;
                                if app.show_source {
                                    app.load_selected_source();
                                }
                            }
                            KeyCode::Down => match app.focus {
                                Focus::CommandList => {
                                    app.next();
                                    app.arg_state.select(None);
                                }
                                Focus::History => {
                                    if !app.history.is_empty() {
                                        let i = match app.history_state.selected() {
                                            Some(i) => {
                                                if i >= app.history.len() - 1 {
                                                    0
                                                } else {
                                                    i + 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        app.history_state.select(Some(i));
                                    }
                                }
                                Focus::ArgumentList => app.next_arg(),
                            },
                            KeyCode::Up => match app.focus {
                                Focus::CommandList => {
                                    app.previous();
                                    app.arg_state.select(None);
                                }
                                Focus::History => {
                                    if !app.history.is_empty() {
                                        let i = match app.history_state.selected() {
                                            Some(i) => {
                                                if i == 0 {
                                                    app.history.len() - 1
                                                } else {
                                                    i - 1
                                                }
                                            }
                                            None => 0,
                                        };
                                        app.history_state.select(Some(i));
                                    }
                                }
                                Focus::ArgumentList => app.previous_arg(),
                            },
                            KeyCode::Char('e') => {
                                if let Some(cmd) = app.get_selected_command() {
                                    let mut full_cmd = app.breadcrumbs.join(" ");
                                    if !full_cmd.is_empty() {
                                        full_cmd.push(' ');
                                    }
                                    full_cmd.push_str(&cmd.name);

                                    app.input_buffer = full_cmd;
                                    app.mode = InputMode::Editing;
                                } else if app.focus == Focus::History {
                                    // edit history item
                                    if let Some(idx) = app.history_state.selected() {
                                        if let Some(cmd_str) = app.history.get(idx) {
                                            app.input_buffer = cmd_str.clone();
                                            app.mode = InputMode::Editing;
                                        }
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                match app.focus {
                                    Focus::CommandList => {
                                        if let Some(cmd) = app.get_selected_command().cloned() {
                                            if !cmd.children.is_empty() {
                                                app.breadcrumbs.push(cmd.name.clone());
                                                app.selection_history
                                                    .push(app.state.selected().unwrap_or(0));
                                                app.state.select(Some(0));
                                            } else {
                                                app.update_input_buffer();
                                                let full_cmd = app.input_buffer.clone();

                                                app.add_history(full_cmd.clone());
                                                execute_shell_command(
                                                    terminal,
                                                    &full_cmd,
                                                    &app.nestfile_path,
                                                )?;
                                            }
                                        }
                                    }
                                    Focus::History => {
                                        // Run selected history item
                                        if let Some(idx) = app.history_state.selected() {
                                            if let Some(cmd_str) = app.history.get(idx).cloned() {
                                                app.add_history(cmd_str.clone());
                                                execute_shell_command(
                                                    terminal,
                                                    &cmd_str,
                                                    &app.nestfile_path,
                                                )?;
                                            }
                                        }
                                    }
                                    Focus::ArgumentList => {
                                        // Interactive Argument Form
                                        let args = app.current_args();
                                        if let Some(idx) = app.arg_state.selected() {
                                            if let Some(arg) = args.get(idx) {
                                                // Check for boolean flag
                                                if arg.param_type == "bool" {
                                                    let current_val = app
                                                        .args_map
                                                        .get(&arg.name)
                                                        .map(|s| s.as_str())
                                                        .unwrap_or("false");
                                                    let new_val = if current_val == "true" {
                                                        "false"
                                                    } else {
                                                        "true"
                                                    };
                                                    app.args_map.insert(
                                                        arg.name.clone(),
                                                        new_val.to_string(),
                                                    );
                                                    app.update_input_buffer();
                                                } else {
                                                    // String/Number - enter editing mode
                                                    let current_val = app
                                                        .args_map
                                                        .get(&arg.name)
                                                        .cloned()
                                                        .unwrap_or_default();
                                                    app.input_buffer = current_val;
                                                    app.mode = InputMode::EditingArg;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        InputMode::Search => match key.code {
                            KeyCode::Enter => {
                                app.mode = InputMode::Normal;
                                app.view_mode = ViewMode::Flat;
                                app.focus = Focus::CommandList;
                                app.args_map.clear();
                            }
                            KeyCode::Esc => {
                                app.mode = InputMode::Normal;
                                app.search_query.clear();
                                app.filtered_commands.clear();
                            }
                            KeyCode::Backspace => {
                                app.search_query.pop();
                                app.update_search();
                            }
                            KeyCode::Char(c) => {
                                app.search_query.push(c);
                                app.update_search();
                            }
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Enter => {
                                let cmd = app.input_buffer.clone();
                                app.add_history(cmd.clone());
                                execute_shell_command(terminal, &cmd, &app.nestfile_path)?;
                                app.mode = InputMode::Normal;
                            }
                            KeyCode::Esc => {
                                app.mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                app.input_buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            _ => {}
                        },
                        InputMode::EditingArg => match key.code {
                            KeyCode::Enter => {
                                // Save value
                                if let Some(idx) = app.arg_state.selected() {
                                    let args = app.current_args();
                                    if let Some(arg) = args.get(idx) {
                                        app.args_map
                                            .insert(arg.name.clone(), app.input_buffer.clone());
                                    }
                                }
                                app.mode = InputMode::Normal;
                                app.update_input_buffer();
                            }
                            KeyCode::Esc => {
                                app.mode = InputMode::Normal;
                                app.update_input_buffer(); // Restore command preview
                            }
                            KeyCode::Char(c) => {
                                app.input_buffer.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            _ => {}
                        },
                    }
                }
            }
            Event::Mouse(mouse) => {
                match mouse.kind {
                    event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        // Placeholder for mouse click logic
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(match app.mode {
                InputMode::Editing | InputMode::EditingArg => 3,
                _ => 1,
            }), // Footer space
        ])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // Left Column: Commands OR History
    if app.view_mode == ViewMode::History {
        let history_items: Vec<ListItem> = app
            .history
            .iter()
            .map(|cmd| ListItem::new(Line::from(cmd.clone())))
            .collect();

        let history_list = List::new(history_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("History")
                    .border_style(match app.focus {
                        Focus::History => Style::default().fg(Color::Green),
                        _ => Style::default(),
                    }),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(history_list, main_chunks[0], &mut app.history_state);
    } else {
        // Command List
        let items: Vec<ListItem> = if !app.search_query.is_empty() {
            app.filtered_commands
                .iter()
                .map(|(path, cmd)| {
                    let style = if !cmd.children.is_empty() {
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(path.clone(), style)))
                })
                .collect()
        } else {
            match app.view_mode {
                ViewMode::Tree => {
                    let current_items = app.get_current_items();
                    current_items
                        .iter()
                        .map(|cmd| {
                            let mut name = cmd.name.clone();
                            let style = if !cmd.children.is_empty() {
                                name.push_str(" /");
                                Style::default()
                                    .fg(Color::Blue)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                            };
                            ListItem::new(Line::from(Span::styled(name, style)))
                        })
                        .collect()
                }
                ViewMode::Flat => app
                    .flat_commands
                    .iter()
                    .map(|(path, cmd)| {
                        let style = if !cmd.children.is_empty() {
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        ListItem::new(Line::from(Span::styled(path.clone(), style)))
                    })
                    .collect(),
                ViewMode::History => Vec::new(), // Should not happen in this branch
            }
        };

        // Title shows current path
        let title = match app.view_mode {
            ViewMode::Tree => {
                if app.breadcrumbs.is_empty() {
                    "Commands (Root)".to_string()
                } else {
                    format!("Commands ({})", app.breadcrumbs.join(" > "))
                }
            }
            ViewMode::Flat => "Commands (All)".to_string(),
            ViewMode::History => "History".to_string(),
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(match app.focus {
                        Focus::CommandList => Style::default().fg(Color::Green),
                        _ => Style::default(),
                    }),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, main_chunks[0], &mut app.state);
    }

    // Right Column: Details (Top) + Arguments (Bottom)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[1]);

    // Source Code Overlay (replaces Details/Args if active)
    if app.show_source {
        let title = if let Some(cmd) = app.get_selected_command() {
            format!("Source: {}", cmd.name)
        } else {
            "Source".to_string()
        };

        let content = app
            .source_code
            .clone()
            .unwrap_or("No source loaded".to_string());
        let p = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(ratatui::widgets::Wrap { trim: false }); // keep formatting

        f.render_widget(p, main_chunks[1]);
    } else {
        // Details Pane
        if let Some(cmd) = app.get_selected_command() {
            let mut text = Vec::new();

            // 1. Header & Signature
            let kind_str = if !cmd.children.is_empty() {
                "Group"
            } else {
                "Command"
            };
            text.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", kind_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    &cmd.name,
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                ),
            ]));
            text.push(Line::from(""));

            // 2. Description
            let desc = cmd.directives.iter().find_map(|d| match d {
                nest_core::nestparse::ast::Directive::Desc(s) => Some(s.clone()),
                _ => None,
            });
            if let Some(d) = desc {
                text.push(Line::from(Span::styled(
                    "Description:",
                    Style::default().fg(Color::Cyan),
                )));
                text.push(Line::from(Span::raw(format!("  {}", d))));
                text.push(Line::from(""));
            }

            // Children (if group)
            if !cmd.children.is_empty() {
                text.push(Line::from(Span::styled(
                    "Subcommands:",
                    Style::default().fg(Color::Cyan),
                )));
                let children_names: Vec<String> =
                    cmd.children.iter().map(|c| c.name.clone()).collect();
                text.push(Line::from(Span::raw(format!(
                    "  {}",
                    children_names.join(", ")
                ))));
                text.push(Line::from(""));
            }

            // Dependencies
            let depends: Vec<String> = cmd
                .directives
                .iter()
                .filter_map(|d| match d {
                    nest_core::nestparse::ast::Directive::Depends(deps, parallel) => {
                        let dep_strs: Vec<String> =
                            deps.iter().map(|dep| dep.command_path.clone()).collect();
                        let p_str = if *parallel { " (parallel)" } else { "" };
                        Some(format!("{}{}", dep_strs.join(", "), p_str))
                    }
                    _ => None,
                })
                .collect();

            if !depends.is_empty() {
                text.push(Line::from(Span::styled(
                    "Dependencies:",
                    Style::default().fg(Color::Cyan),
                )));
                for dep in depends {
                    text.push(Line::from(Span::raw(format!("  {}", dep))));
                }
                text.push(Line::from(""));
            }

            // Variables
            if !cmd.local_variables.is_empty() {
                text.push(Line::from(Span::styled(
                    "Variables:",
                    Style::default().fg(Color::Cyan),
                )));
                for v in &cmd.local_variables {
                    text.push(Line::from(Span::raw(format!("  {} = {}", v.name, v.value))));
                }
                text.push(Line::from(""));
            }

            // Other Info (Source, Cwd, Env)
            let mut info = Vec::new();
            if let Some(src) = &cmd.source_file {
                info.push(format!("Source: {}", src.display()));
            }
            // Check other directives
            for d in &cmd.directives {
                match d {
                    nest_core::nestparse::ast::Directive::Cwd(p) => {
                        info.push(format!("Cwd: {}", p))
                    }
                    nest_core::nestparse::ast::Directive::Env(k, v, hide) => {
                        let display_val = if *hide {
                            "********".to_string()
                        } else {
                            v.clone()
                        };
                        info.push(format!("Env: {}={}", k, display_val))
                    }
                    nest_core::nestparse::ast::Directive::Privileged(true) => {
                        info.push("Privileged: Yes".to_string())
                    }
                    nest_core::nestparse::ast::Directive::Validate(target, rule) => {
                        info.push(format!("Validate {}: {}", target, rule))
                    }
                    _ => {}
                }
            }

            if !info.is_empty() {
                text.push(Line::from(Span::styled(
                    "Info:",
                    Style::default().fg(Color::Cyan),
                )));
                for i in info {
                    text.push(Line::from(Span::raw(format!("  {}", i))));
                }
                text.push(Line::from(""));
            }

            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Details (d for Source)"),
                )
                .wrap(ratatui::widgets::Wrap { trim: true });

            f.render_widget(paragraph, right_chunks[0]);

            // Arguments Pane (Selectable List)
            let current_args = app.current_args();
            let arg_items: Vec<ListItem> = current_args
                .iter()
                .map(|param| {
                    let mut s = String::new();

                    // Check current value
                    let val = app.args_map.get(&param.name).map(|s| s.as_str());

                    if param.param_type == "bool" {
                        let is_checked = val == Some("true");
                        s.push_str(if is_checked { "[x] " } else { "[ ] " });
                        s.push_str("--");
                        s.push_str(&param.name);
                    } else {
                        if param.is_named {
                            s.push_str("--");
                            s.push_str(&param.name);
                        } else {
                            s.push('<');
                            s.push_str(&param.name);
                            s.push('>');
                        }

                        if let Some(v) = val {
                            if !v.is_empty() {
                                s.push_str(": ");
                                s.push_str(v);
                            }
                        }
                    }
                    // Type hint
                    if param.param_type != "bool" {
                        s.push_str(&format!(" ({})", param.param_type));
                    }

                    ListItem::new(Line::from(s))
                })
                .collect();

            let arg_list = List::new(arg_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Arguments (Enter to add)")
                        .border_style(match app.focus {
                            Focus::ArgumentList => Style::default().fg(Color::Green),
                            _ => Style::default(),
                        }),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                )
                .highlight_symbol("> ");

            f.render_stateful_widget(arg_list, right_chunks[1], &mut app.arg_state);
        } // end if let Some(cmd)
    } // end else (not showing source)

    // Input or Help Footer
    match app.mode {
        InputMode::Editing => {
            let input = Paragraph::new(app.input_buffer.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Edit Command"));
            f.render_widget(input, chunks[1]);
            f.set_cursor_position(ratatui::layout::Position {
                x: chunks[1].x + app.input_buffer.len() as u16 + 1,
                y: chunks[1].y + 1,
            });
        }
        InputMode::EditingArg => {
            let input = Paragraph::new(app.input_buffer.as_str())
                .style(Style::default().fg(Color::Magenta))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Edit Argument Value"),
                );
            f.render_widget(input, chunks[1]);
            f.set_cursor_position(ratatui::layout::Position {
                x: chunks[1].x + app.input_buffer.len() as u16 + 1,
                y: chunks[1].y + 1,
            });
        }
        InputMode::Search => {
            let input = Paragraph::new(format!("/{}", app.search_query))
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Search"));
            f.render_widget(input, chunks[1]);
            f.set_cursor_position(ratatui::layout::Position {
                x: chunks[1].x + app.search_query.len() as u16 + 2, // +2 for border and '/'
                y: chunks[1].y + 1,
            });
        }
        InputMode::Normal => {
            let help_text = vec![
                Span::raw("Nav: "),
                Span::styled("Arrows", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | Go In/Run: "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | Go Out: "),
                Span::styled("Left/Bksp", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | View: "),
                Span::styled("v", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | Search: "),
                Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | Src: "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | Edit: "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | Pane: "),
                Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" | Quit: "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            ];
            let help_line = Line::from(help_text);
            let help = Paragraph::new(help_line).style(Style::default().fg(Color::DarkGray));
            f.render_widget(help, chunks[1]);
        }
    }
}
