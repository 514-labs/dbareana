# dbarena Configuration Examples

This directory contains example configurations for data seeding and workload generation.

## Seed Configurations

### seed-basic.toml
**Use Case:** Getting started with seeding
**Tables:** 1 (users)
**Rows:** 100 users
**Complexity:** Minimal - single table, basic generators

```bash
dbarena seed --config examples/seed-basic.toml --container mydb
```

### seed-ecommerce.toml
**Use Case:** E-commerce database testing
**Tables:** 4 (users, products, orders, order_items)
**Rows:** 21,500 total (1K users, 500 products, 5K orders, 15K items)
**Complexity:** Medium - foreign key relationships, realistic data

```bash
dbarena seed --config examples/seed-ecommerce.toml --container mydb --size medium
```

**Required Schema:**
```sql
CREATE TABLE users (id SERIAL PRIMARY KEY, email VARCHAR(255), name VARCHAR(255), created_at TIMESTAMP);
CREATE TABLE products (id SERIAL PRIMARY KEY, name VARCHAR(255), price DECIMAL(10,2), stock INT, category VARCHAR(100));
CREATE TABLE orders (id SERIAL PRIMARY KEY, user_id INT REFERENCES users(id), total DECIMAL(10,2), status VARCHAR(50), created_at TIMESTAMP);
CREATE TABLE order_items (id SERIAL PRIMARY KEY, order_id INT REFERENCES orders(id), product_id INT REFERENCES products(id), quantity INT, price DECIMAL(10,2));
```

### seed-timeseries.toml
**Use Case:** IoT/sensor data testing
**Tables:** 2 (devices, readings)
**Rows:** 100,100 total (100 devices, 100K readings)
**Complexity:** High volume - optimized for large datasets

```bash
dbarena seed --config examples/seed-timeseries.toml --container mydb --size large
```

**Required Schema:**
```sql
CREATE TABLE devices (id SERIAL PRIMARY KEY, device_type VARCHAR(50), location VARCHAR(255));
CREATE TABLE readings (id SERIAL PRIMARY KEY, device_id INT REFERENCES devices(id), value DECIMAL(10,2), timestamp TIMESTAMP);
```

## Workload Configurations

### workload-oltp.toml
**Use Case:** OLTP transaction testing
**Pattern:** Built-in OLTP pattern
**TPS:** 200
**Connections:** 50
**Mix:** 40% SELECT, 30% INSERT, 25% UPDATE, 5% DELETE

```bash
dbarena workload run --config examples/workload-oltp.toml --container mydb
```

### workload-ecommerce.toml
**Use Case:** E-commerce shopping simulation
**Pattern:** Built-in E-commerce pattern
**TPS:** 100
**Connections:** 50
**Mix:** 50% SELECT, 25% INSERT, 20% UPDATE, 5% DELETE

```bash
dbarena workload run --config examples/workload-ecommerce.toml --container mydb
```

**Requires:** Data seeded with `seed-ecommerce.toml`

### workload-custom-mix.toml
**Use Case:** Custom operation distribution
**Pattern:** Custom weights
**TPS:** 50
**Connections:** 20
**Mix:** 85% SELECT, 5% INSERT, 8% UPDATE, 2% DELETE

```bash
dbarena workload run --config examples/workload-custom-mix.toml --container mydb
```

**Features:**
- Custom operation weights
- JOIN queries enabled
- Aggregation queries enabled
- Read-heavy analytics pattern

### workload-custom-sql.toml
**Use Case:** Specific SQL query testing
**Pattern:** Custom SQL queries
**TPS:** 50
**Connections:** 20
**Queries:** 4 specific queries with parameterization

```bash
dbarena workload run --config examples/workload-custom-sql.toml --container mydb
```

**Features:**
- Parameterized SQL queries
- Query-specific weights
- Foreign key parameter generation
- Enum and random value generation

## Quick Start Workflows

### Workflow 1: Basic Testing

```bash
# 1. Create container
dbarena create postgres --name test-db

# 2. Seed basic data
dbarena seed --config examples/seed-basic.toml --container test-db

# 3. Run balanced workload
dbarena workload run --container test-db --pattern balanced --tps 50 --duration 60

# 4. Cleanup
dbarena destroy test-db -y
```

### Workflow 2: E-commerce Complete

```bash
# 1. Create container
dbarena create postgres --name ecom-db

# 2. Create schema
cat > schema.sql << 'EOF'
CREATE TABLE users (id SERIAL PRIMARY KEY, email VARCHAR(255), name VARCHAR(255), created_at TIMESTAMP);
CREATE TABLE products (id SERIAL PRIMARY KEY, name VARCHAR(255), price DECIMAL(10,2), stock INT, category VARCHAR(100));
CREATE TABLE orders (id SERIAL PRIMARY KEY, user_id INT REFERENCES users(id), total DECIMAL(10,2), status VARCHAR(50), created_at TIMESTAMP);
CREATE TABLE order_items (id SERIAL PRIMARY KEY, order_id INT REFERENCES orders(id), product_id INT REFERENCES products(id), quantity INT, price DECIMAL(10,2));
EOF
dbarena exec ecom-db --file schema.sql

# 3. Seed e-commerce data
dbarena seed --config examples/seed-ecommerce.toml --container ecom-db --size medium

# 4. Run e-commerce workload
dbarena workload run --config examples/workload-ecommerce.toml --container ecom-db

# 5. Cleanup
dbarena destroy ecom-db -y
```

### Workflow 3: Performance Comparison

```bash
# Test PostgreSQL
dbarena create postgres --name pg-test
dbarena seed --config examples/seed-ecommerce.toml --container pg-test --seed 42
dbarena workload run --config examples/workload-oltp.toml --container pg-test

# Test MySQL (identical data and workload)
dbarena create mysql --name mysql-test
dbarena seed --config examples/seed-ecommerce.toml --container mysql-test --seed 42
dbarena workload run --config examples/workload-oltp.toml --container mysql-test

# Compare results
dbarena destroy pg-test mysql-test -y
```

## Configuration Tips

### Size Presets

Scale data volume with size presets:

```bash
# Small: 100s of rows
dbarena seed --config examples/seed-ecommerce.toml --container mydb --size small

# Medium: 1,000s of rows (default)
dbarena seed --config examples/seed-ecommerce.toml --container mydb --size medium

# Large: 10,000s of rows
dbarena seed --config examples/seed-ecommerce.toml --container mydb --size large
```

### Row Count Overrides

Override specific table counts:

```bash
dbarena seed --config examples/seed-ecommerce.toml --container mydb --rows users=5000,orders=20000
```

### TPS and Duration

Adjust workload intensity:

```bash
# Light load: 50 TPS for 1 minute
dbarena workload run --container mydb --pattern oltp --tps 50 --duration 60

# Heavy load: 500 TPS for 10 minutes
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 600

# Fixed transaction count: run until 10K transactions
dbarena workload run --container mydb --pattern oltp --tps 100 --transactions 10000
```

## Customization

### Create Your Own Configurations

1. **Copy an Example:**
```bash
cp examples/seed-ecommerce.toml my-seed-config.toml
```

2. **Modify as Needed:**
- Adjust table names
- Change row counts
- Add/remove columns
- Modify generator types

3. **Test:**
```bash
dbarena seed --config my-seed-config.toml --container mydb --size small
```

4. **Scale Up:**
```bash
dbarena seed --config my-seed-config.toml --container mydb --size large
```

## Documentation

For more details, see:
- [Data Seeding Guide](../docs/seeding.md)
- [Workload Generation Guide](../docs/workload.md)
- [Tutorial](../docs/tutorial-v0.5.md)
- [v0.5.0 Specification](../specs/v0.5.0/VERSION_OVERVIEW.md)

## Feedback

Found an issue or have suggestions?
[Open an issue](https://github.com/anthropics/claude-code/issues)
