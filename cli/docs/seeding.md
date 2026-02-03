# Data Seeding Guide

This guide covers how to use dbarena's data seeding capabilities to populate databases with realistic test data.

## Table of Contents
- [Quick Start](#quick-start)
- [Configuration Format](#configuration-format)
- [Data Generators](#data-generators)
- [Foreign Key Relationships](#foreign-key-relationships)
- [Size Presets](#size-presets)
- [Advanced Features](#advanced-features)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Quick Start

### Basic Workflow

```bash
# 1. Create a database container
dbarena create postgres --name mydb

# 2. Create a seed configuration file
cat > seed-config.toml << 'EOF'
global_seed = 42
batch_size = 1000

[[seed_rules.tables]]
name = "users"
count = 1000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

[[seed_rules.tables.columns]]
name = "email"
generator = "email"

[[seed_rules.tables.columns]]
name = "name"
generator = "name"
[seed_rules.tables.columns.options]
type = "full"
EOF

# 3. Seed the database
dbarena seed --config seed-config.toml --container mydb

# 4. Verify the data
dbarena exec mydb -- psql -U postgres -c "SELECT COUNT(*) FROM users"
```

## Configuration Format

### Top-Level Structure

```toml
# Optional: Deterministic seed for reproducibility
global_seed = 42

# Optional: Batch size for inserts (default: 1000)
batch_size = 1000

# Table seeding rules
[[seed_rules.tables]]
name = "table_name"
count = 1000  # Number of rows to generate

[[seed_rules.tables.columns]]
name = "column_name"
generator = "generator_type"
[seed_rules.tables.columns.options]
# Generator-specific options
```

### Minimal Example

```toml
[[seed_rules.tables]]
name = "users"
count = 100

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"

[[seed_rules.tables.columns]]
name = "email"
generator = "email"
```

## Data Generators

### Sequential Generator
Generates incrementing integers.

```toml
[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1  # Starting value (default: 1)
```

### Random Integer Generator
Generates random integers in a range.

```toml
[[seed_rules.tables.columns]]
name = "age"
generator = "random_int"
[seed_rules.tables.columns.options]
min = 18
max = 99
```

### Random Decimal Generator
Generates random decimal numbers.

```toml
[[seed_rules.tables.columns]]
name = "price"
generator = "random_decimal"
[seed_rules.tables.columns.options]
min = 9.99
max = 999.99
precision = 2  # Decimal places
```

### Boolean Generator
Generates random boolean values.

```toml
[[seed_rules.tables.columns]]
name = "is_active"
generator = "boolean"
[seed_rules.tables.columns.options]
probability = 0.8  # 80% true, 20% false (default: 0.5)
```

### Timestamp Generator
Generates timestamps in various modes.

**Current timestamp:**
```toml
[[seed_rules.tables.columns]]
name = "created_at"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "now"
```

**Range of timestamps:**
```toml
[[seed_rules.tables.columns]]
name = "created_at"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "range"
start = "2024-01-01"
end = "2026-01-01"
```

**Relative to now:**
```toml
[[seed_rules.tables.columns]]
name = "created_at"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "relative"
offset_seconds = -86400  # 1 day ago
```

### Email Generator
Generates realistic email addresses.

```toml
[[seed_rules.tables.columns]]
name = "email"
generator = "email"
# No options required
```

Examples: `john.doe@example.com`, `jane.smith@gmail.com`

### Phone Generator
Generates phone numbers.

```toml
[[seed_rules.tables.columns]]
name = "phone"
generator = "phone"
# No options required
```

Examples: `555-0123`, `(555) 555-0199`

### Name Generator
Generates realistic names.

```toml
[[seed_rules.tables.columns]]
name = "full_name"
generator = "name"
[seed_rules.tables.columns.options]
type = "full"  # Options: "first", "last", "full"
```

### Address Generator
Generates street addresses.

```toml
[[seed_rules.tables.columns]]
name = "address"
generator = "address"
# No options required
```

Examples: `123 Main St`, `456 Oak Avenue`

### Template Generator
Generates values from a template string.

```toml
[[seed_rules.tables.columns]]
name = "product_name"
generator = "template"
[seed_rules.tables.columns.options]
template = "Product {random_int:1:999}"
```

Template syntax:
- `{random_int:min:max}` - Random integer
- `{sequential}` - Sequential number

Examples: `Product 42`, `Product 789`

### Enum Generator
Generates values from a predefined list.

```toml
[[seed_rules.tables.columns]]
name = "status"
generator = "enum"
[seed_rules.tables.columns.options]
values = ["pending", "processing", "shipped", "delivered"]
```

### Foreign Key Generator
Generates values referencing another table's column.

```toml
[[seed_rules.tables.columns]]
name = "user_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "users", column = "id" }
```

**Important:** The referenced table must be seeded first. dbarena automatically determines the correct order.

## Foreign Key Relationships

### Automatic Dependency Resolution

dbarena automatically resolves table dependencies based on foreign key references and seeds tables in the correct order.

**Example:**
```toml
# Users table (no dependencies)
[[seed_rules.tables]]
name = "users"
count = 1000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"

# Orders table (depends on users)
[[seed_rules.tables]]
name = "orders"
count = 5000

[[seed_rules.tables.columns]]
name = "user_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "users", column = "id" }
```

dbarena seeds `users` first, then `orders`, ensuring all `user_id` values are valid.

### Complex Dependencies

For complex schemas with multiple levels of dependencies:

```toml
# Level 1: Independent table
[[seed_rules.tables]]
name = "users"
count = 1000

# Level 2: Depends on users
[[seed_rules.tables]]
name = "orders"
count = 5000

[[seed_rules.tables.columns]]
name = "user_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "users", column = "id" }

# Level 3: Depends on orders
[[seed_rules.tables]]
name = "order_items"
count = 15000

[[seed_rules.tables.columns]]
name = "order_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "orders", column = "id" }
```

Seeding order: `users` → `orders` → `order_items`

### Self-Referential Foreign Keys

For hierarchical data (e.g., employee reports to manager):

```toml
[[seed_rules.tables]]
name = "employees"
count = 100

[[seed_rules.tables.columns]]
name = "manager_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "employees", column = "id" }
```

**Note:** Self-referential FKs may result in some NULL values to break circular dependencies.

## Size Presets

Use size presets to quickly scale data volume:

```bash
# Small: 100s of rows
dbarena seed --config seed.toml --container mydb --size small

# Medium: 1,000s of rows (default)
dbarena seed --config seed.toml --container mydb --size medium

# Large: 10,000s+ rows
dbarena seed --config seed.toml --container mydb --size large
```

Size presets multiply the `count` specified in your configuration:
- **Small**: 0.1x
- **Medium**: 1x
- **Large**: 10x

## Advanced Features

### Deterministic Seeding

Use the same seed value to generate identical data across runs:

```toml
global_seed = 42
```

```bash
# These two commands generate identical data
dbarena seed --config seed.toml --container mydb --seed 42
dbarena seed --config seed.toml --container mydb --seed 42
```

### Row Count Overrides

Override row counts from the CLI:

```bash
dbarena seed --config seed.toml --container mydb --rows users=500,orders=2000
```

### Incremental Seeding

Add more data to existing tables without truncation:

```bash
dbarena seed --config seed.toml --container mydb --incremental
```

### Truncate Before Seeding

Clear tables before seeding:

```bash
dbarena seed --config seed.toml --container mydb --truncate
```

### Batch Size Tuning

For large datasets, increase batch size for better performance:

```toml
batch_size = 5000  # Insert 5000 rows per batch
```

Larger batches = faster seeding, but use more memory.

## Examples

### Example 1: E-Commerce Database

```toml
global_seed = 42
batch_size = 1000

# Users
[[seed_rules.tables]]
name = "users"
count = 1000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

[[seed_rules.tables.columns]]
name = "email"
generator = "email"

[[seed_rules.tables.columns]]
name = "name"
generator = "name"
[seed_rules.tables.columns.options]
type = "full"

[[seed_rules.tables.columns]]
name = "created_at"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "range"
start = "2024-01-01"
end = "2026-01-01"

# Products
[[seed_rules.tables]]
name = "products"
count = 500

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"

[[seed_rules.tables.columns]]
name = "name"
generator = "template"
[seed_rules.tables.columns.options]
template = "Product {random_int:1000:9999}"

[[seed_rules.tables.columns]]
name = "price"
generator = "random_decimal"
[seed_rules.tables.columns.options]
min = 9.99
max = 999.99
precision = 2

[[seed_rules.tables.columns]]
name = "stock"
generator = "random_int"
[seed_rules.tables.columns.options]
min = 0
max = 1000

# Orders
[[seed_rules.tables]]
name = "orders"
count = 5000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"

[[seed_rules.tables.columns]]
name = "user_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "users", column = "id" }

[[seed_rules.tables.columns]]
name = "total"
generator = "random_decimal"
[seed_rules.tables.columns.options]
min = 10.0
max = 1000.0
precision = 2

[[seed_rules.tables.columns]]
name = "status"
generator = "enum"
[seed_rules.tables.columns.options]
values = ["pending", "processing", "shipped", "delivered"]

[[seed_rules.tables.columns]]
name = "created_at"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "range"
start = "2024-01-01"
end = "2026-01-25"
```

### Example 2: Time-Series IoT Data

```toml
global_seed = 123

# Devices
[[seed_rules.tables]]
name = "devices"
count = 100

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"

[[seed_rules.tables.columns]]
name = "device_type"
generator = "enum"
[seed_rules.tables.columns.options]
values = ["temperature", "humidity", "pressure", "motion"]

[[seed_rules.tables.columns]]
name = "location"
generator = "address"

# Sensor Readings
[[seed_rules.tables]]
name = "readings"
count = 100000  # Large volume of sensor data

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"

[[seed_rules.tables.columns]]
name = "device_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "devices", column = "id" }

[[seed_rules.tables.columns]]
name = "value"
generator = "random_decimal"
[seed_rules.tables.columns.options]
min = -50.0
max = 150.0
precision = 2

[[seed_rules.tables.columns]]
name = "timestamp"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "range"
start = "2026-01-01"
end = "2026-01-25"
```

## Best Practices

### 1. Start Small, Scale Up
Begin with small row counts to verify your configuration, then scale up:

```bash
# Test with small dataset
dbarena seed --config seed.toml --container mydb --size small

# Verify data
dbarena exec mydb -- psql -U postgres -c "SELECT * FROM users LIMIT 10"

# Scale up
dbarena seed --config seed.toml --container mydb --truncate --size large
```

### 2. Use Deterministic Seeds for Testing
For reproducible tests, always use `global_seed`:

```toml
global_seed = 42  # Same data every time
```

### 3. Optimize for Performance
- Increase `batch_size` for large datasets (5000-10000)
- Seed independent tables in parallel (automatic)
- Disable indexes before large seeding, rebuild after

### 4. Validate Foreign Keys
After seeding, verify no FK violations:

```bash
# PostgreSQL
dbarena exec mydb -- psql -U postgres -c "
  SELECT COUNT(*) FROM orders
  WHERE user_id NOT IN (SELECT id FROM users)"

# Should return 0
```

### 5. Use Realistic Data Patterns
Prefer realistic generators over sequential/random:
- ✅ Use `email` instead of `template: "user{random_int}@test.com"`
- ✅ Use `name` instead of `template: "User {sequential}"`
- ✅ Use `timestamp` with ranges instead of `now` for all rows

### 6. Document Your Seed Configurations
Add comments to explain the purpose of each table:

```toml
# Users table - 1000 test users across 2 years
[[seed_rules.tables]]
name = "users"
count = 1000
```

### 7. Version Control Your Configs
Store seed configurations in version control alongside schema definitions:

```
project/
├── schema/
│   └── ecommerce.sql
├── seed/
│   ├── seed-small.toml
│   ├── seed-medium.toml
│   └── seed-large.toml
```

## Troubleshooting

### Error: "Table not found"
Ensure the table exists in your database schema before seeding.

```bash
# Create schema first
dbarena exec mydb --file schema.sql

# Then seed
dbarena seed --config seed.toml --container mydb
```

### Error: "Foreign key violation"
Check that:
1. Referenced table is included in seed config
2. Referenced table has enough rows
3. Foreign key column uses `foreign_key` generator

### Slow Seeding Performance
- Increase `batch_size` (try 5000 or 10000)
- Reduce row counts for testing
- Check database resource limits

### Inconsistent Data Across Runs
- Set `global_seed` for deterministic generation
- Use `--seed` CLI flag: `dbarena seed --seed 42 ...`

## Next Steps

- See [Workload Generation Guide](workload.md) to generate transactions on seeded data
- See [Testing Guide](TESTING_PHASE8.md) for performance benchmarks
- Check [examples/](../examples/) for more seed configurations
