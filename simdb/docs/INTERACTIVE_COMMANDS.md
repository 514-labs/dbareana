# Interactive Commands Reference

All dbarena commands that work with containers support interactive mode via the `-i` flag. This guide shows how to use interactive mode for each command.

## Overview

Interactive mode provides:
- **Visual selection menus** - No need to remember container names
- **Status-aware filtering** - Only show relevant containers
- **Multi-select support** - Destroy multiple containers at once
- **Error prevention** - Can't select incompatible containers

## Commands

### create -i

**Select databases and versions via menu**

```bash
dbarena create -i
```

Features:
- Multi-select database types (PostgreSQL, MySQL, SQL Server)
- Version selection for each database
- Optional advanced settings
- Confirmation before creation

See [INTERACTIVE_MODE.md](INTERACTIVE_MODE.md) for detailed guide.

---

### start -i

**Select from stopped containers**

```bash
dbarena start -i
```

What it does:
- Lists only **stopped** or **exited** containers
- Single selection (one container at a time)
- Starts the selected container
- Waits for health check

Example flow:
```
Select container to start
──────────────────────────────────────────────────

Container
> my-postgres        postgres         stopped
  old-mysql          mysql            exited
  test-sqlserver     sqlserver        stopped
```

---

### stop -i

**Select from running containers**

```bash
dbarena stop -i
```

What it does:
- Lists only **running** or **healthy** containers
- Single selection
- Stops the selected container with graceful timeout

Options can be combined:
```bash
dbarena stop -i --timeout 30    # Interactive with custom timeout
```

Example flow:
```
Select container to stop
──────────────────────────────────────────────────

Container
> my-postgres        postgres         healthy
  active-mysql       mysql            running
```

---

### restart -i

**Select from running containers to restart**

```bash
dbarena restart -i
```

What it does:
- Lists only **running** containers
- Single selection
- Stops then starts the container
- Waits for health check

Example flow:
```
Select container to restart
──────────────────────────────────────────────────

Container
> my-postgres        postgres         healthy
  active-mysql       mysql            running
```

---

### destroy -i

**Multi-select containers to destroy**

```bash
dbarena destroy -i
```

What it does:
- Lists **all** containers (stopped and running)
- **Select all or multi-select** support
- Asks for confirmation (with "yes to all" option)
- Destroys selected containers

Features:
- **"Select all containers"** option for quick cleanup
- Select specific containers with Space
- **"Confirm all deletions at once?"** prompt when multiple containers selected
- Individual confirmation prompts if not using yes-to-all
- Optional volume removal with `-v`

Example flow:
```
Select containers to destroy
──────────────────────────────────────────────────

Selection mode
> Select specific containers
  Select all containers

[If "Select specific containers" chosen:]

Containers (use Space to select, Enter to confirm)
  [x] old-postgres     postgres         stopped
  [ ] my-mysql         mysql            running
  [x] test-db          postgres         exited
  [ ] prod-db          postgres         healthy

[After selection, if multiple containers:]

About to destroy 2 containers.
? Confirm all deletions at once? (y/N)

[If yes:]
✓ Confirmed: All containers will be destroyed

[If no:]
You will be asked to confirm each container individually.
Destroy container old-postgres? [y/N]
```

Combined with flags:
```bash
dbarena destroy -i -y           # Skip all confirmations
dbarena destroy -i -v           # Also remove volumes
dbarena destroy -i -y -v        # No prompts, remove volumes
```

---

### inspect -i

**Select any container to inspect**

```bash
dbarena inspect -i
```

What it does:
- Lists **all** containers
- Single selection
- Displays detailed container information

Example flow:
```
Select container to inspect
──────────────────────────────────────────────────

Container
> my-postgres        postgres         healthy
  old-mysql          mysql            stopped
  test-sqlserver     sqlserver        running
```

---

### logs -i

**Select container for log viewing**

```bash
dbarena logs -i
```

What it does:
- Lists **all** containers
- Single selection
- Streams logs from selected container

Can be combined with log options:
```bash
dbarena logs -i --follow        # Interactive with follow mode
dbarena logs -i --tail 100      # Interactive with tail limit
dbarena logs -i -f --tail 50    # Combined options
```

Example flow:
```
Select container to view logs
──────────────────────────────────────────────────

Container
> my-postgres        postgres         healthy
  old-mysql          mysql            stopped
  test-sqlserver     sqlserver        running
```

---

## Usage Patterns

### Pattern 1: Start Your Day

```bash
# See what's available
dbarena list -a

# Start what you need interactively
dbarena start -i
# Select: my-dev-postgres

dbarena start -i
# Select: my-dev-mysql
```

### Pattern 2: Check on Things

```bash
# Inspect a container
dbarena inspect -i
# Browse and select what to inspect

# Check logs
dbarena logs -i
# Select container and review logs
```

### Pattern 3: Clean Up

```bash
# Stop running containers
dbarena stop -i
# Select and stop one by one

# Or destroy multiple at once
dbarena destroy -i
# Multi-select old containers
# Space on: test-db-1, test-db-2, old-backup
# Enter to confirm
```

### Pattern 4: Quick Operations

```bash
# Quick restart during development
dbarena restart -i
# Select your dev database
# Wait for it to come back up
```

## Tips

### Keyboard Navigation

- **Up/Down arrows**: Navigate options
- **Space**: Toggle selection (multi-select only)
- **Enter**: Confirm selection
- **Ctrl+C**: Cancel operation

### Filtering Logic

Each command shows only relevant containers:

| Command | Shows | Why |
|---------|-------|-----|
| `start -i` | Stopped/Exited | Can only start stopped containers |
| `stop -i` | Running/Healthy | Can only stop running containers |
| `restart -i` | Running/Healthy | Can only restart running containers |
| `destroy -i` | All | Can destroy any container |
| `inspect -i` | All | Can inspect any container |
| `logs -i` | All | Can view logs from any container |

### When Interactive Mode Fails

If you see "No containers available":
- **start -i**: No stopped containers exist (all are running)
- **stop -i**: No running containers exist (all are stopped)
- Check with `dbarena list -a` to see all containers

### Combining Interactive with Flags

Interactive mode works with other flags:

```bash
# Destroy with auto-confirm and volume removal
dbarena destroy -i -y -v

# Stop with custom timeout
dbarena stop -i --timeout 60

# Logs with follow and tail
dbarena logs -i -f --tail 200
```

The `-i` flag always comes before or after other flags - order doesn't matter.

## Comparison: Interactive vs Command-Line

### Interactive Mode
```bash
dbarena start -i
# Then select from menu
```

**Pros:**
- Don't need to remember names
- Visual confirmation of what you're selecting
- See status before operating
- Prevents mistakes (can't start a running container)
- Great for exploration

**Cons:**
- Requires user interaction
- Not scriptable
- Slower for frequent operations

### Command-Line Mode
```bash
dbarena start my-postgres
```

**Pros:**
- Fast when you know the name
- Scriptable
- Good for automation
- Works in non-interactive shells

**Cons:**
- Need to remember/look up names
- Typos cause errors
- Can't see status before operation

### Best Practice: Use Both

- **Interactive mode**: Daily operations, exploration, cleanup
- **Command-line mode**: Scripts, automation, when you know the exact name

Example workflow:
```bash
# Morning: Interactive to start what you need
dbarena start -i

# During day: Direct commands when you know what you want
dbarena logs my-postgres --tail 50
dbarena restart my-postgres

# Evening: Interactive cleanup
dbarena destroy -i
```

## Examples by Scenario

### Scenario 1: Developer Starting Work

```bash
# See what's available
$ dbarena list -a

# Start your development database
$ dbarena start -i
Select container to start
> my-dev-postgres    postgres    stopped

# Check it's working
$ dbarena inspect -i
Select container to inspect
> my-dev-postgres    postgres    healthy
```

### Scenario 2: Testing Multiple Databases

```bash
# Create test databases interactively
$ dbarena create -i
Select databases: PostgreSQL, MySQL
Versions: 15, 8.0

# After testing, destroy them all
$ dbarena destroy -i
Select containers: [x] test-postgres, [x] test-mysql
Confirm: Yes for both
```

### Scenario 3: Troubleshooting

```bash
# Find the problematic container
$ dbarena list

# Check its logs
$ dbarena logs -i
Select container: problematic-db
# Review logs...

# Restart it
$ dbarena restart -i
Select container: problematic-db

# Check if it's healthy
$ dbarena inspect -i
Select container: problematic-db
```

### Scenario 4: Cleanup Day

```bash
# See everything
$ dbarena list -a

# Destroy old test containers
$ dbarena destroy -i
Select multiple:
[x] old-test-1
[x] old-test-2
[x] backup-20240115
[ ] production-db (keep this!)
Confirm: Yes, Yes, Yes
```

## Advanced Usage

### Scripting with Fallback

Use interactive mode as fallback:

```bash
#!/bin/bash
# Start specific container, or let user choose
if [ -n "$1" ]; then
    dbarena start "$1"
else
    dbarena start -i
fi
```

### Container Discovery

Use interactive mode to discover container names for scripts:

```bash
# Use interactive to find the name
$ dbarena inspect -i
# Note: my-postgres-20240122-xyz

# Then use in scripts
$ ./my-script.sh my-postgres-20240122-xyz
```

## Troubleshooting

### "No containers available to [action]"

**Problem:** No containers match the filter for that command

**Solutions:**
- Check all containers: `dbarena list -a`
- For `start -i`: Create or stop some containers first
- For `stop -i`: Start some containers first

### Selection Menu Not Appearing

**Problem:** Terminal doesn't support interactive menus

**Solutions:**
- Use command-line mode instead
- Check terminal compatibility (needs ANSI support)
- Try a different terminal emulator

### Wrong Container Selected

**Problem:** Accidentally selected wrong container

**Solution:** Press Ctrl+C to cancel, start over

### Can't Multi-Select

**Problem:** Space bar not working in multi-select

**Solution:**
- Ensure you're using a command that supports multi-select (only `destroy -i`)
- Try different terminal
- Use command-line mode as fallback

## Summary

Interactive mode is available for all container operations:

| Command | Mode | Filters | Multi-Select |
|---------|------|---------|--------------|
| `create -i` | Database selection | N/A | Yes |
| `start -i` | Container selection | Stopped only | No |
| `stop -i` | Container selection | Running only | No |
| `restart -i` | Container selection | Running only | No |
| `destroy -i` | Container selection | All | Yes |
| `inspect -i` | Container selection | All | No |
| `logs -i` | Container selection | All | No |

Remember: `-i` makes dbarena interactive and user-friendly, while command-line mode is for speed and automation.
