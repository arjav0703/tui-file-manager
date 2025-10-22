use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Style, Stylize},
    text::Line,
    widgets::Block,
    widgets::{List, ListDirection, ListState},
};

use crate::file_ops::{self, Directory};

#[derive(Debug)]
pub struct App {
    pub exit: bool,
    pub dir: Directory,
    pub list_state: ListState,
}

impl App {
    pub async fn new() -> Self {
        let current_dir = file_ops::get_current_directory().await.unwrap();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            exit: false,
            dir: current_dir,
            list_state,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let _title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let items = self.dir.entries_with_symbols();
        let list = List::new(items)
            .block(Block::bordered().title(self.dir.path.as_str()))
            .style(Style::new().white())
            .highlight_style(Style::new().italic().yellow())
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(list, frame.area(), &mut self.list_state)
    }

    async fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    async fn on_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Right | KeyCode::Char('l') => self.enter_directory().await?,
            KeyCode::Left | KeyCode::Char('h') => self.go_to_parent().await?,
            KeyCode::Enter => self.open_file(),
            KeyCode::Delete | KeyCode::Char('d') | KeyCode::Backspace => self.delete_file().await,
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
        if let Some(selected_file_index) = self.list_state.selected() {
            let entries = self.dir.entries();
            let selected_entry = entries.get(selected_file_index).unwrap();
            let full_path = format!("{}/{}", self.dir.path, selected_entry);

            if std::fs::remove_file(&full_path).is_ok() {
                // Remove the file from the directory listing
                self.dir.files.retain(|f| f.name != *selected_entry);
                // Adjust the selected index if necessary
                let new_index =
                    if selected_file_index >= self.dir.entries().len() && selected_file_index > 0 {
                        selected_file_index - 1
                    } else {
                        selected_file_index
                    };
                self.list_state.select(Some(new_index));
            }

            self.dir.scan_and_add().await.unwrap();
        }
    }
}
