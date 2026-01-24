# Configuration File Reference

dbarena v0.2.0+ supports configuration files to manage database settings, environment variables, and initialization scripts.

## Table of Contents

- [File Formats](#file-formats)
- [File Discovery](#file-discovery)
- [Configuration Structure](#configuration-structure)
- [Complete Example](#complete-example)
- [Environment Variables](#environment-variables)
- [Initialization Scripts](#initialization-scripts)

## File Formats

dbarena supports two configuration formats:
- **TOML** (recommended): `.toml` extension
- **YAML**: `.yaml` or `.yml` extension

## File Discovery

Configuration files are loaded in the following order (first found wins):

1. **Explicit path**: `--config <path>`
2. **Project-local**: `./dbarena.toml` or `./dbarena.yaml`
3. **User config**: `~/.config/dbarena/config.toml` or `config.yaml`
4. **Built-in defaults**: If no config file is found

## Configuration Structure

### Top-Level Sections

```toml
[defaults]        # Global default settings
[profiles.*]      # Named environment profiles
[databases.*]     # Database-specific configuration
```

### Defaults Section

Global defaults applied to all containers:

```toml
[defaults]
persistent = false      # Keep containers after stop (default: false)
memory_mb = 512        # Memory limit in MB
cpu_shares = 1024      # CPU shares (relative weight)
```

### Profiles Section

Named environment profiles that can be applied to any database:

```toml
[profiles.dev]
env = { LOG_LEVEL = "debug", ENVIRONMENT = "development" }

[profiles.prod]
env = { LOG_LEVEL = "error", ENVIRONMENT = "production" }
```

### Databases Section

Database-specific configuration:

```toml
[databases.postgres]
default_version = "16"                    # Default version to use
init_scripts = ["./schema.sql"]          # Initialization scripts

[databases.postgres.env]                 # Base environment variables
POSTGRES_USER = "appuser"
POSTGRES_PASSWORD = "secret"
POSTGRES_DB = "myapp"

[databases.postgres.profiles.dev]       # Database-specific profile
env = { POSTGRES_DB = "myapp_dev" }
```

## Complete Example

```toml
# Global defaults
[defaults]
persistent = false
memory_mb = 512

# Global profiles
[profiles.dev]
env = { LOG_LEVEL = "debug" }

[profiles.prod]
env = { LOG_LEVEL = "error" }

# PostgreSQL configuration
[databases.postgres]
default_version = "16"
init_scripts = ["./scripts/postgres-schema.sql", "./scripts/postgres-seed.sql"]

[databases.postgres.env]
POSTGRES_USER = "appuser"
POSTGRES_PASSWORD = "dev123"
POSTGRES_DB = "myapp"

[databases.postgres.profiles.dev]
env = { POSTGRES_DB = "myapp_dev" }

[databases.postgres.profiles.prod]
env = { POSTGRES_DB = "myapp_prod", POSTGRES_PASSWORD = "prod_secret" }

# MySQL configuration
[databases.mysql]
default_version = "8.0"

[databases.mysql.env]
MYSQL_ROOT_PASSWORD = "mysql123"
MYSQL_DATABASE = "myapp"

# SQL Server configuration
[databases.sqlserver]
default_version = "2022-latest"

[databases.sqlserver.env]
SA_PASSWORD = "YourStrong@Passw0rd"
ACCEPT_EULA = "Y"
```

## Environment Variables

### Precedence Order

Environment variables are merged in the following order (later overrides earlier):

1. **Defaults**: Built-in database defaults
2. **Config base env**: `[databases.<db>.env]`
3. **Global profile**: `[profiles.<name>.env]`
4. **Database profile**: `[databases.<db>.profiles.<name>.env]`
5. **Env file**: `--env-file <path>`
6. **CLI args**: `--env KEY=VALUE` (highest priority)

### Example

```toml
# Base configuration
[databases.postgres.env]
POSTGRES_USER = "default_user"
POSTGRES_DB = "default_db"

# Profile override
[databases.postgres.profiles.dev]
env = { POSTGRES_DB = "dev_db" }
```

```bash
# CLI override (highest priority)
dbarena create postgres --profile dev --env POSTGRES_DB=my_db
# Result: POSTGRES_DB=my_db
```

### Environment File Format

Use `--env-file` to load variables from a file:

```bash
# .env.local
POSTGRES_USER=myuser
POSTGRES_PASSWORD=mysecret
POSTGRES_DB=myapp_local

# Empty lines and comments are ignored
# KEY=VALUE format only
```

Usage:
```bash
dbarena create postgres --env-file .env.local
```

## Initialization Scripts

Add SQL scripts to run automatically after container creation:

```toml
[databases.postgres]
# Simple format (array of paths)
init_scripts = ["./schema.sql", "./seed.sql"]

# Or detailed format with options
[[databases.postgres.init_scripts]]
path = "./schema.sql"
continue_on_error = false

[[databases.postgres.init_scripts]]
path = "./seed.sql"
continue_on_error = true  # Keep going even if this fails
```

### Glob Patterns

Initialization scripts support glob patterns:

```toml
[databases.postgres]
init_scripts = ["./scripts/*.sql", "./migrations/**/*.sql"]
```

### Script Execution

Scripts are executed in the order specified:
1. Container is created and started
2. Health check waits for database to be ready
3. Scripts are copied to container
4. Scripts are executed one by one
5. Output is logged to `~/.local/share/dbarena/logs/<container-id>/`

### Error Handling

By default, if a script fails, container creation stops. Override this with:

```bash
# Keep container even if scripts fail
dbarena create postgres --continue-on-error

# Or in config
[[databases.postgres.init_scripts]]
path = "./optional.sql"
continue_on_error = true
```

## Validation

Validate your configuration without creating containers:

```bash
dbarena config validate
dbarena config validate --config ./dbarena.toml
```

## Best Practices

1. **Version Control**: Commit project-local `dbarena.toml`
2. **Secrets**: Use env files or CLI args for sensitive values
3. **Profiles**: Create profiles for each environment (dev, test, prod)
4. **Init Scripts**: Keep scripts small and focused
5. **Documentation**: Comment your config files

## Examples

See the `examples/` directory for complete examples:
- `dbarena.toml` - Complete example with all features
- `dbarena-minimal.toml` - Minimal configuration
- `profiles.toml` - Profile-focused example
- `.env.example` - Environment file template
