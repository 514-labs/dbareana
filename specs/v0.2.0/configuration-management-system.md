# Configuration Management System

## Feature Overview

A comprehensive configuration management system that allows users to define database schemas, configurations, and deployment settings in declarative TOML files. The system supports database-agnostic schema definitions with automatic DDL generation for PostgreSQL, MySQL, and SQL Server.

## Problem Statement

Setting up test databases requires:
- Writing database-specific DDL scripts
- Managing different SQL dialects for the same schema across multiple databases
- Manually applying schema changes to containers
- Tracking which configurations are deployed where
- Sharing setup scripts across team members

This creates maintenance overhead and inconsistency. When testing CDC across databases, maintaining identical schemas in different SQL dialects is error-prone and time-consuming.

## User Stories

**As a CDC developer**, I want to:
- Define a database schema once and deploy it to PostgreSQL, MySQL, and SQL Server without rewriting SQL
- Version control my database configurations alongside my application code
- Quickly spin up a database with my standard test schema pre-configured
- Generate DDL for review before applying it to a database

**As a team lead**, I want to:
- Share standard database configurations with my team through Git
- Ensure all team members use consistent database setups
- Create reusable configuration templates for common testing scenarios
- Validate configurations before deployment to catch errors early

**As a QA engineer**, I want to:
- Define test database schemas without learning SQL dialects for each database
- Quickly switch between database configurations for different test scenarios
- Ensure my test environments exactly match production schemas

## Technical Requirements

### Functional Requirements

**FR-1: TOML Configuration Format**
- Configuration files use TOML syntax for readability
- Support for database connection settings, schema definitions, and deployment options
- Nested configuration for complex schemas (tables, indexes, constraints)
- Comments allowed for documentation

**FR-2: Database-Agnostic Schema Definition**
- Generic data types map to database-specific types:
  - `integer` → PostgreSQL: `INTEGER`, MySQL: `INT`, SQL Server: `INT`
  - `bigint` → PostgreSQL: `BIGINT`, MySQL: `BIGINT`, SQL Server: `BIGINT`
  - `text` → PostgreSQL: `TEXT`, MySQL: `TEXT`, SQL Server: `NVARCHAR(MAX)`
  - `timestamp` → PostgreSQL: `TIMESTAMP`, MySQL: `DATETIME`, SQL Server: `DATETIME2`
  - `boolean` → PostgreSQL: `BOOLEAN`, MySQL: `TINYINT(1)`, SQL Server: `BIT`
  - `decimal(p,s)` → All: `DECIMAL(p,s)` or `NUMERIC(p,s)`
  - `varchar(n)` → PostgreSQL: `VARCHAR(n)`, MySQL: `VARCHAR(n)`, SQL Server: `NVARCHAR(n)`
  - `json` → PostgreSQL: `JSONB`, MySQL: `JSON`, SQL Server: `NVARCHAR(MAX)` (check constraint)

**FR-3: Schema Components**
- **Tables**: Name, columns, primary key, foreign keys
- **Columns**: Name, data type, nullability, default value, auto-increment
- **Indexes**: Name, columns, unique/non-unique, type (btree, hash)
- **Constraints**: Primary key, foreign keys, unique constraints, check constraints
- **Initial Data**: Optional seed data in TOML format

**FR-4: DDL Generation**
- Generate complete DDL script for target database
- Handle database-specific syntax differences
- Include appropriate formatting and indentation
- Generate in correct dependency order (tables before foreign keys)
- Option to include `DROP TABLE IF EXISTS` statements

**FR-5: Interactive Configuration Generator**
- CLI wizard for creating configurations
- Prompts for database type, schema name, tables, columns
- Smart defaults based on common patterns
- Generates valid TOML file ready for use

**FR-6: Configuration Validation**
- Syntax validation (valid TOML)
- Semantic validation (valid data types, no circular foreign keys)
- Database-specific validation (e.g., SQL Server identifier length limits)
- Clear error messages with line numbers

**FR-7: Configuration Templates**
- Pre-built templates for common scenarios:
  - `basic-cdc`: Simple table setup for CDC testing
  - `ecommerce`: Orders, customers, products tables
  - `time-series`: Time-stamped event data
  - `audit-log`: Audit logging schema
- User can create custom templates

**FR-8: Configuration Deployment**
- Deploy configuration to a running container
- Apply DDL to initialize database
- Optionally seed initial data
- Report success/failure with details

### Non-Functional Requirements

**NFR-1: Performance**
- Configuration file parsing <50ms for typical files (<1000 lines)
- DDL generation <100ms for schemas with <100 tables
- Configuration validation <200ms

**NFR-2: Usability**
- TOML files should be human-readable and easy to edit manually
- Error messages include file name, line number, and description
- Generated DDL should be well-formatted and production-ready

**NFR-3: Reliability**
- Configuration validation prevents deployment of invalid schemas
- DDL generation never produces invalid SQL
- Rollback support if deployment fails mid-way

## Architecture & Design

### Components

```
Configuration File (TOML)
    ↓
Configuration Parser (toml crate)
    ↓
Configuration Validator
    ↓
Schema Model (database-agnostic)
    ↓
DDL Generator (database-specific)
    ↓
Deployment Engine → Docker Container → Database
```

### Key Modules

**`config/parser.rs`**
- Parse TOML configuration files
- Deserialize into Rust structs
- Handle includes/imports

**`config/validator.rs`**
- Validate configuration semantics
- Check data type validity
- Detect circular dependencies
- Database-specific validation rules

**`schema/model.rs`**
- Database-agnostic schema representation
- `Schema`, `Table`, `Column`, `Index`, `Constraint` structs

**`schema/ddl_generator.rs`**
- `DdlGenerator` trait
- Implementations: `PostgresDdlGenerator`, `MySQLDdlGenerator`, `SQLServerDdlGenerator`
- Generate CREATE TABLE, CREATE INDEX, ALTER TABLE statements

**`cli/commands/config.rs`**
- `config new`: Interactive configuration generator
- `config validate`: Validate a configuration file
- `config generate-ddl`: Generate DDL from configuration
- `config deploy`: Deploy configuration to container
- `config template`: Create from template

## Configuration File Format

### Example Configuration

```toml
# simdb configuration file
version = "1"

[database]
name = "testdb"
type = "postgres"  # postgres, mysql, sqlserver
version = "16"

[connection]
host = "localhost"
port = 5432
username = "postgres"
password = "simdb"

[[tables]]
name = "users"
comment = "User account information"

  [[tables.columns]]
  name = "id"
  type = "integer"
  nullable = false
  auto_increment = true

  [[tables.columns]]
  name = "email"
  type = "varchar(255)"
  nullable = false

  [[tables.columns]]
  name = "created_at"
  type = "timestamp"
  nullable = false
  default = "CURRENT_TIMESTAMP"

  [[tables.columns]]
  name = "is_active"
  type = "boolean"
  nullable = false
  default = "true"

  [tables.primary_key]
  columns = ["id"]

  [[tables.indexes]]
  name = "idx_users_email"
  columns = ["email"]
  unique = true

[[tables]]
name = "orders"
comment = "Customer orders"

  [[tables.columns]]
  name = "id"
  type = "bigint"
  nullable = false
  auto_increment = true

  [[tables.columns]]
  name = "user_id"
  type = "integer"
  nullable = false

  [[tables.columns]]
  name = "total_amount"
  type = "decimal(10,2)"
  nullable = false

  [[tables.columns]]
  name = "order_date"
  type = "timestamp"
  nullable = false

  [[tables.columns]]
  name = "status"
  type = "varchar(20)"
  nullable = false

  [tables.primary_key]
  columns = ["id"]

  [[tables.foreign_keys]]
  name = "fk_orders_user_id"
  columns = ["user_id"]
  referenced_table = "users"
  referenced_columns = ["id"]
  on_delete = "CASCADE"
  on_update = "CASCADE"

  [[tables.indexes]]
  name = "idx_orders_user_id"
  columns = ["user_id"]

  [[tables.indexes]]
  name = "idx_orders_status"
  columns = ["status"]

# Optional: Initial seed data
[[seed_data]]
table = "users"
data = [
  { email = "test@example.com", is_active = true },
  { email = "admin@example.com", is_active = true },
]
```

### Template: basic-cdc.toml

```toml
# Basic CDC testing configuration
version = "1"

[database]
name = "cdc_test"
comment = "Simple schema for CDC testing"

[[tables]]
name = "events"
comment = "Event stream table"

  [[tables.columns]]
  name = "id"
  type = "bigint"
  nullable = false
  auto_increment = true

  [[tables.columns]]
  name = "event_type"
  type = "varchar(50)"
  nullable = false

  [[tables.columns]]
  name = "payload"
  type = "json"
  nullable = true

  [[tables.columns]]
  name = "created_at"
  type = "timestamp"
  nullable = false
  default = "CURRENT_TIMESTAMP"

  [tables.primary_key]
  columns = ["id"]

  [[tables.indexes]]
  name = "idx_events_created_at"
  columns = ["created_at"]
```

## CLI Interface Design

### Commands

```bash
# Create new configuration interactively
simdb config new [--template <name>] [--output <path>]

# Validate a configuration file
simdb config validate <config-file>

# Generate DDL from configuration
simdb config generate-ddl <config-file> [--database <type>] [--output <path>]

# Deploy configuration to a container
simdb config deploy <config-file> --container <name>

# List available templates
simdb config templates

# Create configuration from template
simdb config from-template <template-name> [--output <path>]
```

### Example Usage

```bash
# Create new configuration interactively
$ simdb config new --output my-schema.toml
✓ Configuration type: Database Schema
✓ Database type: postgres
✓ Schema name: my_app
✓ Add table: users
  ✓ Add column: id (integer, primary key, auto-increment)
  ✓ Add column: email (varchar(255), unique)
  ✓ Add column: created_at (timestamp)
✓ Add another table? no
Configuration saved to my-schema.toml

# Validate configuration
$ simdb config validate my-schema.toml
✓ Configuration is valid
  - Database: postgres
  - Tables: 1
  - Total columns: 3

# Generate DDL
$ simdb config generate-ddl my-schema.toml --database postgres
-- Generated by simDB v0.2.0
-- Target: PostgreSQL 16

CREATE TABLE users (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_users_email ON users(email);

# Deploy to running container
$ simdb config deploy my-schema.toml --container simdb-postgres-16-a3f9
Connecting to simdb-postgres-16-a3f9...
Applying schema...
  ✓ Created table: users
  ✓ Created index: idx_users_email
Schema deployed successfully!

# Create from template
$ simdb config from-template basic-cdc --output cdc-test.toml
Created configuration from template 'basic-cdc'
File: cdc-test.toml
```

## Implementation Details

### Dependencies (Rust Crates)

```toml
[dependencies]
toml = "0.8"                   # TOML parsing
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
sqlx = { version = "0.7", features = ["postgres", "mysql", "mssql", "runtime-tokio-rustls"] }
dialoguer = "0.11"             # Interactive CLI prompts
```

### Configuration Data Structures

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    version: String,
    database: DatabaseConfig,
    #[serde(default)]
    connection: Option<ConnectionConfig>,
    tables: Vec<Table>,
    #[serde(default)]
    seed_data: Vec<SeedData>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DatabaseConfig {
    name: String,
    #[serde(rename = "type")]
    db_type: Option<String>,  // postgres, mysql, sqlserver
    version: Option<String>,
    #[serde(default)]
    comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Table {
    name: String,
    #[serde(default)]
    comment: Option<String>,
    columns: Vec<Column>,
    primary_key: Option<PrimaryKey>,
    #[serde(default)]
    foreign_keys: Vec<ForeignKey>,
    #[serde(default)]
    indexes: Vec<Index>,
    #[serde(default)]
    check_constraints: Vec<CheckConstraint>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Column {
    name: String,
    #[serde(rename = "type")]
    data_type: String,
    nullable: bool,
    #[serde(default)]
    auto_increment: bool,
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    comment: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Index {
    name: String,
    columns: Vec<String>,
    #[serde(default)]
    unique: bool,
    #[serde(default)]
    index_type: Option<String>,  // btree, hash
}

#[derive(Debug, Deserialize, Serialize)]
struct ForeignKey {
    name: String,
    columns: Vec<String>,
    referenced_table: String,
    referenced_columns: Vec<String>,
    #[serde(default)]
    on_delete: Option<String>,  // CASCADE, SET NULL, RESTRICT
    #[serde(default)]
    on_update: Option<String>,
}
```

### DDL Generator Interface

```rust
trait DdlGenerator {
    fn generate_create_table(&self, table: &Table) -> Result<String>;
    fn generate_create_index(&self, table_name: &str, index: &Index) -> Result<String>;
    fn generate_add_foreign_key(&self, table_name: &str, fk: &ForeignKey) -> Result<String>;
    fn map_data_type(&self, generic_type: &str) -> Result<String>;
    fn generate_full_schema(&self, config: &Config) -> Result<String>;
}

struct PostgresDdlGenerator;
struct MySQLDdlGenerator;
struct SQLServerDdlGenerator;

impl DdlGenerator for PostgresDdlGenerator {
    fn map_data_type(&self, generic_type: &str) -> Result<String> {
        Ok(match generic_type {
            "integer" => "INTEGER",
            "bigint" => "BIGINT",
            "text" => "TEXT",
            "timestamp" => "TIMESTAMP",
            "boolean" => "BOOLEAN",
            "json" => "JSONB",
            t if t.starts_with("varchar") => t.to_uppercase(),
            t if t.starts_with("decimal") => t.to_uppercase(),
            _ => return Err(anyhow!("Unknown data type: {}", generic_type)),
        }.to_string())
    }

    fn generate_create_table(&self, table: &Table) -> Result<String> {
        let mut sql = format!("CREATE TABLE {} (\n", table.name);

        // Columns
        for (i, column) in table.columns.iter().enumerate() {
            let data_type = self.map_data_type(&column.data_type)?;
            let nullable = if column.nullable { "" } else { " NOT NULL" };

            let auto_inc = if column.auto_increment {
                " GENERATED ALWAYS AS IDENTITY"
            } else {
                ""
            };

            let default = column.default.as_ref()
                .map(|d| format!(" DEFAULT {}", d))
                .unwrap_or_default();

            sql.push_str(&format!(
                "    {}{}{}{}{}",
                column.name,
                data_type,
                nullable,
                default,
                auto_inc
            ));

            if i < table.columns.len() - 1 || table.primary_key.is_some() {
                sql.push_str(",\n");
            }
        }

        // Primary key
        if let Some(pk) = &table.primary_key {
            sql.push_str(&format!(
                "    PRIMARY KEY ({})\n",
                pk.columns.join(", ")
            ));
        }

        sql.push_str(");\n");
        Ok(sql)
    }
}
```

## Testing Strategy

### Unit Tests

- `test_toml_parsing()`: Parse valid and invalid TOML configurations
- `test_data_type_mapping()`: Verify correct data type mappings for each database
- `test_ddl_generation_postgres()`: Generate DDL for PostgreSQL and validate syntax
- `test_ddl_generation_mysql()`: Generate DDL for MySQL and validate syntax
- `test_ddl_generation_sqlserver()`: Generate DDL for SQL Server and validate syntax
- `test_foreign_key_ordering()`: Ensure tables are created before foreign keys
- `test_validation_errors()`: Verify validator catches common errors

### Integration Tests

- `test_deploy_to_postgres()`: Deploy configuration to PostgreSQL container, verify schema
- `test_deploy_to_mysql()`: Deploy configuration to MySQL container, verify schema
- `test_deploy_to_sqlserver()`: Deploy configuration to SQL Server container, verify schema
- `test_cross_database_schema()`: Deploy same configuration to all three databases, verify equivalence
- `test_seed_data()`: Deploy with seed data, verify data is inserted

### Manual Testing Scenarios

1. **Interactive Generator**: Walk through configuration wizard, verify output
2. **Template Usage**: Create from each template, validate, deploy
3. **Complex Schema**: Define schema with 20+ tables, foreign keys, indexes
4. **Error Handling**: Introduce errors in TOML, verify helpful error messages
5. **DDL Review**: Generate DDL for all databases, manually review for correctness

## Documentation Requirements

- **Configuration Format Reference**: Complete TOML schema documentation
- **Data Type Mapping Guide**: Table showing generic types and database-specific mappings
- **Template Library**: Documentation of all available templates
- **DDL Generation Examples**: Before/after examples for each database type
- **Migration from SQL**: Guide for converting existing SQL schemas to simDB TOML format

## Migration/Upgrade Notes

Users of v0.1.0 can now:
- Define configurations for their existing databases
- Version control their database setups
- No breaking changes to v0.1.0 functionality

## Future Enhancements

- Schema diffing and migration generation
- Import from existing database (reverse engineering)
- Advanced data type support (arrays, custom types, enums)
- View and stored procedure definitions
- Schema validation rules (custom business logic constraints)
- Configuration inheritance and composition
- Support for database-specific features (partitioning, triggers)
