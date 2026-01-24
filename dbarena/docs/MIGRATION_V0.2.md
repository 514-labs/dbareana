# Migrating from v0.1.0 to v0.2.0

## What's New in v0.2.0

- Configuration file support (TOML/YAML)
- Environment variable profiles
- Initialization script execution
- Enhanced CLI with new flags
- Comprehensive error reporting

## Backwards Compatibility

**Good news**: v0.2.0 is 100% backwards compatible with v0.1.0.

All v0.1.0 commands work unchanged:

```bash
# These still work exactly as before
dbarena create postgres
dbarena create mysql --version 8.0
dbarena create postgres --name mydb --port 5433
```

No changes required to existing workflows!

## Upgrading

### Before (v0.1.0)

```bash
# Hardcoded environment variables
dbarena create postgres
# Always uses: POSTGRES_PASSWORD=postgres, POSTGRES_DB=testdb
```

### After (v0.2.0) - Option 1: Keep It Simple

```bash
# Still works exactly the same!
dbarena create postgres
```

### After (v0.2.0) - Option 2: Use Configuration

```bash
# Create config file
cat > dbarena.toml <<EOF
[databases.postgres.env]
POSTGRES_PASSWORD = "mysecret"
POSTGRES_DB = "myapp"
EOF

# Now uses custom config
dbarena create postgres
```

## Migration Path

### Step 1: Try v0.2.0 Without Changes

Install v0.2.0 and verify existing commands still work:

```bash
# Install/update dbarena
cargo install --path .

# Test existing workflow
dbarena create postgres
dbarena list
dbarena destroy <container>
```

### Step 2: Add Configuration (Optional)

If you want to customize environment variables or use profiles:

1. Create `dbarena.toml` in your project root
2. Add configuration (see examples below)
3. Test with `dbarena create postgres`

### Step 3: Add Init Scripts (Optional)

If you want to automatically set up schema/data:

1. Create SQL scripts in `./scripts/` directory
2. Reference them in config or CLI
3. Test with `dbarena create postgres`

## Common Migration Scenarios

### Scenario 1: Custom Database Name

**Before (v0.1.0):**
```bash
dbarena create postgres
# Manually: psql ... -c "CREATE DATABASE myapp;"
```

**After (v0.2.0):**
```bash
dbarena create postgres --env POSTGRES_DB=myapp
```

Or in config:
```toml
[databases.postgres.env]
POSTGRES_DB = "myapp"
```

### Scenario 2: Multiple Environments

**Before (v0.1.0):**
```bash
# Manual process for each environment
dbarena create postgres --name dev-db
# Connect and manually set up...

dbarena create postgres --name staging-db
# Connect and manually set up...
```

**After (v0.2.0):**
```toml
[profiles.dev]
env = { ENVIRONMENT = "dev" }

[profiles.staging]
env = { ENVIRONMENT = "staging" }

[databases.postgres.profiles.dev]
env = { POSTGRES_DB = "myapp_dev" }

[databases.postgres.profiles.staging]
env = { POSTGRES_DB = "myapp_staging" }
```

```bash
dbarena create postgres --profile dev --name dev-db
dbarena create postgres --profile staging --name staging-db
```

### Scenario 3: Schema Setup

**Before (v0.1.0):**
```bash
dbarena create postgres
# Wait for container...
psql -h localhost -p 5432 -U postgres -d testdb -f ./schema.sql
psql -h localhost -p 5432 -U postgres -d testdb -f ./seed.sql
```

**After (v0.2.0):**
```bash
dbarena create postgres \
    --init-script ./schema.sql \
    --init-script ./seed.sql
# Done! Schema and data are ready.
```

Or in config:
```toml
[databases.postgres]
init_scripts = ["./schema.sql", "./seed.sql"]
```

## New Features You Might Want

### 1. Configuration Files

Create `dbarena.toml`:

```toml
[databases.postgres.env]
POSTGRES_USER = "myapp"
POSTGRES_PASSWORD = "secret"
POSTGRES_DB = "myapp"
```

### 2. Profiles

Create different profiles for different environments:

```toml
[databases.postgres.profiles.dev]
env = { POSTGRES_DB = "myapp_dev" }

[databases.postgres.profiles.prod]
env = { POSTGRES_DB = "myapp_prod", POSTGRES_PASSWORD = "prod_secret" }
```

Usage:
```bash
dbarena create postgres --profile dev
dbarena create postgres --profile prod
```

### 3. Init Scripts

Automatically run SQL scripts:

```bash
dbarena create postgres --init-script ./setup.sql
```

### 4. Environment Files

Load variables from a file:

```bash
dbarena create postgres --env-file .env.local
```

## What Hasn't Changed

- Container management commands (`start`, `stop`, `destroy`, `list`)
- Interactive mode (`-i` flag)
- Docker integration
- Health checking
- Port allocation
- Resource limits (`--memory`, `--cpu-shares`)

## Troubleshooting

### Config Not Loading?

Check file locations:
```bash
# Try explicit path
dbarena create postgres --config ./dbarena.toml

# Check for typos in file name
ls -la dbarena.toml
```

### Init Scripts Not Running?

1. Verify script paths exist
2. Check logs: `~/.local/share/dbarena/logs/<container-id>/`
3. Test manually: `psql ... -f ./script.sql`

### Environment Variables Not Applied?

Check precedence:
1. CLI `--env` (highest)
2. `--env-file`
3. Profile
4. Config
5. Defaults (lowest)

## Getting Help

- [Configuration Reference](CONFIGURATION.md)
- [Init Scripts Guide](INIT_SCRIPTS.md)
- [Examples](../examples/)
- [GitHub Issues](https://github.com/yourusername/dbarena/issues)

## Summary

✅ v0.2.0 is fully backwards compatible
✅ Existing commands work unchanged
✅ New features are optional
✅ Migrate at your own pace
✅ Configuration is opt-in

Start simple, add features as needed!
