use color_eyre::Result;

mod file_ops;
mod app;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::default();
    app.dir.scan_and_add().await.unwrap();

    let result = app.run(terminal);
    ratatui::restore();

    result
}
