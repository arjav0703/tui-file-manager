use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long)]
    pub show_hidden_files: bool,
}

pub fn load_config() -> Cli {
    Cli::parse()
}
