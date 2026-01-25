# Execute SQL Command Guide

Execute SQL scripts directly against running containers without needing to create files.

## Quick Start

### Inline SQL

Execute SQL directly from the command line:

```bash
dbarena exec --script "SELECT * FROM users;"
```

### From File

Execute SQL from a file:

```bash
dbarena exec --file ./query.sql
```

### Interactive Mode

Select container interactively:

```bash
dbarena exec -i --script "SELECT * FROM users;"
```

## Usage

### Basic Syntax

```bash
dbarena exec [OPTIONS]
```

### Options

- `--container <name>` - Container name or ID (optional, prompts if not specified)
- `-i, --interactive` - Select container interactively
- `--script <sql>` - Inline SQL script to execute
- `--file <path>` - Path to SQL file to execute

### Requirements

- Exactly one of `--script` or `--file` must be provided
- Container must be running
- Container must be managed by dbarena

## Examples

### Example 1: Quick Query

```bash
# Count rows in a table
dbarena exec --container postgres-16-abc123 --script "SELECT COUNT(*) FROM users;"
```

### Example 2: Interactive Selection

```bash
# Select container from list
dbarena exec -i --script "SELECT * FROM users LIMIT 10;"
```

### Example 3: Run SQL File

```bash
# Execute a complex query from file
dbarena exec --file ./reports/monthly_summary.sql
```

### Example 4: Multiple Statements

```bash
# Execute multiple statements
dbarena exec --script "
CREATE TABLE temp_results (
    id INT PRIMARY KEY,
    value TEXT
);

INSERT INTO temp_results VALUES (1, 'test');

SELECT * FROM temp_results;
"
```

### Example 5: Data Manipulation

```bash
# Update records
dbarena exec --container mydb --script "
UPDATE users
SET status = 'active'
WHERE last_login > NOW() - INTERVAL '30 days';
"
```

## How It Works

1. **Container Selection**: If not specified, you can select from running containers
2. **Script Preparation**:
   - Inline scripts are saved to a temporary file
   - File scripts are used directly
3. **Execution**: Script is copied to container and executed using the database's CLI
4. **Output**: Results are displayed with syntax highlighting
5. **Logging**: Output is saved to `~/.local/share/dbarena/logs/<container-id>/`

## Database-Specific Notes

### PostgreSQL

Uses `psql` to execute scripts:
```bash
psql -U $POSTGRES_USER -d $POSTGRES_DB -f script.sql
```

**Tips:**
- Use `\timing` to show query execution time
- Use `\x` for expanded display mode
- Transaction blocks work normally

### MySQL

Uses `mysql` to execute scripts:
```bash
mysql -u root -p$MYSQL_ROOT_PASSWORD $MYSQL_DATABASE < script.sql
```

**Tips:**
- Use `DELIMITER` for stored procedures
- `SOURCE` command is not needed
- Comments use `--` or `/* */`

### SQL Server

Uses `sqlcmd` to execute scripts:
```bash
/opt/mssql-tools/bin/sqlcmd -S localhost -U sa -P $SA_PASSWORD -i script.sql
```

**Tips:**
- Use `GO` to separate batches
- Variable substitution not supported
- SQLCMD mode directives not supported

## Error Handling

### Script Errors

If a script fails, detailed error information is displayed:

```
✗ Script execution failed
  Duration: 0.15s

Error Details:
────────────────────────────────────────────────────────────
ERROR:  syntax error at or near "SLECT"
LINE 1: SLECT * FROM users;
        ^

Suggestion: Did you mean 'SELECT'?
────────────────────────────────────────────────────────────
```

### Container Not Running

```
Error: Container not found: postgres-16-abc123

Available containers:
  dbarena list
```

### Connection Issues

```
Error: Failed to connect to container

Troubleshooting:
  1. Verify container is running: dbarena list
  2. Check container logs: dbarena logs <container>
  3. Verify database is healthy: dbarena inspect <container>
```

## Best Practices

1. **Test First**: Test complex scripts on a dev container first
2. **Use Transactions**: Wrap DML in transactions for safety
3. **Keep Scripts Small**: Break large scripts into smaller chunks
4. **Save Important Queries**: Use `--file` for queries you'll reuse
5. **Check Results**: Always verify output before running on production

## Comparison with Init Scripts

| Feature | Exec Command | Init Scripts |
|---------|-------------|--------------|
| **When** | After container creation | During container creation |
| **Target** | Running containers | New containers |
| **Purpose** | Ad-hoc queries | Setup/initialization |
| **Logging** | Optional (saved to logs) | Always logged |
| **Idempotency** | User's responsibility | Recommended in scripts |

## Advanced Usage

### Pipe Output

Redirect output to a file:

```bash
dbarena exec --script "SELECT * FROM users;" > users.txt
```

### Combine with Other Tools

```bash
# Export data and process with jq
dbarena exec --script "SELECT json_agg(users) FROM users;" | jq '.'
```

### Script Templates

Create reusable script templates:

```bash
# user_report.sql
SELECT
    COUNT(*) as total_users,
    COUNT(DISTINCT email) as unique_emails,
    MAX(created_at) as last_signup
FROM users
WHERE created_at > CURRENT_DATE - INTERVAL '7 days';
```

```bash
dbarena exec --file ./templates/user_report.sql
```

## Troubleshooting

### Permission Denied

```bash
# Ensure you have permission to read the script file
chmod +r ./script.sql
dbarena exec --file ./script.sql
```

### Large Output

```bash
# Limit output rows
dbarena exec --script "SELECT * FROM large_table LIMIT 100;"
```

### Timeout Issues

For long-running queries, check container logs for progress:

```bash
# Terminal 1
dbarena exec --file ./long_query.sql

# Terminal 2 (monitor logs)
dbarena logs <container> --follow
```

## Related Commands

- `dbarena create --init-script` - Run scripts during creation
- `dbarena init test` - Test script against container
- `dbarena logs` - View container logs
- `dbarena inspect` - View container details

## See Also

- [Configuration Guide](CONFIGURATION.md)
- [Init Scripts Guide](INIT_SCRIPTS.md)
- [Troubleshooting](TROUBLESHOOTING.md)
