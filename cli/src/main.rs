use clap::Parser;
use dbarena::cli::commands::{config, create, destroy, exec, init_cmd, inspect, list, logs, query, snapshot, start, stats, stop, volume};
use dbarena::cli::interactive::{show_main_menu, MainMenuChoice};
use dbarena::cli::{Cli, Commands, ConfigCommands, InitCommands, SnapshotCommands, VolumeCommands};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set up Ctrl+C handler
    let result = tokio::select! {
        result = run() => result,
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\n\nInterrupted by user (Ctrl+C)");
            std::process::exit(130); // Standard exit code for SIGINT
        }
    };

    result
}

async fn run() -> anyhow::Result<()> {
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
                config: None,
                profile: None,
                env: vec![],
                env_file: None,
                init_script: vec![],
                continue_on_error: false,
                keep_on_error: false,
                log_dir: None,
                script_timeout: 30,
                validate_only: false,
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
            config,
            profile,
            env,
            env_file,
            init_script,
            continue_on_error,
            keep_on_error,
            log_dir,
            script_timeout,
            validate_only,
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
                config,
                profile,
                env,
                env_file,
                init_script,
                continue_on_error,
                keep_on_error,
                log_dir,
                script_timeout,
                validate_only,
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
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Validate {
                config: config_path,
                check_scripts,
            } => config::handle_config_validate(config_path, check_scripts).await,
            ConfigCommands::Show { config: config_path, profile } => {
                config::handle_config_show(config_path, profile).await
            }
            ConfigCommands::Init => config::handle_config_init().await,
        },
        Commands::Init(init_command) => match init_command {
            InitCommands::Test { script, container } => {
                init_cmd::handle_init_test(script, container).await
            }
            InitCommands::Validate { script, database } => {
                init_cmd::handle_init_validate(script, database).await
            }
        },
        Commands::Query {
            container,
            interactive,
            script,
            file,
        } => query::handle_query(container, interactive, script, file).await,
        Commands::Exec {
            containers,
            all,
            filter,
            user,
            workdir,
            parallel,
            command,
        } => exec::handle_exec(containers, all, filter, user, workdir, parallel, command).await,
        Commands::Stats {
            container,
            follow,
            tui,
            all,
            json,
        } => {
            use bollard::Docker;
            use std::sync::Arc;

            let docker = Docker::connect_with_local_defaults()
                .map_err(|_| dbarena::error::DBArenaError::DockerNotAvailable)?;
            let docker = Arc::new(docker);

            stats::handle_stats(docker, container, follow, tui, all, json).await
        }
        Commands::Snapshot(snapshot_cmd) => match snapshot_cmd {
            SnapshotCommands::Create { container, name, message } => {
                snapshot::handle_snapshot_create(container, name, message).await
            }
            SnapshotCommands::List { json } => {
                snapshot::handle_snapshot_list(json).await
            }
            SnapshotCommands::Restore { snapshot, name, port } => {
                snapshot::handle_snapshot_restore(snapshot, name, port).await
            }
            SnapshotCommands::Delete { snapshot, yes } => {
                snapshot::handle_snapshot_delete(snapshot, yes).await
            }
            SnapshotCommands::Inspect { snapshot, json } => {
                snapshot::handle_snapshot_inspect(snapshot, json).await
            }
        },
        Commands::Volume(volume_cmd) => match volume_cmd {
            VolumeCommands::Create { name, mount_path } => {
                volume::handle_volume_create(name, mount_path).await
            }
            VolumeCommands::List { all, json } => {
                volume::handle_volume_list(all, json).await
            }
            VolumeCommands::Delete { name, force, yes } => {
                volume::handle_volume_delete(name, force, yes).await
            }
            VolumeCommands::Inspect { name, json } => {
                volume::handle_volume_inspect(name, json).await
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
