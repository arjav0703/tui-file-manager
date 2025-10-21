use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Style, Stylize},
    text::Line,
    widgets::Block,
    widgets::{List, ListDirection, ListState},
};

use crate::file_ops::Directory;

#[derive(Debug)]
pub struct App {
    pub exit: bool,
    pub dir: Directory,
    pub list_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        let current_dir = Directory::new("tui-file-manager".to_string(), ".".to_string());
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            exit: false,
            dir: current_dir,
            list_state,
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
            .highlight_style(Style::new().italic().yellow())
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(list, frame.area(), &mut self.list_state)
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
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            _ => {}
        }
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

    fn quit(&mut self) {
        self.exit = true;
    }
}
