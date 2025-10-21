use color_eyre::Result;

mod file_ops;
use file_ops::Directory;
mod app;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::default();
    app.dir.scan_and_add().await.unwrap();

    let _app = app.clone();
    let result = app.run(terminal);
    ratatui::restore();

    let mut current_dir = Directory::new("tui-file-manager".to_string(), ".".to_string());

    current_dir.scan_and_add().await.unwrap();
    let _entries = current_dir.entries();
    dbg!(_app);

    result
}
