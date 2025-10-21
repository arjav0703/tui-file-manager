use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};

mod structures;
use structures::Directory;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut app = App { exit: false };
    let result = app.run(&mut terminal);
    ratatui::restore();

    let mut current_dir = Directory::new("tui-file-manager".to_string(), ".".to_string());

    current_dir.scan_and_add().await.unwrap();
    let entries = current_dir.entries();
    dbg!(entries);

    result
}

pub struct App {
    pub exit: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        use crossterm::event;
        use std::time::Duration;

        while !self.exit {
            let _ = terminal.draw(|frame| self.render(frame));

            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && let event::KeyCode::Char('q') = key.code
            {
                self.exit = true;
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget("hello world", frame.area());
    }
}
