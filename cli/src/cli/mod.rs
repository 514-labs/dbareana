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

        /// Start all stopped containers
        #[arg(short, long)]
        all: bool,
    },

    /// Stop a running container
    Stop {
        /// Container name or ID
        container: Option<String>,

        /// Interactive mode - select from running containers
        #[arg(short, long)]
        interactive: bool,

        /// Stop all running containers
        #[arg(short, long)]
        all: bool,

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

        /// Destroy all containers
        #[arg(short, long)]
        all: bool,

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
    Query {
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

    /// Show container performance metrics
    Stats {
        /// Container name or ID
        container: Option<String>,

        /// Follow stats with live updates (refresh every 2s)
        #[arg(short, long)]
        follow: bool,

        /// Launch interactive TUI dashboard
        #[arg(long)]
        tui: bool,

        /// Launch enhanced multi-pane TUI with database metrics and logs
        #[arg(long)]
        multipane: bool,

        /// Show stats for all running containers
        #[arg(short, long)]
        all: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Execute a command in one or more containers
    Exec {
        /// Container name(s) or ID(s) to execute command in
        containers: Vec<String>,

        /// Execute on all running containers
        #[arg(short, long)]
        all: bool,

        /// Filter containers by name pattern (glob style: postgres-*)
        #[arg(short, long)]
        filter: Option<String>,

        /// User to run command as
        #[arg(short, long)]
        user: Option<String>,

        /// Working directory
        #[arg(short, long)]
        workdir: Option<String>,

        /// Run command in parallel across containers (default: sequential)
        #[arg(short, long)]
        parallel: bool,

        /// Command to execute (use -- to separate: dbarena exec <container> -- <command>)
        #[arg(last = true)]
        command: Vec<String>,
    },

    /// Container snapshot management
    #[command(subcommand)]
    Snapshot(SnapshotCommands),

    /// Volume management
    #[command(subcommand)]
    Volume(VolumeCommands),

    /// Network management
    #[command(subcommand)]
    Network(NetworkCommands),

    /// Container template management
    #[command(subcommand)]
    Template(TemplateCommands),
}

#[derive(clap::Subcommand)]
pub enum VolumeCommands {
    /// Create a new volume
    Create {
        /// Volume name
        name: String,

        /// Mount path (default: /data)
        #[arg(short, long)]
        mount_path: Option<String>,
    },

    /// List volumes
    List {
        /// Show all volumes (not just dbarena-managed)
        #[arg(short, long)]
        all: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Delete a volume
    Delete {
        /// Volume name
        name: String,

        /// Force deletion even if in use
        #[arg(short, long)]
        force: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Inspect volume details
    Inspect {
        /// Volume name
        name: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

#[derive(clap::Subcommand)]
pub enum SnapshotCommands {
    /// Create a snapshot from a container
    Create {
        /// Container name or ID
        container: String,

        /// Snapshot name
        #[arg(short, long)]
        name: String,

        /// Optional message describing the snapshot
        #[arg(short, long)]
        message: Option<String>,
    },

    /// List all snapshots
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Restore a snapshot to a new container
    Restore {
        /// Snapshot ID or name
        snapshot: String,

        /// Name for the restored container
        #[arg(short, long)]
        name: Option<String>,

        /// Host port to bind to
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Delete a snapshot
    Delete {
        /// Snapshot ID or name
        snapshot: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Inspect snapshot details
    Inspect {
        /// Snapshot ID or name
        snapshot: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
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

#[derive(clap::Subcommand)]
pub enum NetworkCommands {
    /// Create a new network
    Create {
        /// Network name
        name: String,

        /// Network driver (bridge, host, none, or custom)
        #[arg(short, long)]
        driver: Option<String>,

        /// Subnet in CIDR format (e.g., 172.20.0.0/16)
        #[arg(long)]
        subnet: Option<String>,

        /// Gateway address (e.g., 172.20.0.1)
        #[arg(long)]
        gateway: Option<String>,

        /// Create an internal network (no external connectivity)
        #[arg(long)]
        internal: bool,
    },

    /// List networks
    List {
        /// Show all networks (not just dbarena-managed)
        #[arg(short, long)]
        all: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Inspect network details
    Inspect {
        /// Network name
        name: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Delete a network
    Delete {
        /// Network name
        name: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Connect a container to a network
    Connect {
        /// Network name
        network: String,

        /// Container name or ID
        container: String,

        /// Network aliases for the container
        #[arg(long)]
        alias: Vec<String>,
    },

    /// Disconnect a container from a network
    Disconnect {
        /// Network name
        network: String,

        /// Container name or ID
        container: String,
    },
}

#[derive(clap::Subcommand)]
pub enum TemplateCommands {
    /// Save a container as a template
    Save {
        /// Container name or ID
        container: String,

        /// Template name
        #[arg(short, long)]
        name: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all templates
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Delete a template
    Delete {
        /// Template name
        name: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Export a template to a file
    Export {
        /// Template name
        name: String,

        /// Destination file path
        path: std::path::PathBuf,
    },

    /// Import a template from a file
    Import {
        /// Source file path
        path: std::path::PathBuf,
    },

    /// Inspect template details
    Inspect {
        /// Template name
        name: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}
