use clap::Parser;
use dbarena::cli::commands::{create, destroy, inspect, list, logs, start, stop};
use dbarena::cli::interactive::{show_main_menu, MainMenuChoice};
use dbarena::cli::{Cli, Commands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = match cli.verbose {
        0 => "dbarena=warn",
        1 => "dbarena=info",
        2 => "dbarena=debug",
        3 => "dbarena=trace",
        _ => "dbarena=trace,bollard=debug",
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)))
        .init();

    // If no command specified, show main menu
    let command = if let Some(cmd) = cli.command {
        cmd
    } else {
        // Show interactive main menu
        match show_main_menu()? {
            MainMenuChoice::Create => Commands::Create {
                databases: vec![],
                interactive: true,
                version: None,
                name: None,
                port: None,
                persistent: false,
                memory: None,
                cpu_shares: None,
            },
            MainMenuChoice::List => Commands::List { all: false },
            MainMenuChoice::Start => Commands::Start {
                container: None,
                interactive: true,
            },
            MainMenuChoice::Stop => Commands::Stop {
                container: None,
                interactive: true,
                timeout: 10,
            },
            MainMenuChoice::Restart => Commands::Restart {
                container: None,
                interactive: true,
            },
            MainMenuChoice::Destroy => Commands::Destroy {
                container: None,
                interactive: true,
                yes: false,
                volumes: false,
            },
            MainMenuChoice::Inspect => Commands::Inspect {
                container: None,
                interactive: true,
            },
            MainMenuChoice::Logs => Commands::Logs {
                container: None,
                interactive: true,
                follow: false,
                tail: None,
            },
            MainMenuChoice::Exit => {
                println!("\n{}", console::style("Goodbye! 👋").cyan());
                return Ok(());
            }
        }
    };

    // Handle commands
    let result = match command {
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
        } => logs::handle_logs(container, interactive, follow, tail).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
