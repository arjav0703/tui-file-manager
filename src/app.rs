use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Style, Stylize},
    text::Line,
    widgets::Block,
    widgets::{List, ListDirection},
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
        let _title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let items = self.dir.entries();
        let list = List::new(items)
            .block(Block::bordered().title(self.dir.path.as_str()))
            .style(Style::new().white())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_widget(list, frame.area())
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
}
