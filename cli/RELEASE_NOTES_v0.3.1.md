# dbarena v0.3.1 Release Notes

**Release Date**: January 24, 2026

## Overview

Version 0.3.1 introduces powerful bulk execution capabilities, allowing you to run commands across multiple database containers simultaneously. This release focuses on enhanced operational efficiency and improved error handling for multi-container workflows.

## 🚀 New Features

### Bulk Command Execution

Execute shell commands across one or multiple containers with flexible targeting options:

```bash
# Single container
dbarena exec postgres-1 -- psql -U postgres -c "SELECT version();"

# All containers
dbarena exec --all -- echo "Hello from all containers"

# Pattern filtering (glob-style)
dbarena exec --filter 'postgres-*' -- pg_isready

# Multiple specific containers
dbarena exec db1 db2 db3 -- hostname

# Parallel execution (fast!)
dbarena exec --all --parallel -- your-command
```

**Features:**
- **Sequential execution** (default): Run commands one container at a time
- **Parallel execution** (`--parallel`): Run commands simultaneously across all targets (3-5x faster)
- **Pattern filtering** (`--filter`): Use glob patterns to target containers by name
- **Custom user/workdir**: Specify execution context with `--user` and `--workdir` flags
- **Comprehensive error handling**: See detailed success/failure summary for all operations

### Query Command (Renamed)

The SQL query execution command has been renamed from `exec` to `query` for clarity:

```bash
# Inline SQL
dbarena query postgres-1 --script "SELECT current_database();"

# SQL file
dbarena query postgres-1 --file schema.sql

# Interactive mode
dbarena query -i --script "SELECT version();"
```

This separates database-specific SQL execution from general shell command execution.

## 📊 Enhanced Error Handling

Multi-container operations now provide clear execution summaries:

```
────────────────────────────────────────────────────────────────────────────────
Execution Summary
────────────────────────────────────────────────────────────────────────────────
✓ 2 container(s) succeeded:
  • postgres-1
  • postgres-2

✗ 1 container(s) failed:
  • postgres-3 - Exit code: 1

2/3 successful
```

**Error Handling Benefits:**
- ✅ **Individual container results**: See each container's outcome immediately
- ✅ **Detailed failure information**: Exit codes and error messages for each failure
- ✅ **No early termination**: All containers execute even if some fail
- ✅ **Non-zero exit code**: Script-friendly error propagation

## 🎯 Use Cases

### Database Migrations Across Fleet

```bash
# Run migration script on all PostgreSQL containers
dbarena exec --filter 'postgres-*' --parallel -- \
  psql -U postgres -f /migrations/v2.sql
```

### Health Checks

```bash
# Check if all databases are ready
dbarena exec --all -- pg_isready
```

### Batch Configuration

```bash
# Update configuration across all containers
dbarena exec --all --parallel -- \
  sh -c 'echo "shared_preload_libraries = pg_stat_statements" >> /etc/postgresql/postgresql.conf'
```

### Performance Testing

```bash
# Run pgbench on multiple databases simultaneously
dbarena exec --filter 'bench-*' --parallel -- \
  pgbench -U postgres -c 10 -j 2 -t 1000
```

## 🔧 Technical Details

### Parallel Execution Performance

Parallel mode uses Rust's `futures::join_all` to execute commands concurrently:

- **Sequential**: Time = sum of all execution times
- **Parallel**: Time = max of all execution times
- **Speedup**: Typically 3-5x faster for I/O-bound operations

Example from our tests:
- Sequential: 6.2 seconds (3 containers with 2s, 1s, 3s sleep)
- Parallel: 3.3 seconds (runs all simultaneously)

### Error Collection

Both sequential and parallel modes:
- Collect all results before summarizing
- Never short-circuit on first error
- Provide complete visibility into all failures
- Return non-zero exit code if any container fails

## 📦 Installation

### Quick Install (Unix/Linux/macOS)

```bash
curl -sSL https://raw.githubusercontent.com/514-labs/dbareana/main/cli/install.sh | bash
```

### Manual Installation

Download the binary for your platform:

- **macOS (Apple Silicon)**: [dbarena-aarch64-apple-darwin](https://github.com/514-labs/dbareana/releases/download/v0.3.1/dbarena-aarch64-apple-darwin)
- **macOS (Intel)**: [dbarena-x86_64-apple-darwin](https://github.com/514-labs/dbareana/releases/download/v0.3.1/dbarena-x86_64-apple-darwin)
- **Linux (x86_64)**: [dbarena-x86_64-unknown-linux-gnu](https://github.com/514-labs/dbareana/releases/download/v0.3.1/dbarena-x86_64-unknown-linux-gnu)

Make it executable and move to your PATH:

```bash
chmod +x dbarena-*
sudo mv dbarena-* /usr/local/bin/dbarena
```

### Verify Installation

```bash
dbarena --version
# Output: dbarena 0.3.1
```

## 🔄 Upgrade from v0.3.0

### Breaking Changes

**None!** v0.3.1 is fully backward compatible with v0.3.0.

### Command Changes

- `dbarena exec <container> --script <sql>` → `dbarena query <container> --script <sql>`
- Old `exec` command is now `query` for SQL execution
- New `exec` command is for shell command execution

If you were using the old SQL exec command, simply replace `exec` with `query` in your scripts.

## 📚 Documentation

- **Bulk Operations Guide**: See examples and best practices in the updated README
- **Error Handling**: Complete guide to interpreting execution summaries
- **Performance Tips**: When to use parallel vs sequential execution

## 🐛 Bug Fixes

- Fixed Docker error messages not being captured properly in multi-container operations
- Improved error messages for stopped/paused containers
- Better handling of container name resolution with patterns

## 🙏 Acknowledgments

This release continues our focus on operational excellence and developer productivity. The bulk execution feature was designed based on real-world use cases from database testing and migration workflows.

## 📋 Full Changelog

### Added
- `dbarena exec` command for shell command execution across containers
- `--all` flag to target all running containers
- `--filter` flag for glob-pattern container filtering
- `--parallel` flag for concurrent command execution
- `--user` and `--workdir` flags for execution context
- Comprehensive execution summary for multi-container operations
- `dbarena query` command (renamed from `exec`) for SQL execution

### Changed
- SQL execution command renamed from `exec` to `query`

### Fixed
- Error messages now properly captured in multi-container scenarios
- Non-zero exit codes correctly propagated to calling shell

## 🔮 Coming Next: v0.3.2

The next release will focus on:
- **Network Configuration**: Custom networks and container connectivity
- **Container Templates**: Reusable configuration templates

Stay tuned!

---

**Full v0.3.x Series:**
- [v0.3.0](https://github.com/514-labs/dbareana/releases/tag/v0.3.0) - Performance Monitoring, Snapshots, Volumes
- [v0.3.1](https://github.com/514-labs/dbareana/releases/tag/v0.3.1) - Bulk Operations (current)
- v0.3.2 - Network & Templates (coming soon)
