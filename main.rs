use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nest_core::nestparse::ast::Command;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, path::Path};

enum InputMode {
    Normal,
    // Add other modes if needed (e.g. searching)
}

struct App {
    mode: InputMode,
    commands: Vec<Command>,
    state: ListState,
    command_paths: Vec<(String, Command)>, // Flattened list for display: (Display Path, Command)
    should_quit: bool,
}

impl App {
    fn new(commands: Vec<Command>) -> App {
        let mut app = App {
            mode: InputMode::Normal,
            commands,
            state: ListState::default(),
            command_paths: Vec::new(),
            should_quit: false,
        };
        app.flatten_commands();
        if !app.command_paths.is_empty() {
            app.state.select(Some(0));
        }
        app
    }

    fn flatten_commands(&mut self) {
        self.command_paths.clear();
        let commands = self.commands.clone();
        for cmd in &commands {
            self.flatten_recursive(cmd, &[]);
        }
    }

    fn flatten_recursive(&mut self, cmd: &Command, parent_path: &[String]) {
        let mut current_path = parent_path.to_vec();
        current_path.push(cmd.name.clone());
        
        let display_path = current_path.join(" ");
        self.command_paths.push((display_path.clone(), cmd.clone()));

        for child in &cmd.children {
            self.flatten_recursive(child, &current_path);
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.command_paths.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.command_paths.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Load Nestfile
    // Logic similar to nest-cli/main.rs but simplified for now
    let current_dir = std::env::current_dir()?;
    let nestfile_path = find_nestfile(&current_dir).ok_or("Nestfile not found")?;
    
    let content = nest_core::nestparse::file::read_file_unchecked(&nestfile_path)?;
    // Process includes logic is needed if we want full support...
    // But process_includes is in nestparse::include
    let mut visited = std::collections::HashSet::new();
    let processed_content = nest_core::nestparse::include::process_includes(&content, &nestfile_path, &mut visited)
        .map_err(|e| format!("Error processing includes: {}", e))?;

    let mut parser = nest_core::nestparse::parser::Parser::new(&processed_content);
    let parse_result = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App
    let mut app = App::new(parse_result.commands);

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

fn find_nestfile(dir: &Path) -> Option<std::path::PathBuf> {
    let filenames = ["nestfile", "Nestfile", "nest", "Nest"];
    for name in filenames {
        let path = dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                         // TODO: Execute command
                    }
                    _ => {}
                }
            }
        }
        if app.should_quit {
             return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.area());

    // Command List
    let items: Vec<ListItem> = app.command_paths
        .iter()
        .map(|(path, _)| {
            ListItem::new(Line::from(path.as_str()))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    // Details Pane
    if let Some(selected_idx) = app.state.selected() {
        if let Some((_, cmd)) = app.command_paths.get(selected_idx) {
            let desc = cmd.directives.iter().find_map(|d| match d {
                nest_core::nestparse::ast::Directive::Desc(s) => Some(s.clone()),
                _ => None,
            }).unwrap_or_else(|| "No description".to_string());

            let text = vec![
                Line::from(Span::styled(format!("Command: {}", cmd.name), Style::default().add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("Description:", Style::default().fg(Color::Cyan))),
                Line::from(desc),
                Line::from(""),
                // We could show more info like params, deps, etc.
            ];

            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Details"))
                .wrap(ratatui::widgets::Wrap { trim: true });
            
            f.render_widget(paragraph, chunks[1]);
        }
    }
}
