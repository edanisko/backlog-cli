use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, stdout};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "backlog")]
#[command(about = "A simple backlog manager for your repos", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item to the backlog
    Add {
        /// The backlog item description
        #[arg(trailing_var_arg = true)]
        description: Vec<String>,
    },
    /// List backlog items (current repo or all)
    List {
        /// Show all backlogs across all repos
        #[arg(short, long)]
        all: bool,
    },
    /// Mark an item as done
    Done {
        /// Item number to mark as done
        number: usize,
    },
    /// Remove an item from the backlog
    Remove {
        /// Item number to remove
        number: usize,
    },
    /// Show what to do next (first incomplete item)
    Next,
    /// Interactive CLI mode
    Cli,
}

#[derive(Serialize, Deserialize, Clone)]
struct BacklogItem {
    description: String,
    created_at: DateTime<Utc>,
    done: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct Backlog {
    items: Vec<BacklogItem>,
}

#[derive(Serialize, Deserialize, Default)]
struct GlobalIndex {
    /// Maps repo paths to their backlog file paths
    repos: Vec<String>,
}

fn get_repo_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    let mut dir = current_dir.as_path();

    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

fn get_repo_backlog_path() -> Option<PathBuf> {
    let repo_root = get_repo_root()?;
    Some(repo_root.join(".todo").join("backlog.json"))
}

fn get_global_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".backlog")
}

fn get_global_index_path() -> PathBuf {
    get_global_dir().join("index.json")
}

fn load_backlog(path: &PathBuf) -> Backlog {
    if path.exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Backlog::default()
    }
}

fn save_backlog(path: &PathBuf, backlog: &Backlog) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(backlog)?;
    fs::write(path, content)
}

fn load_global_index() -> GlobalIndex {
    let path = get_global_index_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        GlobalIndex::default()
    }
}

fn save_global_index(index: &GlobalIndex) -> std::io::Result<()> {
    let path = get_global_index_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(index)?;
    fs::write(path, content)
}

fn register_repo(repo_path: &str) {
    let mut index = load_global_index();
    if !index.repos.contains(&repo_path.to_string()) {
        index.repos.push(repo_path.to_string());
        let _ = save_global_index(&index);
    }
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Edit,
    ConfirmDelete,
}

struct App {
    backlog: Backlog,
    backlog_path: PathBuf,
    selected: usize,
    mode: Mode,
    edit_buffer: String,
    edit_cursor: usize,
    output: Option<String>,
    pending_d: bool, // for dd delete
}

impl App {
    fn new(backlog: Backlog, backlog_path: PathBuf) -> Self {
        Self {
            backlog,
            backlog_path,
            selected: 0,
            mode: Mode::Normal,
            edit_buffer: String::new(),
            edit_cursor: 0,
            output: None,
            pending_d: false,
        }
    }

    fn save(&self) -> io::Result<()> {
        save_backlog(&self.backlog_path, &self.backlog)
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        if !self.backlog.items.is_empty() && self.selected < self.backlog.items.len() - 1 {
            self.selected += 1;
        }
    }

    fn toggle_done(&mut self) {
        if !self.backlog.items.is_empty() {
            self.backlog.items[self.selected].done = !self.backlog.items[self.selected].done;
            let _ = self.save();
        }
    }

    fn move_item_up(&mut self) {
        if self.selected > 0 {
            self.backlog.items.swap(self.selected, self.selected - 1);
            self.selected -= 1;
            let _ = self.save();
        }
    }

    fn move_item_down(&mut self) {
        if !self.backlog.items.is_empty() && self.selected < self.backlog.items.len() - 1 {
            self.backlog.items.swap(self.selected, self.selected + 1);
            self.selected += 1;
            let _ = self.save();
        }
    }

    fn enter_edit_mode(&mut self) {
        if !self.backlog.items.is_empty() {
            self.edit_buffer = self.backlog.items[self.selected].description.clone();
            self.edit_cursor = self.edit_buffer.len();
            self.mode = Mode::Edit;
        }
    }

    fn confirm_edit(&mut self) {
        if !self.backlog.items.is_empty() {
            self.backlog.items[self.selected].description = self.edit_buffer.clone();
            let _ = self.save();
        }
        self.mode = Mode::Normal;
    }

    fn cancel_edit(&mut self) {
        self.mode = Mode::Normal;
    }

    fn select_item(&mut self) {
        if !self.backlog.items.is_empty() {
            self.output = Some(self.backlog.items[self.selected].description.clone());
        }
    }

    fn delete_selected(&mut self) {
        if !self.backlog.items.is_empty() {
            self.backlog.items.remove(self.selected);
            if self.selected >= self.backlog.items.len() && self.selected > 0 {
                self.selected -= 1;
            }
            let _ = self.save();
        }
        self.mode = Mode::Normal;
    }
}

/// A custom widget for rendering the backlog list with wrapped items
struct BacklogList<'a> {
    items: &'a [BacklogItem],
    selected: usize,
}

impl<'a> BacklogList<'a> {
    fn new(items: &'a [BacklogItem], selected: usize) -> Self {
        Self { items, selected }
    }
}

impl Widget for BacklogList<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default().borders(Borders::ALL).title("Backlog");
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 10 || inner.height < 1 {
            return;
        }

        let prefix_width: u16 = 8; // "1. [x] " = 8 chars
        let text_width = inner.width.saturating_sub(prefix_width) as usize;

        let mut y = 0u16;
        for (i, item) in self.items.iter().enumerate() {
            if y >= inner.height {
                break;
            }

            let checkbox = if item.done { "[x]" } else { "[ ]" };
            let prefix = format!("{}. {} ", i + 1, checkbox);

            let style = if i == self.selected {
                if item.done {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().add_modifier(Modifier::REVERSED)
                }
            } else if item.done {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            // Wrap the description text
            let desc_chars: Vec<char> = item.description.chars().collect();
            let lines: Vec<String> = if text_width > 0 && !desc_chars.is_empty() {
                desc_chars
                    .chunks(text_width)
                    .map(|chunk| chunk.iter().collect())
                    .collect()
            } else {
                vec![item.description.clone()]
            };

            for (line_idx, line_text) in lines.iter().enumerate() {
                if y >= inner.height {
                    break;
                }

                let x_start = inner.x;
                let y_pos = inner.y + y;

                // Render prefix only on first line
                if line_idx == 0 {
                    for (j, ch) in prefix.chars().enumerate() {
                        if (j as u16) < prefix_width {
                            buf[(x_start + j as u16, y_pos)]
                                .set_char(ch)
                                .set_style(style);
                        }
                    }
                } else {
                    // Indent continuation lines
                    for j in 0..prefix_width {
                        buf[(x_start + j, y_pos)].set_char(' ').set_style(style);
                    }
                }

                // Render the text portion
                for (j, ch) in line_text.chars().enumerate() {
                    let x_pos = x_start + prefix_width + j as u16;
                    if x_pos < inner.x + inner.width {
                        buf[(x_pos, y_pos)].set_char(ch).set_style(style);
                    }
                }

                // Fill remaining width with style (for reversed highlight)
                let text_end = prefix_width + line_text.chars().count() as u16;
                for j in text_end..inner.width {
                    buf[(x_start + j, y_pos)].set_char(' ').set_style(style);
                }

                y += 1;
            }
        }
    }
}

fn run_tui(backlog_path: PathBuf) -> io::Result<Option<String>> {
    let backlog = load_backlog(&backlog_path);

    if backlog.items.is_empty() {
        return Ok(None);
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(backlog, backlog_path);

    loop {
        terminal.draw(|f| {
            let constraints = if app.mode == Mode::Edit {
                vec![
                    Constraint::Min(3),
                    Constraint::Length(5),
                    Constraint::Length(3),
                ]
            } else {
                vec![Constraint::Min(3), Constraint::Length(3)]
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(f.area());

            let list = BacklogList::new(&app.backlog.items, app.selected);
            f.render_widget(list, chunks[0]);

            if app.mode == Mode::Edit {
                let before_cursor: String =
                    app.edit_buffer.chars().take(app.edit_cursor).collect();
                let cursor_char: String = app
                    .edit_buffer
                    .chars()
                    .skip(app.edit_cursor)
                    .take(1)
                    .collect();
                let after_cursor: String =
                    app.edit_buffer.chars().skip(app.edit_cursor + 1).collect();

                let cursor_display = if cursor_char.is_empty() {
                    " ".to_string()
                } else {
                    cursor_char
                };

                let edit_text = Line::from(vec![
                    Span::raw(before_cursor),
                    Span::styled(
                        cursor_display,
                        Style::default().bg(Color::White).fg(Color::Black),
                    ),
                    Span::raw(after_cursor),
                ]);

                let edit_box = Paragraph::new(edit_text)
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("Edit"));
                f.render_widget(edit_box, chunks[1]);
            }

            let help_chunk = if app.mode == Mode::Edit {
                chunks[2]
            } else {
                chunks[1]
            };

            let help_text = match app.mode {
                Mode::Edit => "Enter:confirm  Esc:cancel",
                Mode::ConfirmDelete => "Delete item? y:yes  n/Esc:cancel",
                Mode::Normal => "j/k:nav  Enter:select  x:toggle  e:edit  dd/Del:delete  K/J:move  q:quit",
            };
            let help_style = if app.mode == Mode::ConfirmDelete {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let help = Paragraph::new(help_text)
                .style(help_style)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, help_chunk);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.mode {
                Mode::Normal => {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => break,
                        (KeyCode::Char('J'), m) if m.contains(KeyModifiers::SHIFT) => {
                            app.move_item_down();
                            app.pending_d = false;
                        }
                        (KeyCode::Char('K'), m) if m.contains(KeyModifiers::SHIFT) => {
                            app.move_item_up();
                            app.pending_d = false;
                        }
                        (KeyCode::Down, m) if m.contains(KeyModifiers::SHIFT) => {
                            app.move_item_down();
                            app.pending_d = false;
                        }
                        (KeyCode::Up, m) if m.contains(KeyModifiers::SHIFT) => {
                            app.move_item_up();
                            app.pending_d = false;
                        }
                        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                            app.move_down();
                            app.pending_d = false;
                        }
                        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                            app.move_up();
                            app.pending_d = false;
                        }
                        (KeyCode::Char('x'), _) => {
                            app.toggle_done();
                            app.pending_d = false;
                        }
                        (KeyCode::Char('e'), _) => {
                            app.enter_edit_mode();
                            app.pending_d = false;
                        }
                        (KeyCode::Char('d'), _) => {
                            if app.pending_d {
                                // dd - delete immediately
                                app.delete_selected();
                                app.pending_d = false;
                            } else {
                                app.pending_d = true;
                            }
                        }
                        (KeyCode::Delete, _) | (KeyCode::Backspace, _) => {
                            // Delete/Backspace key - requires confirmation
                            app.mode = Mode::ConfirmDelete;
                            app.pending_d = false;
                        }
                        (KeyCode::Enter, _) => {
                            app.select_item();
                            break;
                        }
                        _ => {
                            app.pending_d = false;
                        }
                    }
                }
                Mode::ConfirmDelete => match key.code {
                    KeyCode::Char('y') => app.delete_selected(),
                    KeyCode::Char('n') | KeyCode::Esc => app.mode = Mode::Normal,
                    _ => {}
                },
                Mode::Edit => match key.code {
                    KeyCode::Enter => app.confirm_edit(),
                    KeyCode::Esc => app.cancel_edit(),
                    KeyCode::Backspace => {
                        if app.edit_cursor > 0 {
                            let mut chars: Vec<char> = app.edit_buffer.chars().collect();
                            chars.remove(app.edit_cursor - 1);
                            app.edit_buffer = chars.into_iter().collect();
                            app.edit_cursor -= 1;
                        }
                    }
                    KeyCode::Delete => {
                        let chars: Vec<char> = app.edit_buffer.chars().collect();
                        if app.edit_cursor < chars.len() {
                            let mut chars = chars;
                            chars.remove(app.edit_cursor);
                            app.edit_buffer = chars.into_iter().collect();
                        }
                    }
                    KeyCode::Left => {
                        if app.edit_cursor > 0 {
                            app.edit_cursor -= 1;
                        }
                    }
                    KeyCode::Right => {
                        let len = app.edit_buffer.chars().count();
                        if app.edit_cursor < len {
                            app.edit_cursor += 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        let mut chars: Vec<char> = app.edit_buffer.chars().collect();
                        chars.insert(app.edit_cursor, c);
                        app.edit_buffer = chars.into_iter().collect();
                        app.edit_cursor += 1;
                    }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(app.output)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { description }) => {
            let Some(backlog_path) = get_repo_backlog_path() else {
                eprintln!("Not in a git repository");
                std::process::exit(1);
            };

            let desc = description.join(" ");
            if desc.is_empty() {
                eprintln!("Please provide a description");
                std::process::exit(1);
            }

            let mut backlog = load_backlog(&backlog_path);
            backlog.items.push(BacklogItem {
                description: desc.clone(),
                created_at: Utc::now(),
                done: false,
            });

            if let Err(e) = save_backlog(&backlog_path, &backlog) {
                eprintln!("Failed to save backlog: {}", e);
                std::process::exit(1);
            }

            // Register this repo in the global index
            if let Some(repo_root) = get_repo_root() {
                register_repo(&repo_root.to_string_lossy());
            }

            println!("Added: {}", desc);
        }

        Some(Commands::List { all }) => {
            if all {
                let index = load_global_index();
                if index.repos.is_empty() {
                    println!("No backlogs found.");
                    return;
                }

                for repo_path in &index.repos {
                    let backlog_file = PathBuf::from(repo_path).join(".todo").join("backlog.json");
                    let backlog = load_backlog(&backlog_file);

                    let pending: Vec<_> = backlog.items.iter().filter(|i| !i.done).collect();
                    if pending.is_empty() {
                        continue;
                    }

                    println!("\n{}", repo_path);
                    println!("{}", "-".repeat(repo_path.len()));
                    for (i, item) in backlog.items.iter().enumerate() {
                        let status = if item.done { "[x]" } else { "[ ]" };
                        println!("  {}. {} {}", i + 1, status, item.description);
                    }
                }
                println!();
            } else {
                let Some(backlog_path) = get_repo_backlog_path() else {
                    eprintln!("Not in a git repository");
                    std::process::exit(1);
                };

                let backlog = load_backlog(&backlog_path);
                if backlog.items.is_empty() {
                    println!("Backlog is empty.");
                    return;
                }

                println!("\nBacklog:");
                println!("--------");
                for (i, item) in backlog.items.iter().enumerate() {
                    let status = if item.done { "[x]" } else { "[ ]" };
                    println!("{}. {} {}", i + 1, status, item.description);
                }
                println!();
            }
        }

        Some(Commands::Done { number }) => {
            let Some(backlog_path) = get_repo_backlog_path() else {
                eprintln!("Not in a git repository");
                std::process::exit(1);
            };

            let mut backlog = load_backlog(&backlog_path);
            if number == 0 || number > backlog.items.len() {
                eprintln!("Invalid item number");
                std::process::exit(1);
            }

            backlog.items[number - 1].done = true;
            if let Err(e) = save_backlog(&backlog_path, &backlog) {
                eprintln!("Failed to save backlog: {}", e);
                std::process::exit(1);
            }

            println!("Marked as done: {}", backlog.items[number - 1].description);
        }

        Some(Commands::Remove { number }) => {
            let Some(backlog_path) = get_repo_backlog_path() else {
                eprintln!("Not in a git repository");
                std::process::exit(1);
            };

            let mut backlog = load_backlog(&backlog_path);
            if number == 0 || number > backlog.items.len() {
                eprintln!("Invalid item number");
                std::process::exit(1);
            }

            let removed = backlog.items.remove(number - 1);
            if let Err(e) = save_backlog(&backlog_path, &backlog) {
                eprintln!("Failed to save backlog: {}", e);
                std::process::exit(1);
            }

            println!("Removed: {}", removed.description);
        }

        Some(Commands::Next) => {
            let Some(backlog_path) = get_repo_backlog_path() else {
                eprintln!("Not in a git repository");
                std::process::exit(1);
            };

            let backlog = load_backlog(&backlog_path);
            let next = backlog.items.iter().find(|i| !i.done);

            match next {
                Some(item) => println!("{}", item.description),
                None => {
                    eprintln!("All done! Backlog is clear.");
                }
            }
        }

        Some(Commands::Cli) => {
            let Some(backlog_path) = get_repo_backlog_path() else {
                eprintln!("Not in a git repository");
                std::process::exit(1);
            };

            match run_tui(backlog_path) {
                Ok(Some(output)) => println!("{}", output),
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        None => {
            // Default: show backlog for current repo
            let Some(backlog_path) = get_repo_backlog_path() else {
                eprintln!("Not in a git repository. Use 'backlog --help' for usage.");
                std::process::exit(1);
            };

            let backlog = load_backlog(&backlog_path);
            if backlog.items.is_empty() {
                println!("Backlog is empty. Use 'backlog add <description>' to add items.");
                return;
            }

            let pending: Vec<_> = backlog.items.iter().filter(|i| !i.done).collect();
            if pending.is_empty() {
                println!("All done! Backlog is clear.");
            } else {
                println!("\n{} item(s) in backlog:", pending.len());
                for (i, item) in backlog.items.iter().enumerate() {
                    if !item.done {
                        let status = "[ ]";
                        println!("{}. {} {}", i + 1, status, item.description);
                    }
                }
                println!();
            }
        }
    }
}
