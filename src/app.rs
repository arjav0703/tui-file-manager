use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};

use crate::file_ops::Directory;

#[derive(Debug, Clone)]
pub struct App {
    pub exit: bool,
    pub dir: Directory,
}

impl Default for App {
    fn default() -> Self {
        let current_dir = Directory::new("tui-file-manager".to_string(), ".".to_string());
        Self {
            exit: false,
            dir: current_dir,
        }
    }
}

impl App {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let text = self.get_dir_structure();
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        if let (_, KeyCode::Esc | KeyCode::Char('q')) = (key.modifiers, key.code) {
            self.quit();
        }
    }

    fn quit(&mut self) {
        self.exit = true;
    }

    fn get_dir_structure(&self) -> String {
        let enrties = self.dir.entries();
        enrties.join("\n")
    }
}
