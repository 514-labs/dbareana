# dbarena v0.5.0 Tutorial: Complete Workflow

This tutorial walks you through a complete workflow using dbarena v0.5.0's data seeding and workload generation features.

**What You'll Learn:**
- Creating database containers
- Defining schemas
- Seeding realistic test data
- Running workload patterns
- Monitoring performance
- Comparing databases

**Time Required:** 30-45 minutes

## Prerequisites

- dbarena v0.5.0 installed
- Docker running
- Basic SQL knowledge

## Tutorial Scenario

You're building an e-commerce application and need to:
1. Test database performance with realistic data
2. Simulate shopping activity
3. Measure system performance under load
4. Compare PostgreSQL vs MySQL performance

## Step 1: Create Database Containers

Create PostgreSQL and MySQL containers for comparison:

```bash
# PostgreSQL container
dbarena create postgres --name ecom-pg --version 16

# MySQL container
dbarena create mysql --name ecom-mysql --version 8.0
```

**Expected Output:**
```
✓ Created container: ecom-pg (PostgreSQL 16)
Container ID: abc123...
Port: 5432 → 32768
Status: Running

✓ Created container: ecom-mysql (MySQL 8.0)
Container ID: def456...
Port: 3306 → 32769
Status: Running
```

**Verify:**
```bash
dbarena list
```

You should see both containers running.

## Step 2: Define the Schema

Create the e-commerce database schema:

```bash
cat > schema-ecommerce.sql << 'EOF'
-- Users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Products table
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price DECIMAL(10, 2),
    stock INTEGER DEFAULT 0,
    category VARCHAR(100)
);

-- Orders table
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    total DECIMAL(10, 2),
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Order items table
CREATE TABLE order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES orders(id),
    product_id INTEGER NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL,
    price DECIMAL(10, 2)
);

-- Create indexes for performance
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_order_items_product_id ON order_items(product_id);
EOF
```

**Apply the schema to both databases:**

```bash
# PostgreSQL
dbarena exec ecom-pg --file schema-ecommerce.sql

# MySQL (adjust syntax)
cat > schema-ecommerce-mysql.sql << 'EOF'
-- MySQL version of schema
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price DECIMAL(10, 2),
    stock INT DEFAULT 0,
    category VARCHAR(100)
);

CREATE TABLE orders (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    total DECIMAL(10, 2),
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE order_items (
    id INT AUTO_INCREMENT PRIMARY KEY,
    order_id INT NOT NULL,
    product_id INT NOT NULL,
    quantity INT NOT NULL,
    price DECIMAL(10, 2),
    FOREIGN KEY (order_id) REFERENCES orders(id),
    FOREIGN KEY (product_id) REFERENCES products(id)
);

CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_created_at ON orders(created_at);
CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_order_items_product_id ON order_items(product_id);
EOF

dbarena exec ecom-mysql --file schema-ecommerce-mysql.sql
```

**Verify schema:**
```bash
# PostgreSQL
dbarena exec ecom-pg -- psql -U postgres -c "\dt"

# MySQL
dbarena exec ecom-mysql -- mysql -u root -ppassword -e "SHOW TABLES"
```

## Step 3: Create Seed Configuration

Define how to populate the database with realistic test data:

```bash
cat > seed-ecommerce.toml << 'EOF'
# Deterministic seed for reproducibility
global_seed = 42
batch_size = 1000

# Users table - 1,000 customers
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

# Products table - 500 products
[[seed_rules.tables]]
name = "products"
count = 500

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

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

[[seed_rules.tables.columns]]
name = "category"
generator = "enum"
[seed_rules.tables.columns.options]
values = ["Electronics", "Clothing", "Home", "Books", "Toys", "Sports"]

# Orders table - 5,000 orders
[[seed_rules.tables]]
name = "orders"
count = 5000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

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
values = ["pending", "processing", "shipped", "delivered", "cancelled"]

[[seed_rules.tables.columns]]
name = "created_at"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "range"
start = "2024-01-01"
end = "2026-01-25"

# Order items table - 15,000 items (avg 3 per order)
[[seed_rules.tables]]
name = "order_items"
count = 15000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

[[seed_rules.tables.columns]]
name = "order_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "orders", column = "id" }

[[seed_rules.tables.columns]]
name = "product_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "products", column = "id" }

[[seed_rules.tables.columns]]
name = "quantity"
generator = "random_int"
[seed_rules.tables.columns.options]
min = 1
max = 5

[[seed_rules.tables.columns]]
name = "price"
generator = "random_decimal"
[seed_rules.tables.columns.options]
min = 9.99
max = 999.99
precision = 2
EOF
```

**Key Points:**
- `global_seed = 42` ensures reproducible data
- Tables seeded in dependency order: users → products → orders → order_items
- Foreign keys automatically reference existing data
- Realistic data: emails, names, timestamps, prices

## Step 4: Seed Both Databases

Populate both databases with identical data:

```bash
# Seed PostgreSQL
echo "Seeding PostgreSQL..."
dbarena seed --config seed-ecommerce.toml --container ecom-pg --seed 42

# Seed MySQL
echo "Seeding MySQL..."
dbarena seed --config seed-ecommerce.toml --container ecom-mysql --seed 42
```

**Expected Output:**
```
Seeding PostgreSQL...
✓ Seeded table: users (1,000 rows in 0.8s, 1,250 rows/sec)
✓ Seeded table: products (500 rows in 0.4s, 1,250 rows/sec)
✓ Seeded table: orders (5,000 rows in 3.2s, 1,562 rows/sec)
✓ Seeded table: order_items (15,000 rows in 9.1s, 1,648 rows/sec)

Total: 21,500 rows in 13.5 seconds
```

**Verify the data:**
```bash
# PostgreSQL
dbarena exec ecom-pg -- psql -U postgres -c "
  SELECT
    'users' as table_name, COUNT(*) as row_count FROM users
  UNION ALL
  SELECT 'products', COUNT(*) FROM products
  UNION ALL
  SELECT 'orders', COUNT(*) FROM orders
  UNION ALL
  SELECT 'order_items', COUNT(*) FROM order_items
"

# MySQL
dbarena exec ecom-mysql -- mysql -u root -ppassword -e "
  SELECT 'users' as table_name, COUNT(*) as row_count FROM users
  UNION ALL
  SELECT 'products', COUNT(*) FROM products
  UNION ALL
  SELECT 'orders', COUNT(*) FROM orders
  UNION ALL
  SELECT 'order_items', COUNT(*) FROM order_items
"
```

**Verify no FK violations:**
```bash
# PostgreSQL
dbarena exec ecom-pg -- psql -U postgres -c "
  SELECT COUNT(*) as invalid_orders
  FROM orders
  WHERE user_id NOT IN (SELECT id FROM users)
"
# Should return 0
```

## Step 5: Create Workload Configuration

Define a realistic e-commerce workload:

```bash
cat > workload-ecommerce.toml << 'EOF'
name = "E-commerce Shopping Simulation"
pattern = "ecommerce"
tables = ["users", "products", "orders", "order_items"]
connections = 50      # 50 concurrent shoppers
target_tps = 100      # 100 transactions per second
duration_seconds = 300  # Run for 5 minutes
EOF
```

**What This Simulates:**
- 50 concurrent users browsing/shopping
- 100 transactions per second (reads + writes)
- E-commerce pattern: 50% SELECT, 25% INSERT, 20% UPDATE, 5% DELETE
- 5-minute test duration

## Step 6: Run Workload on PostgreSQL

Test PostgreSQL performance:

```bash
dbarena workload run --config workload-ecommerce.toml --container ecom-pg
```

**Watch the live progress:**
```
======================================================================
Workload Progress
======================================================================

  ⏱ Time: 150.3s / 300.0s (50.1%)
     [=========================                         ]

  ⚡ TPS: 98.7 (target: 100)
     ✓ On target

  ✓ Success: 99.3% (14,805 / 14,911 total)

  ⏲ Latency:
     P50: 12.34ms
     P95: 45.67ms
     P99: 78.90ms

  📝 Operations:
     SELECT: 7,456 (50.0%)
     INSERT: 3,728 (25.0%)
     UPDATE: 2,982 (20.0%)
     DELETE: 745 (5.0%)

──────────────────────────────────────────────────────────────────────
  ℹ Press Ctrl+C to stop
```

**Final Results:**
```
======================================================================
Workload Complete
======================================================================

  Pattern: E-commerce Shopping Simulation
  Duration: 300.02s

  📊 Total Transactions: 30,003
     Successful: 29,889 (99.6%)
     Failed: 114

  ⚡ Throughput: 99.7 TPS

  ⏲ Latency:
     P50: 11.45ms
     P95: 42.31ms
     P99: 76.89ms
     Max: 234.56ms

  📝 Operation Distribution:
     SELECT: 15,002 (50.0%)
     INSERT: 7,501 (25.0%)
     UPDATE: 6,000 (20.0%)
     DELETE: 1,500 (5.0%)
```

## Step 7: Run Workload on MySQL

Test MySQL with identical workload:

```bash
dbarena workload run --config workload-ecommerce.toml --container ecom-mysql
```

**Compare Results:**
| Metric | PostgreSQL | MySQL |
|--------|-----------|-------|
| TPS | 99.7 | ? |
| P50 Latency | 11.45ms | ? |
| P99 Latency | 76.89ms | ? |
| Success Rate | 99.6% | ? |

## Step 8: Monitor in Real-Time (Optional)

For longer tests, monitor system metrics:

```bash
# Terminal 1: Start workload
dbarena workload run --config workload-ecommerce.toml --container ecom-pg

# Terminal 2: Monitor metrics
dbarena stats --multipane
```

**TUI Dashboard Shows:**
- CPU/Memory usage per container
- Database QPS/TPS
- Connection counts
- Workload progress
- Real-time graphs

## Step 9: Scale Up the Test

Try higher load:

```bash
# Increase to 500 TPS with 100 connections
dbarena workload run \
    --container ecom-pg \
    --pattern ecommerce \
    --tps 500 \
    --connections 100 \
    --duration 300
```

**Watch for:**
- Does TPS stay within ±10% of target?
- Does P99 latency stay <100ms?
- Does success rate stay >95%?

If not, you've found the database's capacity limit!

## Step 10: Try Different Patterns

Experiment with other workload patterns:

### OLTP Pattern (More Writes)
```bash
dbarena workload run \
    --container ecom-pg \
    --pattern oltp \
    --tps 200 \
    --duration 180
```

### Read-Heavy Pattern (Mostly Reads)
```bash
dbarena workload run \
    --container ecom-pg \
    --pattern read_heavy \
    --tps 500 \
    --duration 180
```

### Time-Series Pattern (High Inserts)
```bash
dbarena workload run \
    --container ecom-pg \
    --pattern time_series \
    --tps 1000 \
    --duration 180
```

## Step 11: Cleanup

When done testing:

```bash
# Stop and remove containers
dbarena destroy ecom-pg -y
dbarena destroy ecom-mysql -y

# Verify cleanup
dbarena list
```

## Next Steps

### Explore More Features

**1. Custom Workloads:**
Define specific SQL queries for your use case.
```bash
# See examples/workload-custom-sql.toml
```

**2. Larger Datasets:**
Test with 100K+ rows.
```bash
dbarena seed --config seed-ecommerce.toml --container mydb --size large
```

**3. Long-Duration Tests:**
Run workloads for hours to test stability.
```bash
dbarena workload run --container mydb --pattern ecommerce --tps 100 --duration 3600
```

**4. CDC Testing (v0.6.0):**
Enable Change Data Capture and monitor change events.
```bash
dbarena cdc enable --container mydb
dbarena workload run --container mydb --pattern ecommerce --tps 100 --duration 300
dbarena cdc monitor --container mydb
```

### Learn More

- [Data Seeding Guide](seeding.md) - Complete seeding reference
- [Workload Generation Guide](workload.md) - Detailed workload docs
- [Testing Guide](TESTING_PHASE8.md) - Performance benchmarks
- [v0.5.0 Spec](../specs/v0.5.0/VERSION_OVERVIEW.md) - Technical details

## Troubleshooting

### Seeding is Slow
- Increase `batch_size` in config (try 5000)
- Check database has sufficient memory

### Workload TPS Too Low
- Reduce target TPS
- Increase `connections`
- Check database CPU usage

### High Error Rate
- Check database logs
- Reduce TPS or connections
- Verify schema and data are correct

### Containers Won't Start
- Check Docker is running: `docker ps`
- Check port conflicts: `dbarena list`
- Review logs: `dbarena logs <container>`

## Summary

Congratulations! You've:
- ✅ Created database containers
- ✅ Defined an e-commerce schema
- ✅ Seeded realistic test data (21,500 rows)
- ✅ Ran workload simulations
- ✅ Measured performance metrics
- ✅ Compared two databases

You now have the skills to:
- Test database performance with realistic data
- Simulate production-like workloads
- Measure and compare database systems
- Find capacity limits and bottlenecks

## Feedback

Found an issue or have suggestions?
[Open an issue](https://github.com/anthropics/claude-code/issues)
