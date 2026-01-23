use clap::Parser;
use simdb::cli::commands::{create, destroy, inspect, list, logs, start, stop};
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
    let result = match cli.command {
        Commands::Create {
            databases,
            interactive,
            version,
            name,
            port,
            persistent,
            memory,
            cpu_shares,
        } => {
            create::handle_create(
                databases,
                interactive,
                version,
                name,
                port,
                persistent,
                memory,
                cpu_shares,
            )
            .await
        }
        Commands::Start { container } => start::handle_start(container).await,
        Commands::Stop { container, timeout } => stop::handle_stop(container, timeout).await,
        Commands::Restart { container } => {
            // Restart is stop + start
            stop::handle_stop(container.clone(), 10).await?;
            start::handle_start(container).await
        }
        Commands::Destroy {
            container,
            yes,
            volumes,
        } => destroy::handle_destroy(container, yes, volumes).await,
        Commands::List { all } => list::handle_list(all).await,
        Commands::Inspect { container } => inspect::handle_inspect(container).await,
        Commands::Logs {
            container,
            follow,
            tail,
        } => logs::handle_logs(container, follow, tail).await
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
