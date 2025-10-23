use color_eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListDirection, ListState, Paragraph, Widget},
};
use std::fs;
use tui_textarea::TextArea;

use crate::file_ops::{self, Directory};

#[derive(Debug)]
pub struct App {
    pub exit: bool,
    pub dir: Directory,
    pub subdir: Option<Directory>,
    pub list_state: ListState,
    pub show_confirmation: bool,
    pub show_rename: bool,
    pub show_new_file: bool,
    pub new_file_input: TextArea<'static>,
    pub file_to_delete: Option<String>,
    pub file_to_rename: Option<String>,
    pub rename_input: TextArea<'static>,
    pub clipboard: Option<Clipboard>,
}

#[derive(Debug)]
pub struct Clipboard {
    pub cut: bool,
    pub path: String,
}

impl App {
    pub async fn new() -> Self {
        let current_dir = file_ops::get_current_directory().await.unwrap();
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut rename_input = TextArea::default();
        rename_input.set_block(Block::bordered().title("New name"));

        let mut new_file_input = TextArea::default();
        new_file_input.set_block(Block::bordered().title("New name"));

        let mut app = Self {
            exit: false,
            dir: current_dir,
            subdir: None,
            list_state,
            show_rename: false,
            show_confirmation: false,
            file_to_rename: None,
            file_to_delete: None,
            clipboard: None,
            rename_input,
            new_file_input,
            show_new_file: false,
        };

        app.update_subdir_preview_async().await;
        app
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        use ratatui::style::Color;
        use ratatui::widgets::BorderType;

        let items = self.dir.entries_with_symbols();
        
        // Main directory list with styled border
        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(format!(" 📁 {} ", self.dir.path))
                    .title_style(Style::new().bold().cyan())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().cyan())
            )
            .style(Style::new().white())
            .highlight_style(
                Style::new()
                    .bg(Color::Rgb(60, 60, 80))
                    .fg(Color::Rgb(255, 215, 0))
                    .bold()
            )
            .highlight_symbol("▶ ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        let helper_text = Text::from(vec![
            Line::from(vec![
                " q:Quit ".into(),
                "│".dark_gray(),
                " ↑↓/jk:Nav ".into(),
                "│".dark_gray(),
                " ←→/hl:Dir ".into(),
                "│".dark_gray(),
                " Enter:Open ".into(),
                "│".dark_gray(),
                " d:Del ".into(),
                "│".dark_gray(),
                " r:Rename ".into(),
            ]).style(Style::new().fg(Color::Rgb(200, 200, 200))),
            Line::from(vec![
                " y:Yank ".into(),
                "│".dark_gray(),
                " a:New ".into(),
                "│".dark_gray(),
                " c:Copy ".into(),
                "│".dark_gray(),
                " x:Cut ".into(),
                "│".dark_gray(),
                " p:Paste ".into(),
            ]).style(Style::new().fg(Color::Rgb(200, 200, 200))),
        ]);

        // Render main list
        frame.render_stateful_widget(
            list,
            Rect {
                x: 0,
                y: 0,
                width: frame.area().width / 2,
                height: frame.area().height - 3,
            },
            &mut self.list_state,
        );

        // Preview panel
        let items2 = if let Some(subdir) = &self.subdir {
            subdir.entries_with_symbols()
        } else {
            vec!["   No preview available".to_string()]
        };
        
        let preview_title = if let Some(subdir) = &self.subdir {
            format!(" 👁  Preview: {} ", subdir.name)
        } else {
            " 👁  Preview ".to_string()
        };
        
        let list2 = List::new(items2)
            .block(
                Block::bordered()
                    .title(preview_title)
                    .title_style(Style::new().bold().magenta())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().magenta())
            )
            .style(Style::new().fg(Color::Rgb(180, 180, 200)))
            .direction(ListDirection::TopToBottom);

        frame.render_widget(
            list2,
            Rect {
                x: frame.area().width / 2,
                y: 0,
                width: frame.area().width / 2,
                height: frame.area().height - 3,
            },
        );

        // Status bar at bottom
        frame.render_widget(
            Paragraph::new(helper_text)
                .centered()
                .block(
                    Block::bordered()
                        .border_type(BorderType::Double)
                        .border_style(Style::new().green())
                ),
            Rect {
                x: 0,
                y: frame.area().height - 3,
                width: frame.area().width,
                height: 3,
            },
        );

        // Render confirmation overlay if active
        if self.show_confirmation {
            let area = centered_rect(50, 20, frame.area());
            let msg = if let Some(file) = &self.file_to_delete {
                format!("Delete '{}'? (y/n)", file)
            } else {
                "Delete file? (y/n)".to_string()
            };

            let dialog = ConfirmationDialog {
                message: msg,
            };

            frame.render_widget(dialog, area);
        }

        if self.show_rename {
            use ratatui::style::Color;
            use ratatui::widgets::BorderType;
            
            let area = centered_rect(60, 25, frame.area());
            let block = Block::bordered()
                .title(" ✏️  Rename File ")
                .title_style(Style::new().bold().yellow())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().yellow())
                .style(Style::new().bg(Color::Rgb(30, 30, 40)));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            frame.render_widget(&self.rename_input, inner);
        }

        if self.show_new_file {
            use ratatui::style::Color;
            use ratatui::widgets::BorderType;
            
            let area = centered_rect(60, 25, frame.area());
            let block = Block::bordered()
                .title(" ➕ New File ")
                .title_style(Style::new().bold().green())
                .border_type(BorderType::Rounded)
                .border_style(Style::new().green())
                .style(Style::new().bg(Color::Rgb(30, 30, 40)));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            frame.render_widget(&self.new_file_input, inner);
        }
    }

    async fn handle_crossterm_events(&mut self) -> Result<()> {
        let event = event::read()?;

        // Handle rename input separately to pass the raw event
        if self.show_rename {
            if let Event::Key(key) = &event
                && key.kind == KeyEventKind::Press
            {
                self.handle_rename_input(*key).await?;
            }
            return Ok(());
        }

        // Handle new file input separately
        if self.show_new_file {
            if let Event::Key(key) = &event
                && key.kind == KeyEventKind::Press
            {
                self.handle_new_file_input(*key).await?;
            }
            return Ok(());
        }

        match event {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    async fn handle_rename_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if let Some(old_name) = &self.file_to_rename {
                    let new_name = self.rename_input.lines().join("").trim().to_string();
                    if !new_name.is_empty() {
                        // Remove trailing slash if it's a directory
                        let old_name_clean = old_name.trim_end_matches('/');
                        let old_path = format!("{}/{}", self.dir.path, old_name_clean);
                        let new_path = format!("{}/{}", self.dir.path, new_name);
                        if let Err(err) = fs::rename(&old_path, &new_path) {
                            eprintln!("Failed to rename file: {err}");
                        } else {
                            self.dir.scan_and_add().await.unwrap();
                        }
                    }
                }
                self.rename_input = TextArea::default();
                self.rename_input
                    .set_block(Block::bordered().title("New name"));
                self.show_rename = false;
                self.file_to_rename = None;
            }
            KeyCode::Esc => {
                self.rename_input = TextArea::default();
                self.rename_input
                    .set_block(Block::bordered().title("New name"));
                self.show_rename = false;
                self.file_to_rename = None;
            }
            _ => {
                // Pass the event to the text area input
                self.rename_input.input(Event::Key(key));
            }
        }
        Ok(())
    }

    async fn handle_new_file_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                let new_name = self.new_file_input.lines().join("").trim().to_string();
                if !new_name.is_empty() {
                    let new_path = format!("{}/{}", self.dir.path, new_name);
                    if let Err(err) = fs::File::create(&new_path) {
                        eprintln!("Failed to create file: {err}");
                    } else {
                        self.dir.scan_and_add().await.unwrap();
                    }
                }
                self.new_file_input = TextArea::default();
                self.new_file_input
                    .set_block(Block::bordered().title("New name"));
                self.show_new_file = false;
            }
            KeyCode::Esc => {
                self.new_file_input = TextArea::default();
                self.new_file_input
                    .set_block(Block::bordered().title("New name"));
                self.show_new_file = false;
            }
            _ => {
                // Pass the event to the text area input
                self.new_file_input.input(Event::Key(key));
            }
        }
        Ok(())
    }

    async fn on_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.show_confirmation {
            match key.code {
                KeyCode::Char('y') => {
                    if let Some(file) = &self.file_to_delete {
                        let full_path = format!("{}/{}", self.dir.path, file);
                        if let Err(_e) = fs::remove_file(&full_path) {
                            fs::remove_dir_all(&full_path).unwrap_or_else(|err| {
                                eprintln!("Failed to delete directory: {err}");
                            });
                        }
                        self.dir.scan_and_add().await.unwrap();
                    }
                    self.show_confirmation = false;
                    self.file_to_delete = None;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.show_confirmation = false;
                    self.file_to_delete = None;
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                self.update_subdir_preview_async().await;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                self.update_subdir_preview_async().await;
            }
            KeyCode::Right | KeyCode::Char('l') => self.enter_directory().await?,
            KeyCode::Left | KeyCode::Char('h') => self.go_to_parent().await?,
            KeyCode::Enter => self.open_file(),
            KeyCode::Delete | KeyCode::Char('d') | KeyCode::Backspace => self.delete_file().await,
            KeyCode::Char('r') => self.rename_file(),
            KeyCode::Char('y') => self.yank_file(),
            KeyCode::Char('a') => self.new_file(),
            KeyCode::Char('c') => {
                self.handle_copy_file();
            }
            KeyCode::Char('x') => {
                self.handle_cut_file();
            }
            KeyCode::Char('p') => {
                self.handle_paste().await;
            }
            _ => {}
        }
        Ok(())
    }

    fn select_next(&mut self) {
        let items_len = self.dir.entries().len();
        if items_len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                // in case we reach the botton, go to top
                if i >= items_len - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let items_len = self.dir.entries().len();
        if items_len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                // in case we reach the top, go to the bottom
                if i == 0 { items_len - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    async fn update_subdir_preview_async(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i)
                && selected_entry.ends_with('/')
            {
                let dir_name = selected_entry.trim_end_matches('/');

                if let Some(subdir) = self.dir.subdirectories.iter().find(|d| d.name == dir_name) {
                    let mut preview_dir = Directory::new(subdir.name.clone(), subdir.path.clone());
                    // Scan asynchronously
                    if preview_dir.scan_and_add().await.is_ok() {
                        self.subdir = Some(preview_dir);
                    } else {
                        self.subdir = None;
                    }
                    return;
                }
            }
        }
        self.subdir = None;
    }

    async fn enter_directory(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                // Check if it's a directory (ends with '/')
                if selected_entry.ends_with('/') {
                    let dir_name = selected_entry.trim_end_matches('/');
                    // Find the subdirectory and navigate into it
                    if let Some(subdir) =
                        self.dir.subdirectories.iter().find(|d| d.name == dir_name)
                    {
                        let new_path = subdir.path.clone();
                        let new_name = subdir.name.clone();
                        self.dir = Directory::new(new_name, new_path);

                        // Scan the new directory
                        self.dir.scan_and_add().await.unwrap();
                        self.list_state.select(Some(0));
                        self.update_subdir_preview_async().await;
                    }
                }
            }
        }
        Ok(())
    }

    async fn go_to_parent(&mut self) -> Result<()> {
        use std::path::Path;

        let current_path = Path::new(&self.dir.path);
        if let Some(parent) = current_path.parent()
            && let Some(parent_str) = parent.to_str()
        {
            let parent_name = parent
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            self.dir = Directory::new(parent_name, parent_str.to_string());

            // Scan the parent directory
            self.dir.scan_and_add().await.unwrap();
            self.list_state.select(Some(0));
            self.update_subdir_preview_async().await;
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.exit = true;
    }
}

impl App {
    fn open_file(&mut self) {
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
        use std::process::Command;

        if let Some(file_index) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(file_index)
            // && !selected_entry.ends_with('/')
            {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);

                // leave TUI mode (temporarily)
                if let Err(e) = disable_raw_mode() {
                    eprintln!("Failed to disable raw mode: {e}");
                }

                #[cfg(target_os = "macos")]
                let mut cmd = Command::new("open");
                #[cfg(target_os = "linux")]
                let mut cmd = Command::new("xdg-open");
                #[cfg(target_os = "windows")]
                let mut cmd = Command::new("cmd");

                #[cfg(target_os = "windows")]
                {
                    cmd.args(["/C", "start", "", &full_path])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                }
                #[cfg(not(target_os = "windows"))]
                {
                    cmd.arg(&full_path)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                }

                match cmd.status() {
                    Ok(status) => {
                        if !status.success() {
                            // eprintln!("Failed to open file: {:?}", status);
                        }
                    }
                    Err(_err) => {
                        // eprintln!("Error launching file: {err}");
                    }
                }

                if let Err(e) = enable_raw_mode() {
                    eprintln!("Failed to enable raw mode: {e}");
                }
            }
        }
    }

    async fn delete_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                self.show_confirmation = true;
                self.file_to_delete = Some(selected_entry.clone());
            }
        }
    }

    fn rename_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                self.show_rename = true;
                self.file_to_rename = Some(selected_entry.to_string());

                // Pre-populate the input with the current filename
                let current_name = selected_entry.trim_end_matches('/');
                self.rename_input = TextArea::from([current_name]);
                self.rename_input
                    .set_block(Block::bordered().title("New name"));
            }
        }
    }

    fn yank_file(&mut self) {
        use std::process::Command;

        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);

                #[cfg(target_os = "macos")]
                let mut cmd = Command::new("pbcopy");
                #[cfg(target_os = "linux")]
                let mut cmd = Command::new("xclip");
                #[cfg(target_os = "windows")]
                let mut cmd = Command::new("clip");

                #[cfg(target_os = "linux")]
                {
                    cmd.args(["-selection", "clipboard"]);
                }

                let mut process = cmd
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .expect("Failed to spawn clipboard command");

                if let Some(stdin) = process.stdin.as_mut() {
                    use std::io::Write;
                    stdin
                        .write_all(full_path.as_bytes())
                        .expect("Failed to write to clipboard");
                }

                let _ = process.wait();
            }
        }
    }

    fn handle_copy_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);
                self.clipboard = Some(Clipboard {
                    cut: false,
                    path: full_path,
                });
            }
        }
    }

    fn handle_cut_file(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let entries = self.dir.entries();
            if let Some(selected_entry) = entries.get(i) {
                let full_path = format!("{}/{}", self.dir.path, selected_entry);
                self.clipboard = Some(Clipboard {
                    cut: true,
                    path: full_path,
                });
            }
        }
    }

    async fn handle_paste(&mut self) {
        if let Some(clipboard_path) = &self.clipboard.as_ref().map(|c| &c.path) {
            let filename = clipboard_path.rsplit('/').next().unwrap_or("pasted_file");
            let new_path = format!("{}/{}", self.dir.path, filename);
            if let Err(err) = fs::copy(clipboard_path, &new_path) {
                eprintln!("Failed to paste file: {err}");
            } else {
                self.dir.scan_and_add().await.unwrap();
            }
        }

        if let Some(to_cut) = &self.clipboard {
            if to_cut.cut {
                if let Err(_r) = fs::remove_file(&to_cut.path) {
                    fs::remove_dir_all(&to_cut.path).unwrap_or_else(|err| {
                        eprintln!("Failed to delete original directory after cut: {err}");
                    });
                }
                self.dir.scan_and_add().await.unwrap();
            }
            self.clipboard = None;
        }
    }

    fn new_file(&mut self) {
        self.show_new_file = true;
        self.rename_input = TextArea::default();
        self.rename_input
            .set_block(Block::bordered().title("New name"));
    }
}

struct ConfirmationDialog {
    message: String,
}

impl Widget for ConfirmationDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::style::Color;
        use ratatui::widgets::BorderType;
        
        let block = Block::bordered()
            .title(" ⚠️  Confirm Deletion ")
            .title_style(Style::new().bold().red())
            .border_type(BorderType::Rounded)
            .border_style(Style::new().red())
            .style(Style::new().bg(Color::Rgb(40, 20, 20)));
        let inner = block.inner(area);
        block.render(area, buf);

        let text = Text::from(vec![
            Line::from(""),
            Line::from(self.message.as_str())
                .centered()
                .style(Style::new().bold().white()),
            Line::from(""),
            Line::from(" Press Y to confirm, N to cancel ")
                .centered()
                .style(Style::new().fg(Color::Rgb(150, 150, 150))),
        ]);
        Paragraph::new(text).centered().render(inner, buf);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    horizontal[1]
}
