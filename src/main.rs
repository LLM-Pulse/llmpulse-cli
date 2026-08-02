use clap::Parser;
use llmpulse_cli::{cli::Cli, commands, error::format_error};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(error) = commands::run(cli).await {
        eprintln!("{}", format_error(&error));
        std::process::exit(1);
    }
}
