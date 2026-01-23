pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "simdb")]
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
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create and start a new database container
    Create {
        /// Database type(s) to create (postgres, mysql, sqlserver)
        databases: Vec<String>,

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
        container: String,
    },

    /// Stop a running container
    Stop {
        /// Container name or ID
        container: String,

        /// Timeout in seconds before force kill
        #[arg(short, long, default_value = "10")]
        timeout: u64,
    },

    /// Restart a container
    Restart {
        /// Container name or ID
        container: String,
    },

    /// Destroy a container
    Destroy {
        /// Container name or ID
        container: String,

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
        container: String,
    },

    /// Show container logs
    Logs {
        /// Container name or ID
        container: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(short, long)]
        tail: Option<usize>,
    },
}
