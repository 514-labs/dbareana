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

        /// Path to configuration file
        #[arg(long)]
        config: Option<std::path::PathBuf>,

        /// Environment profile to use
        #[arg(long)]
        profile: Option<String>,

        /// Override environment variables (KEY=VALUE)
        #[arg(long, value_name = "KEY=VALUE")]
        env: Vec<String>,

        /// Load environment variables from file
        #[arg(long)]
        env_file: Option<std::path::PathBuf>,

        /// Initialization scripts to run (can be specified multiple times)
        #[arg(long)]
        init_script: Vec<std::path::PathBuf>,

        /// Continue if initialization scripts fail
        #[arg(long)]
        continue_on_error: bool,

        /// Keep container even if init scripts fail (default: destroy on failure)
        #[arg(long)]
        keep_on_error: bool,

        /// Custom directory for saving init script logs
        #[arg(long)]
        log_dir: Option<std::path::PathBuf>,

        /// Timeout for each init script (in seconds)
        #[arg(long, default_value = "30")]
        script_timeout: u64,

        /// Validate config and scripts without creating container
        #[arg(long)]
        validate_only: bool,
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

    /// Configuration management commands
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Initialization script utilities
    #[command(subcommand)]
    Init(InitCommands),

    /// Execute SQL on a running container
    Exec {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select container
        #[arg(short, long)]
        interactive: bool,

        /// SQL script to execute
        #[arg(short, long)]
        script: Option<String>,

        /// SQL file to execute
        #[arg(short, long)]
        file: Option<std::path::PathBuf>,
    },
}

#[derive(clap::Subcommand)]
pub enum ConfigCommands {
    /// Validate configuration file
    Validate {
        /// Path to configuration file
        #[arg(long)]
        config: Option<std::path::PathBuf>,

        /// Also check that init script files exist
        #[arg(long)]
        check_scripts: bool,
    },

    /// Show loaded configuration
    Show {
        /// Path to configuration file
        #[arg(long)]
        config: Option<std::path::PathBuf>,

        /// Show resolved environment variables for a profile
        #[arg(long)]
        profile: Option<String>,
    },

    /// Initialize example configuration file
    Init,
}

#[derive(clap::Subcommand)]
pub enum InitCommands {
    /// Test a script against a running container
    Test {
        /// Path to SQL script
        script: std::path::PathBuf,

        /// Container name or ID
        #[arg(long)]
        container: String,
    },

    /// Validate script syntax (basic check)
    Validate {
        /// Path to SQL script
        script: std::path::PathBuf,

        /// Database type (postgres, mysql, sqlserver)
        #[arg(long)]
        database: String,
    },
}
