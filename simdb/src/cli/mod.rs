pub mod commands;
pub mod interactive;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dbarena")]
#[command(version)]
#[command(about = "Database Simulation Environment", long_about = None)]
pub struct Cli {
    /// Increase logging verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-error output
    #[arg(short, long)]
    pub quiet: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create and start a new database container
    Create {
        /// Database type(s) to create (postgres, mysql, sqlserver)
        databases: Vec<String>,

        /// Interactive mode - select databases and versions via menu
        #[arg(short, long)]
        interactive: bool,

        /// Database version
        #[arg(short, long)]
        version: Option<String>,

        /// Custom container name
        #[arg(short, long)]
        name: Option<String>,

        /// Host port to bind to
        #[arg(short, long)]
        port: Option<u16>,

        /// Use persistent volume
        #[arg(long)]
        persistent: bool,

        /// Memory limit in MB
        #[arg(long)]
        memory: Option<u64>,

        /// CPU shares (relative weight)
        #[arg(long)]
        cpu_shares: Option<u64>,
    },

    /// Start a stopped container
    Start {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select from stopped containers
        #[arg(short, long)]
        interactive: bool,
    },

    /// Stop a running container
    Stop {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select from running containers
        #[arg(short, long)]
        interactive: bool,

        /// Timeout in seconds before force kill
        #[arg(short, long, default_value = "10")]
        timeout: u64,
    },

    /// Restart a container
    Restart {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select from running containers
        #[arg(short, long)]
        interactive: bool,
    },

    /// Destroy a container
    Destroy {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select containers to destroy
        #[arg(short, long)]
        interactive: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Also remove volumes
        #[arg(short = 'v', long)]
        volumes: bool,
    },

    /// List containers
    List {
        /// Show all containers (including stopped)
        #[arg(short, long)]
        all: bool,
    },

    /// Inspect container details
    Inspect {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select container to inspect
        #[arg(short, long)]
        interactive: bool,
    },

    /// Show container logs
    Logs {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select container for logs
        #[arg(short, long)]
        interactive: bool,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(short, long)]
        tail: Option<usize>,
    },
}
