# simDB - Database Simulation Environment

A high-performance database simulation environment with Docker container management, designed for rapid testing and development.

## Status

🚧 **Under Development** - v0.1.0

## Overview

simDB provides instant database environments for:
- PostgreSQL
- MySQL
- SQL Server

## Quick Start

```bash
# Build the project
cargo build --release

# Create a PostgreSQL database
./target/release/simdb create postgres

# List running containers
./target/release/simdb list

# Show help
./target/release/simdb --help
```

## Features (v0.1.0)

- ✅ Docker container lifecycle management
- ✅ Support for PostgreSQL, MySQL, and SQL Server
- ✅ Health checking for database readiness
- ✅ CLI interface with comprehensive commands
- 🚧 Parallel container creation
- 🚧 Comprehensive benchmarks
- 🚧 Full documentation

## Requirements

- Docker Engine running locally
- Rust 1.70+ (for building from source)

## Development

```bash
# Run tests
cargo test

# Run with logging
cargo run -- -vvv create postgres

# Check code formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy -- -D warnings
```

## License

MIT OR Apache-2.0
