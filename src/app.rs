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

mod commands;
mod confirmation;
mod key_handler;
mod navigation;
mod render;

use crate::file_ops::{self, Directory};
use confirmation::{ConfirmationDialog, centered_rect};

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

    fn quit(&mut self) {
        self.exit = true;
    }
}
