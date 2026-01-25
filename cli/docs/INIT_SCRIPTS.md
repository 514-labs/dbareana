# Initialization Scripts Guide

Automatically execute SQL scripts when creating database containers.

## Quick Start

### CLI Method

```bash
dbarena create postgres --init-script ./schema.sql --init-script ./seed.sql
```

### Config File Method

```toml
[databases.postgres]
init_scripts = ["./schema.sql", "./seed.sql"]
```

## How It Works

1. Container is created and started
2. Health check waits for database to be ready
3. Scripts are copied to `/tmp/dbarena_init/` in container
4. Scripts execute in order using database CLI
5. Output is logged to `~/.local/share/dbarena/logs/<container-id>/`

## Database-Specific Execution

### PostgreSQL
```bash
psql -U $POSTGRES_USER -d $POSTGRES_DB -f /tmp/dbarena_init/script.sql
```

### MySQL
```bash
mysql -u root -p$MYSQL_ROOT_PASSWORD $MYSQL_DATABASE < /tmp/dbarena_init/script.sql
```

### SQL Server
```bash
/opt/mssql-tools/bin/sqlcmd -S localhost -U sa -P $SA_PASSWORD -i /tmp/dbarena_init/script.sql
```

## Script Examples

### PostgreSQL Schema

```sql
-- schema.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_username ON users(username);
```

### Seed Data

```sql
-- seed.sql
INSERT INTO users (username) VALUES
    ('admin'),
    ('user1'),
    ('user2');
```

## Error Handling

### Default Behavior

By default, if a script fails:
- Container creation stops
- Error details are displayed
- Container is destroyed (unless `--keep-on-error` is used)

### Continue on Error

```bash
# Keep going even if scripts fail
dbarena create postgres \
    --init-script ./schema.sql \
    --init-script ./optional.sql \
    --continue-on-error
```

Or in config:

```toml
[[databases.postgres.init_scripts]]
path = "./schema.sql"
continue_on_error = false

[[databases.postgres.init_scripts]]
path = "./optional.sql"
continue_on_error = true
```

## Logs

Script output is saved to:

```
~/.local/share/dbarena/logs/<container-id>/
├── schema.sql.log
├── seed.sql.log
└── metadata.json
```

View logs:

```bash
cat ~/.local/share/dbarena/logs/postgres-16-abc123/schema.sql.log
```

## Glob Patterns

Use glob patterns to match multiple files:

```toml
[databases.postgres]
init_scripts = [
    "./migrations/*.sql",
    "./seeds/**/*.sql"
]
```

Files are executed in alphabetical order within each pattern.

## Debugging Failed Scripts

### 1. Check the Error Message

Errors show:
- Script path
- Line number (if parseable)
- Error message from database
- Suggestions for common typos

### 2. Review the Log

```bash
cat ~/.local/share/dbarena/logs/<container-id>/<script>.log
```

### 3. Test Manually

Connect to the container and test the script:

```bash
# PostgreSQL
psql -h localhost -p 54321 -U postgres -d testdb -f ./script.sql

# MySQL
mysql -h localhost -P 33061 -u root -pmysql testdb < ./script.sql
```

### 4. Common Issues

**Syntax Errors:**
```
ERROR: syntax error at or near "INSRT"
LINE 1: INSRT INTO users VALUES (1);
```
→ Fix typo: `INSRT` → `INSERT`

**Missing Tables:**
```
ERROR: relation "users" does not exist
```
→ Ensure schema script runs before seed script

**Permission Errors:**
```
ERROR: permission denied for table users
```
→ Check user has necessary permissions

## Best Practices

1. **Order Matters**: Schema first, then seed data
2. **Idempotent Scripts**: Use `IF NOT EXISTS` / `ON CONFLICT DO NOTHING`
3. **Small Scripts**: Break large scripts into smaller, focused files
4. **Test Locally**: Test scripts before adding to config
5. **Error Handling**: Decide if each script is critical or optional
6. **Transactions**: Wrap related statements in transactions

## Example: Complete Setup

```toml
# dbarena.toml
[databases.postgres.env]
POSTGRES_DB = "myapp"
POSTGRES_USER = "appuser"
POSTGRES_PASSWORD = "dev123"

[databases.postgres]
init_scripts = [
    "./db/schema.sql",
    "./db/seed.sql",
    "./db/indexes.sql"
]
```

```bash
# Create container with init scripts
dbarena create postgres

# Container is ready with schema and data!
psql -h localhost -p 54321 -U appuser -d myapp
```

## Related

- [Configuration Reference](CONFIGURATION.md)
- [Examples](../examples/)
