# Interactive Mode Guide

dbarena's interactive mode provides a user-friendly menu system for creating database containers without memorizing command-line flags.

## Quick Start

```bash
dbarena create -i
```

## What You'll See

### Step 1: Database Selection

```
Interactive Database Selection
──────────────────────────────────────────────────

Select databases to create (use Space to select, Enter to confirm)
  [ ] PostgreSQL
  [ ] MySQL
  [ ] SQL Server
```

Use the arrow keys to navigate, **Space** to select/deselect, and **Enter** to confirm.

You can select **multiple databases** to create them all at once.

### Step 2: Version Selection (for each database)

After selecting databases, you can choose **multiple versions** for each one:

```
PostgreSQL:
  Select versions (use Space to select, Enter to confirm)
  [ ] 16 (latest)
  [ ] 15
  [ ] 14
  [ ] 13
  [ ] 12
  [ ] 11
  [ ] Custom version
```

Use **Space** to select multiple versions, then **Enter** to confirm.

This allows you to create multiple versions of the same database in one go. For example:
- Select PostgreSQL 16, 15, and 14 to test compatibility across versions
- Select MySQL 8.0 and 5.7 to compare behavior
- Create both old and new SQL Server versions for migration testing

#### Available Versions

**PostgreSQL:**
- 16 (latest) - recommended
- 15, 14, 13, 12, 11 - older stable versions
- Custom version - enter any Docker tag

**MySQL:**
- 8.0 (latest) - recommended
- 8.4 - newer release
- 5.7, 5.6 - legacy versions
- Custom version - enter any Docker tag

**SQL Server:**
- 2022-latest - recommended
- 2019-latest
- 2017-latest
- Custom version - enter any Docker tag

### Step 3: Advanced Options (optional)

```
Configure advanced options? (y/N)
```

If you select **Yes**, you can configure:

#### Memory Limit
```
Memory limit in MB (leave empty for no limit): 512
```

Sets the maximum memory the container can use. Useful for:
- Preventing resource exhaustion
- Simulating production constraints
- Running multiple containers on limited hardware

#### CPU Shares
```
CPU shares (leave empty for no limit): 512
```

Relative CPU weight (1024 = 100% of one core). Higher values get more CPU time when under contention.

#### Persistent Volume
```
Use persistent volume? (y/N)
```

When enabled, data survives container restarts. Useful for:
- Development work you want to keep
- Testing database migrations
- Scenarios where data persistence matters

### Step 4: Confirmation

```
Selected databases:
  • postgres (16)
  • mysql (8.0)

Proceed with these selections? (Y/n)
```

Review your selections and confirm. Press Enter to proceed or type 'n' to cancel.

## Example Workflows

### Workflow 1: Quick PostgreSQL for Testing

```bash
$ dbarena create -i

# Select: PostgreSQL only
# Version: 16 (latest)
# Advanced: No
# Confirm: Yes
```

Result: PostgreSQL 16 container running in ~5 seconds

### Workflow 2: Multi-Database Development Environment

```bash
$ dbarena create -i

# Select: PostgreSQL, MySQL, SQL Server (all three)
# Versions: All latest
# Advanced: No
# Confirm: Yes
```

Result: Three database containers, all ready for development

### Workflow 3: Resource-Constrained Testing

```bash
$ dbarena create -i

# Select: PostgreSQL
# Version: 15
# Advanced: Yes
  # Memory: 256
  # CPU shares: 256
  # Persistent: No
# Confirm: Yes
```

Result: PostgreSQL 15 with limited resources, perfect for load testing

### Workflow 4: Custom Version

```bash
$ dbarena create -i

# Select: PostgreSQL
# Version: Custom version
  # Enter: 14.5-alpine
# Advanced: No
# Confirm: Yes
```

Result: Specific PostgreSQL version pulled from Docker Hub

### Workflow 5: Multi-Version Testing

```bash
$ dbarena create -i

# Select: PostgreSQL
# Versions: Select 16, 15, 14 (use Space to select multiple)
# Advanced: No
# Confirm: Yes
```

Result: Three PostgreSQL containers (versions 16, 15, and 14) created in parallel

This is perfect for:
- Testing application compatibility across database versions
- Migration testing (compare old vs new behavior)
- Feature availability testing
- Performance comparisons between versions

### Workflow 6: Cross-Database Multi-Version Matrix

```bash
$ dbarena create -i

# Select: PostgreSQL, MySQL (both)
# PostgreSQL versions: 16, 15
# MySQL versions: 8.0, 5.7
# Advanced: No
# Confirm: Yes
```

Result: Four containers created (postgres-16, postgres-15, mysql-8.0, mysql-5.7)

Perfect for testing ORMs or libraries that support multiple databases and need comprehensive compatibility testing.

## Tips

### Keyboard Shortcuts

- **Arrow keys**: Navigate options
- **Space**: Toggle selection (multi-select)
- **Enter**: Confirm selection
- **Ctrl+C**: Cancel operation

### Multi-Select Strategy

When creating multiple databases:
1. Select all databases first
2. Configure versions for each one
3. Set advanced options once (applies to all)
4. Review and confirm

### Version Selection Best Practices

- **Use "latest" tags** for development (16, 8.0, 2022-latest)
- **Use specific versions** for production simulation (15.3, 8.0.35)
- **Use custom versions** for testing specific releases or alpine variants

### When to Use Custom Versions

- Testing specific bug fixes in patch releases
- Using Alpine Linux variants for smaller image size
- Testing beta or RC (release candidate) versions
- Matching exact production database versions

## Comparison with Command-Line Mode

### Interactive Mode
```bash
dbarena create -i
# Then make selections in menu
```

**Pros:**
- No need to remember flags
- Visual feedback
- Guided workflow
- Easy multi-database setup
- Less prone to typos

**Cons:**
- Not scriptable
- Requires terminal interaction
- Slower for repeated operations

### Command-Line Mode
```bash
dbarena create postgres --version 15 --memory 512
```

**Pros:**
- Scriptable
- Fast for repeated operations
- Can use in CI/CD
- No user interaction needed

**Cons:**
- Need to remember flags
- Easy to make mistakes
- Less discoverable

## Troubleshooting

### "No databases selected" Error

**Problem:** Pressed Enter without selecting any databases

**Solution:** Use Space to select at least one database before pressing Enter

### Custom Version Not Found

**Problem:** Entered a version that doesn't exist on Docker Hub

**Solution:** Check available tags at:
- PostgreSQL: https://hub.docker.com/_/postgres/tags
- MySQL: https://hub.docker.com/_/mysql/tags
- SQL Server: https://hub.docker.com/_/microsoft-mssql-server/tags

### Menu Not Appearing

**Problem:** Terminal doesn't support interactive menus

**Solution:** Use command-line mode instead:
```bash
dbarena create postgres mysql sqlserver
```

### Advanced Options Not Showing

**Problem:** Skipped the "Configure advanced options?" prompt

**Solution:** Use command-line flags for advanced options:
```bash
dbarena create postgres -i --memory 512 --cpu-shares 512 --persistent
```

## Examples by Use Case

### Development
```bash
dbarena create -i
# Select: Your primary database
# Version: Latest
# Advanced: No
```

### Testing
```bash
dbarena create -i
# Select: All databases you support
# Versions: Match production
# Advanced: Yes (set resource limits)
```

### Learning
```bash
dbarena create -i
# Select: One database to explore
# Version: Latest
# Advanced: No (defaults are fine)
```

### CI/CD Simulation
```bash
# Use command-line mode for automation:
dbarena create postgres --version 15 --memory 256 --cpu-shares 256
```

## Next Steps

After creating containers with interactive mode:

```bash
# List your containers
dbarena list

# View connection details
dbarena inspect <container-name>

# Check logs
dbarena logs <container-name>

# Stop when done
dbarena stop <container-name>

# Destroy when finished
dbarena destroy <container-name>
```

## Feedback

Interactive mode is designed to make dbarena accessible to everyone. If you have suggestions for improving the experience, please open an issue!
