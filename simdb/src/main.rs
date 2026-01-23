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
        Commands::Start {
            container,
            interactive,
        } => start::handle_start(container, interactive).await,
        Commands::Stop {
            container,
            interactive,
            timeout,
        } => stop::handle_stop(container, interactive, timeout).await,
        Commands::Restart {
            container,
            interactive,
        } => {
            // Restart is stop + start
            stop::handle_stop(container.clone(), interactive, 10).await?;
            start::handle_start(container, interactive).await
        }
        Commands::Destroy {
            container,
            interactive,
            yes,
            volumes,
        } => destroy::handle_destroy(container, interactive, yes, volumes).await,
        Commands::List { all } => list::handle_list(all).await,
        Commands::Inspect {
            container,
            interactive,
        } => inspect::handle_inspect(container, interactive).await,
        Commands::Logs {
            container,
            interactive,
            follow,
            tail,
        } => logs::handle_logs(container, interactive, follow, tail).await
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
