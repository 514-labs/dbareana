use clap::Parser;
use simdb::cli::{Cli, Commands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = match cli.verbose {
        0 => "simdb=info",
        1 => "simdb=debug",
        2 => "simdb=trace",
        _ => "simdb=trace,bollard=debug",
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)))
        .init();

    // Handle commands
    match cli.command {
        Commands::Create { .. } => {
            println!("Create command not yet implemented");
        }
        Commands::Start { .. } => {
            println!("Start command not yet implemented");
        }
        Commands::Stop { .. } => {
            println!("Stop command not yet implemented");
        }
        Commands::Restart { .. } => {
            println!("Restart command not yet implemented");
        }
        Commands::Destroy { .. } => {
            println!("Destroy command not yet implemented");
        }
        Commands::List { .. } => {
            println!("List command not yet implemented");
        }
        Commands::Inspect { .. } => {
            println!("Inspect command not yet implemented");
        }
        Commands::Logs { .. } => {
            println!("Logs command not yet implemented");
        }
    }

    Ok(())
}
