use color_eyre::Result;

mod structures;
use structures::Directory;
mod app;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App { exit: false };
    let result = app.run(terminal);
    ratatui::restore();

    let mut current_dir = Directory::new("tui-file-manager".to_string(), ".".to_string());

    current_dir.scan_and_add().await.unwrap();
    let entries = current_dir.entries();
    dbg!(entries);

    result
}
