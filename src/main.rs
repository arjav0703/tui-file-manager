use color_eyre::Result;

mod app;
mod file_ops;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let mut app = App::new().await;
    app.dir.scan_and_add().await.unwrap();

    let result = app.run(terminal).await;

    ratatui::restore();

    result
}
