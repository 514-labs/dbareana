use clap::Parser;
use dbarena::cli::commands::{config, create, destroy, docs, exec, init_cmd, inspect, list, logs, network, query, seed, snapshot, start, stats, stop, template, volume, workload};
use dbarena::cli::interactive::{show_main_menu, MainMenuChoice};
use dbarena::cli::{Cli, Commands, ConfigCommands, DocsCommands, InitCommands, NetworkCommands, SnapshotCommands, TemplateCommands, VolumeCommands};
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
                all: false,
            },
            MainMenuChoice::Stop => Commands::Stop {
                container: None,
                interactive: true,
                all: false,
                timeout: 10,
            },
            MainMenuChoice::Restart => Commands::Restart {
                container: None,
                interactive: true,
            },
            MainMenuChoice::Destroy => Commands::Destroy {
                container: None,
                interactive: true,
                all: false,
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
            all,
        } => start::handle_start(container, interactive, all).await,
        Commands::Stop {
            container,
            interactive,
            all,
            timeout,
        } => stop::handle_stop(container, interactive, all, timeout).await,
        Commands::Restart {
            container,
            interactive,
        } => {
            // Restart is stop + start
            stop::handle_stop(container.clone(), interactive, false, 10).await?;
            start::handle_start(container, interactive, false).await
        }
        Commands::Destroy {
            container,
            interactive,
            all,
            yes,
            volumes,
        } => destroy::handle_destroy(container, interactive, all, yes, volumes).await,
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
            container_flag,
            interactive,
            script,
            file,
        } => {
            let resolved_container = container_flag.or(container);
            query::handle_query(resolved_container, interactive, script, file).await
        }
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
            multipane,
            all,
            json,
        } => {
            use bollard::Docker;
            use std::sync::Arc;

            let docker = Docker::connect_with_local_defaults()
                .map_err(|_| dbarena::error::DBArenaError::DockerNotAvailable)?;
            let docker = Arc::new(docker);

            stats::handle_stats(docker, container, follow, tui, multipane, all, json).await
        }
        Commands::Snapshot(snapshot_cmd) => match snapshot_cmd {
            SnapshotCommands::Create { container, container_flag, name, message } => {
                let resolved_container = container_flag
                    .or(container)
                    .ok_or_else(|| anyhow::anyhow!("Container name or ID is required"))?;
                snapshot::handle_snapshot_create(resolved_container, name, message).await
            }
            SnapshotCommands::List { json } => {
                snapshot::handle_snapshot_list(json).await
            }
            SnapshotCommands::Restore { snapshot, snapshot_flag, name, port } => {
                let resolved_snapshot = snapshot_flag
                    .or(snapshot)
                    .ok_or_else(|| anyhow::anyhow!("Snapshot ID or name is required"))?;
                snapshot::handle_snapshot_restore(resolved_snapshot, name, port).await
            }
            SnapshotCommands::Delete { snapshot, snapshot_flag, yes } => {
                let resolved_snapshot = snapshot_flag
                    .or(snapshot)
                    .ok_or_else(|| anyhow::anyhow!("Snapshot ID or name is required"))?;
                snapshot::handle_snapshot_delete(resolved_snapshot, yes).await
            }
            SnapshotCommands::Inspect { snapshot, snapshot_flag, json } => {
                let resolved_snapshot = snapshot_flag
                    .or(snapshot)
                    .ok_or_else(|| anyhow::anyhow!("Snapshot ID or name is required"))?;
                snapshot::handle_snapshot_inspect(resolved_snapshot, json).await
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
        Commands::Network(network_cmd) => match network_cmd {
            NetworkCommands::Create { name, driver, subnet, gateway, internal } => {
                network::handle_network_create(name, driver, subnet, gateway, internal).await
            }
            NetworkCommands::List { all, json } => {
                network::handle_network_list(all, json).await
            }
            NetworkCommands::Inspect { name, json } => {
                network::handle_network_inspect(name, json).await
            }
            NetworkCommands::Delete { name, yes } => {
                network::handle_network_delete(name, yes).await
            }
            NetworkCommands::Connect { network: net, container, alias } => {
                network::handle_network_connect(net, container, alias).await
            }
            NetworkCommands::Disconnect { network: net, container } => {
                network::handle_network_disconnect(net, container).await
            }
        },
        Commands::Seed {
            container,
            config,
            size,
            seed: seed_value,
            truncate,
            incremental,
            rows,
        } => {
            seed::handle_seed(
                config,
                container,
                size,
                seed_value,
                truncate,
                incremental,
                rows,
            )
            .await
        }
        Commands::Workload {
            container,
            pattern,
            config,
            connections,
            tps,
            duration,
            transactions,
        } => {
            workload::handle_workload_run(
                container,
                pattern,
                config,
                connections,
                tps,
                duration,
                transactions,
            )
            .await
        }
        Commands::Docs(docs_cmd) => match docs_cmd {
            DocsCommands::List { installed, available, json } => {
                docs::handle_docs_list(installed, available, json).await
            }
            DocsCommands::Install {
                db,
                version,
                force,
                keep_source,
                accept_license,
            } => {
                docs::handle_docs_install(db, version, force, keep_source, accept_license).await
            }
            DocsCommands::Search {
                db,
                version,
                query,
                limit,
                json,
            } => docs::handle_docs_search(db, version, query, limit, json).await,
            DocsCommands::Show {
                doc_id,
                max_chars,
                json,
            } => docs::handle_docs_show(doc_id, max_chars, json).await,
            DocsCommands::Remove { db, version, yes } => {
                docs::handle_docs_remove(db, version, yes).await
            }
        },
        Commands::Template(template_cmd) => match template_cmd {
            TemplateCommands::Save { container, name, description } => {
                template::handle_template_save(container, name, description).await
            }
            TemplateCommands::List { json } => {
                template::handle_template_list(json).await
            }
            TemplateCommands::Delete { name, yes } => {
                template::handle_template_delete(name, yes).await
            }
            TemplateCommands::Export { name, path } => {
                template::handle_template_export(name, path).await
            }
            TemplateCommands::Import { path } => {
                template::handle_template_import(path).await
            }
            TemplateCommands::Inspect { name, json } => {
                template::handle_template_inspect(name, json).await
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
