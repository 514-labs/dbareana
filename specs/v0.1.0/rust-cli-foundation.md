# Rust CLI Foundation

## Feature Overview

Establishes the core command-line interface architecture for simDB, providing an interactive and intuitive user experience for managing database instances. Built with Rust, the CLI offers type-safe command parsing, clear error handling, and a foundation for future TUI integration.

## Problem Statement

Database testing tools often have steep learning curves with complex command structures or require manual configuration file editing. Users need:
- An intuitive command structure that's easy to remember
- Clear, actionable error messages when things go wrong
- Fast execution with minimal startup overhead
- A consistent interface across different database types

Without a well-designed CLI foundation, adding features becomes difficult and the user experience suffers from inconsistency.

## User Stories

**As a developer**, I want to:
- Use simple, memorable commands like `simdb create postgres` instead of complex Docker invocations
- See helpful command suggestions when I make a typo or use an invalid option
- Get clear error messages that tell me exactly what went wrong and how to fix it
- Have fast command execution without noticeable startup delay

**As a new user**, I want to:
- Discover available commands through `--help` flags
- See examples for common use cases
- Understand what each command option does without reading external documentation

## Technical Requirements

### Functional Requirements

**FR-1: Command Structure**
- Top-level commands: `create`, `start`, `stop`, `restart`, `destroy`, `list`, `inspect`, `logs`
- Global options: `--verbose`, `--quiet`, `--help`, `--version`
- Subcommand-specific options with validation
- Support for both long-form (`--version`) and short-form (`-v`) flags

**FR-2: Interactive Experience**
- Rich terminal output with colors and formatting
- Progress indicators for long-running operations
- Interactive confirmations for destructive operations
- Command autocompletion support (bash, zsh, fish shells)

**FR-3: Error Handling**
- User-friendly error messages (avoid technical stack traces in normal mode)
- Suggestions for fixing common errors
- Exit codes following UNIX conventions (0 = success, non-zero = error)
- `--verbose` flag reveals detailed error information for debugging

**FR-4: Configuration Support**
- Read configuration from `~/.simdb/config.toml` (user-level defaults)
- Read configuration from `./.simdb.toml` (project-level settings)
- Environment variable support: `SIMDB_*` prefix
- Command-line options override configuration files

**FR-5: Output Formatting**
- Human-readable output by default (tables, colored text)
- JSON output mode (`--json`) for programmatic use
- Quiet mode (`--quiet`) for CI/CD environments (minimal output)

### Non-Functional Requirements

**NFR-1: Performance**
- CLI startup time <50ms on modern hardware
- Command execution overhead <10ms
- Memory footprint <20MB during idle

**NFR-2: Cross-Platform**
- Support macOS (Intel and ARM)
- Support Linux (x86_64, ARM64)
- Support Windows 10+ (with WSL2 for Docker)

**NFR-3: User Experience**
- Commands complete within 2 seconds or show progress indicator
- Error messages are concise (<5 lines) unless verbose mode is enabled
- Help text fits within standard terminal width (80 columns)

## Architecture & Design

### Components

```
main.rs
    ↓
CLI Parser (clap)
    ↓
Command Handler (match on command type)
    ↓
Business Logic (ContainerManager, ConfigManager, etc.)
    ↓
Output Formatter (stdout, stderr)
```

### Key Modules

**`main.rs`**
- Entry point, CLI initialization
- Global error handler
- Logging setup (tracing integration)

**`cli/mod.rs`**
- `CliArgs` struct: Top-level CLI structure
- Command enum: All available commands
- Clap configuration and parsing

**`cli/commands/`**
- Individual command handlers:
  - `create.rs`: Handle database creation
  - `start.rs`: Handle container start
  - `stop.rs`: Handle container stop
  - `destroy.rs`: Handle container destruction
  - `list.rs`: Handle listing containers
  - `inspect.rs`: Handle container inspection
  - `logs.rs`: Handle log streaming
- Each module implements an `execute()` function

**`cli/output.rs`**
- Output formatting utilities
- Terminal colors and styling
- Table rendering
- Progress indicators

**`config/mod.rs`**
- Configuration file loading
- Environment variable parsing
- Configuration merging (defaults → file → env → CLI args)

### Data Flow

```
User input → Clap parser → Validate arguments →
Execute command → Call business logic → Format output →
Display result (stdout/stderr) → Exit with code
```

## CLI Interface Design

### Command Syntax

```
simdb [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### Global Options

```
--verbose, -v          Increase verbosity (can be repeated: -vv, -vvv)
--quiet, -q            Suppress non-essential output
--json                 Output in JSON format (for programmatic use)
--help, -h             Show help information
--version              Show version information
--config <path>        Use custom configuration file
```

### Command Examples

```bash
# Get help on the tool
simdb --help

# Get help on a specific command
simdb create --help

# Create a database with verbose output
simdb --verbose create postgres --version 16

# List all containers in JSON format (for scripting)
simdb --json list

# Destroy with custom config file
simdb --config ./custom-config.toml destroy my-db
```

### Help Output Example

```bash
$ simdb --help
simDB - Database Simulation Environment

USAGE:
    simdb [OPTIONS] <COMMAND>

OPTIONS:
    -v, --verbose         Increase verbosity
    -q, --quiet           Suppress non-essential output
        --json            Output in JSON format
    -h, --help            Print help information
    -V, --version         Print version information
        --config <PATH>   Use custom configuration file

COMMANDS:
    create     Create and start a new database instance
    start      Start an existing database container
    stop       Stop a running database container
    restart    Restart a database container
    destroy    Destroy a database container
    list       List all simDB database containers
    inspect    Show detailed information about a container
    logs       Show logs from a database container
    help       Print this message or the help of a subcommand

Use "simdb <COMMAND> --help" for more information about a command.
```

### Error Message Examples

**Good Error (Actionable):**
```bash
$ simdb create postgres
Error: Docker daemon is not running

To fix this:
  • Start Docker Desktop
  • Or run: sudo systemctl start docker (Linux)

Run 'simdb --verbose create postgres' for more details.
```

**Bad Error (Unhelpful):**
```bash
$ simdb create postgres
Error: Connection refused (os error 111)
```

## Implementation Details

### Dependencies (Rust Crates)

```toml
[dependencies]
# CLI argument parsing
clap = { version = "4.5", features = ["derive", "cargo", "env", "unicode", "wrap_help"] }

# Terminal output and colors
console = "0.15"               # Terminal colors and formatting
indicatif = "0.17"             # Progress bars and spinners
comfy-table = "7.1"            # Table rendering

# Error handling
anyhow = "1.0"                 # Flexible error handling
thiserror = "1.0"              # Derive error types

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"             # JSON output
toml = "0.8"                   # Configuration files

# Logging
tracing = "0.1"                # Structured logging
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Async runtime
tokio = { version = "1.36", features = ["full"] }
```

### CLI Structure (Clap)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "simdb")]
#[command(about = "Database Simulation Environment", long_about = None)]
#[command(version)]
struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress non-essential output
    #[arg(short, long)]
    quiet: bool,

    /// Output in JSON format
    #[arg(long)]
    json: bool,

    /// Custom configuration file
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and start a new database instance
    Create {
        /// Database type (postgres, mysql, sqlserver)
        database: String,

        /// Database version
        #[arg(short = 'V', long)]
        version: Option<String>,

        /// Custom container name
        #[arg(short, long)]
        name: Option<String>,

        /// Host port mapping
        #[arg(short, long)]
        port: Option<u16>,

        /// Enable persistent storage
        #[arg(long)]
        persistent: bool,

        /// Environment variables (KEY=VALUE)
        #[arg(short, long = "env", value_name = "KEY=VALUE")]
        environment: Vec<String>,
    },

    /// Start an existing database container
    Start {
        /// Container name
        name: String,
    },

    /// Stop a running database container
    Stop {
        /// Container name
        name: String,

        /// Timeout in seconds
        #[arg(short, long, default_value = "10")]
        timeout: u16,
    },

    /// Destroy a database container
    Destroy {
        /// Container name
        name: String,

        /// Remove associated volume
        #[arg(long)]
        remove_volume: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// List all simDB database containers
    List {
        /// Include stopped containers
        #[arg(short, long)]
        all: bool,
    },

    /// Show detailed information about a container
    Inspect {
        /// Container name
        name: String,
    },

    /// Show logs from a database container
    Logs {
        /// Container name
        name: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show from the end
        #[arg(short, long)]
        tail: Option<usize>,
    },
}
```

### Output Formatting

```rust
use comfy_table::{Table, presets::UTF8_FULL};
use console::style;

fn print_container_list(containers: Vec<Container>) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL)
         .set_header(vec!["NAME", "TYPE", "VERSION", "STATUS", "PORTS", "UPTIME"]);

    for container in containers {
        let status_colored = match container.status.as_str() {
            "healthy" => style("healthy").green(),
            "starting" => style("starting").yellow(),
            "unhealthy" => style("unhealthy").red(),
            _ => style(&container.status).dim(),
        };

        table.add_row(vec![
            &container.name,
            &container.db_type,
            &container.version,
            &status_colored.to_string(),
            &container.ports,
            &container.uptime,
        ]);
    }

    println!("{table}");
}
```

### Progress Indicators

```rust
use indicatif::{ProgressBar, ProgressStyle};

fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

// Usage:
let spinner = create_spinner("Waiting for database to be ready...");
// ... perform operation ...
spinner.finish_with_message("PostgreSQL is ready!");
```

## Testing Strategy

### Unit Tests

- `test_cli_parsing()`: Verify command parsing works for all valid inputs
- `test_argument_validation()`: Test validation of mutually exclusive options
- `test_config_merging()`: Verify correct precedence of config sources
- `test_help_output()`: Ensure help text is properly formatted

### Integration Tests

- `test_full_workflow()`: Test complete CLI flow from parsing to execution
- `test_json_output()`: Verify JSON output format is valid and consistent
- `test_error_messages()`: Ensure error messages are user-friendly
- `test_exit_codes()`: Verify correct exit codes for success/failure scenarios

### Manual Testing Scenarios

1. **Tab Completion**: Install shell completions, verify tab completion works
2. **Interactive Prompts**: Test confirmation prompts for destructive actions
3. **Terminal Resize**: Verify output adapts to different terminal widths
4. **Color Output**: Test with terminals that support/don't support colors
5. **Piped Output**: Verify `simdb list | grep postgres` works correctly

## Documentation Requirements

- **CLI Reference**: Complete documentation of all commands and options
- **Configuration Guide**: How to use configuration files and environment variables
- **Shell Completion Setup**: Instructions for bash, zsh, and fish
- **Troubleshooting**: Common CLI issues and solutions

## Migration/Upgrade Notes

Not applicable - this is the initial release.

## Future Enhancements

- Interactive mode (REPL) for exploratory workflows
- Command aliases (e.g., `simdb rm` as alias for `destroy`)
- Plugin system for custom commands
- Shell history integration
- Batch command execution from files
- Remote execution (manage containers on remote Docker hosts)
- Built-in tutorials (`simdb tutorial cdc-testing`)
