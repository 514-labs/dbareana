# dbarena v0.2.0 Release Notes

**Release Date:** 2026-01-23
**Status:** Production Release

## 🎉 Major Features

### Configuration Management System

v0.2.0 introduces a powerful configuration management system that allows you to manage database environments through declarative configuration files.

**Key Features:**
- **TOML and YAML support** - Use either format for your configs
- **Environment profiles** - Define named profiles (dev, staging, prod) with different settings
- **Precedence layers** - CLI > env-file > profile > config file > defaults
- **File discovery** - Automatic config loading from project, user, or default locations
- **Validation** - Built-in config validation with helpful error messages

**Example:**
```toml
[profiles.dev]
[profiles.dev.env]
POSTGRES_DB = "devdb"
POSTGRES_USER = "devuser"
POSTGRES_PASSWORD = "devpass"

[databases.postgres.profiles.dev]
version = "16"
port = 5432
```

```bash
# Use a profile
dbarena create postgres --config dbarena.toml --profile dev

# Override specific values
dbarena create postgres --profile dev --env POSTGRES_USER=alice
```

### Initialization Scripts

Automatically execute SQL scripts when containers are created.

**Features:**
- **Multi-script support** - Run multiple scripts in order
- **Error detection** - Database-specific error parsing with line numbers
- **Error handling** - Continue on error or stop (configurable)
- **Automatic logging** - All script output saved to `~/.local/share/dbarena/logs/`
- **Cross-database** - Works with PostgreSQL, MySQL, and SQL Server

**Example:**
```bash
# Single script
dbarena create postgres --init-script schema.sql

# Multiple scripts (executed in order)
dbarena create postgres \
  --init-script 01_schema.sql \
  --init-script 02_seed_data.sql \
  --init-script 03_indexes.sql

# Continue on errors
dbarena create postgres --init-script setup.sql --continue-on-error
```

### SQL Execution Command

Execute SQL commands on running containers without connecting manually.

**Features:**
- **Inline SQL** - Execute SQL directly from command line
- **File execution** - Run SQL files
- **Interactive selection** - Choose which container to execute on
- **Multiple queries** - Execute multiple statements at once

**Example:**
```bash
# Inline SQL
dbarena exec my-postgres "SELECT * FROM users;"

# From file
dbarena exec my-postgres --file query.sql

# Interactive mode (prompts for container)
dbarena exec "CREATE TABLE test (id INT);"
```

### Configuration Utilities

New commands to help manage configurations.

**Commands:**
- `dbarena config validate` - Validate config file syntax and semantics
- `dbarena config show` - Display resolved configuration with profiles
- `dbarena config init` - Generate example configuration file

**Example:**
```bash
# Generate starter config
dbarena config init > dbarena.toml

# Validate your config
dbarena config validate --config dbarena.toml

# See what will be applied
dbarena config show --config dbarena.toml --profile dev
```

## 📦 What's New

### New Features

1. **Configuration Files**
   - TOML and YAML parsing
   - Environment profiles with inheritance
   - Config file discovery (project > user > defaults)
   - Validation with helpful error messages

2. **Initialization Scripts**
   - Execute SQL scripts on container creation
   - Multi-script execution with ordering
   - Database-specific error parsing (PostgreSQL, MySQL, SQL Server)
   - Continue-on-error behavior
   - Automatic logging to `~/.local/share/dbarena/logs/`

3. **SQL Execution**
   - Execute inline SQL or from files
   - Interactive container selection
   - Support for all database types

4. **New CLI Commands**
   - `config validate` - Validate configuration files
   - `config show` - Display resolved configuration
   - `config init` - Generate example config
   - `exec` - Execute SQL on running containers

5. **New CLI Flags**
   - `--config <path>` - Specify config file
   - `--profile <name>` - Select environment profile
   - `--env KEY=VALUE` - Override environment variables
   - `--env-file <path>` - Load environment from file
   - `--init-script <path>` - Add initialization scripts (repeatable)
   - `--continue-on-error` - Continue if init scripts fail
   - `--keep-on-error` - Keep container if creation fails
   - `--log-dir <path>` - Custom log directory

### Improvements

1. **Error Reporting**
   - Database-specific error parsing with line numbers
   - Typo suggestions (e.g., "Did you mean INSERT?" for INSRT)
   - Error codes for MySQL and SQL Server
   - Context-aware error messages

2. **Logging**
   - Automatic logging of init script execution
   - Metadata tracking (duration, success/failure)
   - Organized by container ID
   - Easy troubleshooting with detailed logs

3. **Documentation**
   - Comprehensive configuration guide (`docs/CONFIGURATION.md`)
   - Init scripts documentation (`docs/INIT_SCRIPTS.md`)
   - Exec command documentation (`docs/EXEC_COMMAND.md`)
   - Migration guide from v0.1.0 (`docs/MIGRATION_V0.2.md`)

### Bug Fixes

- Fixed tmpfs mount preventing init script uploads (now uses `/var/dbarena_init`)
- Fixed PostgreSQL error detection (added `ON_ERROR_STOP=1`)
- Fixed continue-on-error logic with proper output parsing
- Fixed database selection for init scripts (uses `postgres` database)

## 🔧 Technical Details

### New Modules

- `src/config/` - Configuration management
  - `schema.rs` - Configuration data structures
  - `loader.rs` - File loading and discovery
  - `validator.rs` - Validation logic
  - `profile.rs` - Profile resolution
  - `merger.rs` - Configuration merging

- `src/init/` - Initialization scripts
  - `copier.rs` - File copying to containers
  - `executor.rs` - Script execution
  - `logs.rs` - Log management

### Dependencies Added

- `toml` - TOML parsing
- `serde_yaml` - YAML parsing
- `dirs` - XDG directory utilities
- `glob` - Pattern matching
- `tar` - TAR archive handling
- `uuid` - Unique ID generation

### Test Coverage

- **217+ tests total**
  - 99 unit tests (100% pass rate)
  - 118+ integration tests (functional)
- **Test categories:**
  - Configuration parsing and validation
  - Profile resolution and merging
  - Init script execution
  - Error parsing
  - SQL execution
  - Backwards compatibility

## 📚 Documentation

New documentation files:

- `docs/CONFIGURATION.md` - Complete configuration guide
- `docs/INIT_SCRIPTS.md` - Initialization scripts guide
- `docs/EXEC_COMMAND.md` - SQL execution guide
- `docs/MIGRATION_V0.2.md` - Migration guide from v0.1.0
- `MANUAL_TEST_RESULTS.md` - Manual testing results

## 🔄 Backwards Compatibility

**100% backwards compatible with v0.1.0.**

All v0.1.0 commands and flags continue to work exactly as before:

```bash
# All these still work without any config files
dbarena create postgres
dbarena create mysql --port 3307
dbarena create sqlserver --memory 2048
dbarena list
dbarena stop <container>
dbarena start <container>
dbarena destroy <container>
```

New features are opt-in - you only use them if you want to.

## 🚀 Migration Guide

### From v0.1.0 to v0.2.0

**No migration required!** v0.2.0 is fully backwards compatible.

**Optional: Start using new features**

1. **Generate a config file:**
   ```bash
   dbarena config init > dbarena.toml
   ```

2. **Define your environments:**
   ```toml
   [profiles.dev]
   [profiles.dev.env]
   POSTGRES_DB = "devdb"

   [profiles.prod]
   [profiles.prod.env]
   POSTGRES_DB = "proddb"
   ```

3. **Use profiles:**
   ```bash
   dbarena create postgres --profile dev
   ```

4. **Add init scripts to your workflow:**
   ```bash
   dbarena create postgres --profile dev --init-script schema.sql
   ```

See `docs/MIGRATION_V0.2.md` for detailed examples.

## 📥 Installation

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/yourusername/dbarena/releases/tag/v0.2.0):

- macOS (Apple Silicon): `dbarena-v0.2.0-aarch64-apple-darwin`
- macOS (Intel): `dbarena-v0.2.0-x86_64-apple-darwin`
- Linux (x86_64): `dbarena-v0.2.0-x86_64-unknown-linux-gnu`
- Linux (ARM64): `dbarena-v0.2.0-aarch64-unknown-linux-gnu`

### From Source

```bash
git clone https://github.com/yourusername/dbarena.git
cd dbarena
git checkout v0.2.0
cargo build --release
```

Binary will be at `target/release/dbarena`.

## 🔜 What's Next

Planned for v0.3.0:

- Container snapshots and restore
- Network configuration
- Volume management
- Container templates
- Bulk operations
- Performance monitoring

## 👥 Contributors

- Claude Sonnet 4.5 (Development & Testing)
- [Your name here]

## 📄 License

MIT OR Apache-2.0

## 🐛 Known Issues

None at release time.

## 💬 Feedback

Found a bug? Have a feature request?

- Open an issue: https://github.com/yourusername/dbarena/issues
- Discussions: https://github.com/yourusername/dbarena/discussions

---

**Full Changelog**: [v0.1.0...v0.2.0](https://github.com/yourusername/dbarena/compare/v0.1.0...v0.2.0)
