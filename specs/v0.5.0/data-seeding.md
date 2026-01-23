# Data Seeding

## Feature Overview

Comprehensive data seeding system that populates databases with realistic test data based on schema definitions. Supports multiple data generation strategies, volume scaling, and maintains referential integrity across tables.

## Problem Statement

Testing databases requires representative data:
- Writing manual INSERT statements is time-consuming and doesn't scale
- Generating realistic data (names, emails, timestamps) is error-prone
- Maintaining foreign key relationships manually is tedious
- Cross-database testing requires identical datasets in different dialects
- Performance testing needs large volumes of data quickly

Without automated data seeding, users spend significant time on data preparation instead of actual testing.

## User Stories

**As a CDC developer**, I want to:
- Seed a database with 10,000 user records and 50,000 orders in minutes
- Generate realistic email addresses and timestamps (not test1@test.com)
- Ensure all foreign keys reference valid records
- Reproduce the same dataset across test runs using a seed value

**As a QA engineer**, I want to:
- Define test data requirements in configuration files
- Generate edge cases (NULL values, boundary conditions)
- Seed databases quickly as part of test setup
- Clear and reseed data between test scenarios

**As a performance engineer**, I want to:
- Generate millions of rows for load testing
- Control data distribution (e.g., 70% active users, 30% inactive)
- Create time-series data with realistic temporal patterns
- Measure seeding performance itself

## Technical Requirements

### Functional Requirements

**FR-1: Data Generation Strategies**
- **Sequential**: Integer sequences (1, 2, 3, ...)
- **Random**: Random values within constraints
- **Realistic**: Faker-like data (names, emails, addresses, phone numbers)
- **Template**: User-defined templates with placeholders
- **Referenced**: Values from related tables (for foreign keys)
- **Timestamp**: Current time or time ranges with realistic distributions
- **Enum**: Select from predefined value lists

**FR-2: Data Type Support**
- Integer types: Sequential or random within range
- String types: Fixed length, variable length, templates, realistic names/emails
- Timestamp types: Now, relative to now, or within date ranges
- Boolean: True/false with configurable distribution
- Decimal/Numeric: Random within precision constraints
- JSON: Structured JSON with variable fields

**FR-3: Referential Integrity**
- Detect foreign key relationships from schema
- Seed parent tables before child tables
- Generate foreign key values from existing parent rows
- Handle self-referential foreign keys
- Support optional (nullable) foreign keys

**FR-4: Volume Scaling**
- Named presets: `--size small` (100-1000 rows), `medium` (1000-10000), `large` (10000-100000)
- Custom row counts per table
- Batch inserts for performance (1000 rows per batch)
- Progress indicators for large datasets

**FR-5: Configuration Format**
```toml
[[seed_rules]]
table = "users"
count = 1000

  [[seed_rules.columns]]
  name = "email"
  generator = "email"

  [[seed_rules.columns]]
  name = "created_at"
  generator = "timestamp"
  range = "last_30_days"

  [[seed_rules.columns]]
  name = "is_active"
  generator = "boolean"
  true_probability = 0.7

[[seed_rules]]
table = "orders"
count = 5000

  [[seed_rules.columns]]
  name = "user_id"
  generator = "foreign_key"
  references = "users.id"

  [[seed_rules.columns]]
  name = "total_amount"
  generator = "decimal"
  min = 10.00
  max = 1000.00
  precision = 2
```

**FR-6: Incremental Seeding**
- Add rows to existing tables
- Respect existing ID sequences
- Avoid duplicate unique values
- Option to truncate before seeding

**FR-7: Reproducibility**
- `--seed <value>` option for deterministic randomness
- Same seed produces identical data across runs
- Document seed value in output for later reproduction

### Non-Functional Requirements

**NFR-1: Performance**
- Seed 1,000 rows in <5 seconds
- Seed 100,000 rows in <60 seconds
- Use batch inserts (1000 rows per batch) for efficiency
- Parallel seeding for independent tables

**NFR-2: Data Quality**
- Generated emails pass basic validation (contain @, domain)
- Timestamps are valid and distributed realistically
- No foreign key constraint violations
- Unique constraints respected

## Implementation Details

### Dependencies

```toml
[dependencies]
fake = "2.9"                   # Faker-like data generation
rand = "0.8"                   # Random number generation
rand_chacha = "0.3"            # Deterministic RNG
sqlx = { version = "0.7", features = ["postgres", "mysql", "mssql"] }
tokio = { version = "1.36", features = ["full"] }
```

### Data Generator Interface

```rust
use fake::{Fake, Faker};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub trait DataGenerator: Send + Sync {
    fn generate(&self, rng: &mut ChaCha8Rng) -> String;
}

pub struct EmailGenerator;
impl DataGenerator for EmailGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> String {
        use fake::faker::internet::en::SafeEmail;
        SafeEmail().fake_with_rng(rng)
    }
}

pub struct SequentialGenerator {
    current: AtomicUsize,
    start: usize,
}

impl DataGenerator for SequentialGenerator {
    fn generate(&self, _rng: &mut ChaCha8Rng) -> String {
        self.current.fetch_add(1, Ordering::SeqCst).to_string()
    }
}

pub struct TimestampGenerator {
    base: DateTime<Utc>,
    range_days: i64,
}

impl DataGenerator for TimestampGenerator {
    fn generate(&self, rng: &mut ChaCha8Rng) -> String {
        use rand::Rng;
        let offset_seconds = rng.gen_range(0..(self.range_days * 86400));
        let timestamp = self.base + Duration::seconds(offset_seconds);
        timestamp.to_rfc3339()
    }
}
```

### Seeding Engine

```rust
pub struct SeedingEngine {
    db_connection: DatabaseConnection,
    rng: ChaCha8Rng,
    batch_size: usize,
}

impl SeedingEngine {
    pub async fn seed_table(&mut self, rule: &SeedRule) -> Result<()> {
        let mut rows_generated = 0;
        let mut batch = Vec::with_capacity(self.batch_size);

        while rows_generated < rule.count {
            // Generate a row
            let row = self.generate_row(rule)?;
            batch.push(row);

            // Insert batch when full
            if batch.len() >= self.batch_size {
                self.insert_batch(&rule.table, &batch).await?;
                batch.clear();
                rows_generated += self.batch_size;

                // Show progress
                self.report_progress(rule.table, rows_generated, rule.count);
            }
        }

        // Insert remaining rows
        if !batch.is_empty() {
            self.insert_batch(&rule.table, &batch).await?;
        }

        Ok(())
    }

    fn generate_row(&mut self, rule: &SeedRule) -> Result<HashMap<String, String>> {
        let mut row = HashMap::new();

        for column_rule in &rule.columns {
            let generator = self.get_generator(&column_rule.generator)?;
            let value = generator.generate(&mut self.rng);
            row.insert(column_rule.name.clone(), value);
        }

        Ok(row)
    }

    async fn insert_batch(&self, table: &str, rows: &[HashMap<String, String>]) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Build INSERT statement
        let columns: Vec<&String> = rows[0].keys().collect();
        let placeholders = self.build_placeholders(rows.len(), columns.len());

        let query = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table,
            columns.join(", "),
            placeholders
        );

        // Flatten values
        let mut values = Vec::new();
        for row in rows {
            for col in &columns {
                values.push(row.get(*col).unwrap().clone());
            }
        }

        // Execute
        sqlx::query(&query)
            .bind_many(values)
            .execute(&self.db_connection)
            .await?;

        Ok(())
    }
}
```

### Foreign Key Resolution

```rust
pub struct ForeignKeyResolver {
    cache: HashMap<String, Vec<i64>>,  // table.column -> list of IDs
}

impl ForeignKeyResolver {
    pub async fn resolve(&mut self, table: &str, column: &str, db: &DatabaseConnection) -> Result<Vec<i64>> {
        let cache_key = format!("{}.{}", table, column);

        if let Some(ids) = self.cache.get(&cache_key) {
            return Ok(ids.clone());
        }

        // Query for all IDs in the referenced table
        let query = format!("SELECT {} FROM {}", column, table);
        let rows = sqlx::query(&query)
            .fetch_all(db)
            .await?;

        let ids: Vec<i64> = rows
            .iter()
            .map(|row| row.get(0))
            .collect();

        self.cache.insert(cache_key, ids.clone());
        Ok(ids)
    }

    pub fn random_id(&self, table: &str, column: &str, rng: &mut ChaCha8Rng) -> Result<i64> {
        let cache_key = format!("{}.{}", table, column);
        let ids = self.cache.get(&cache_key)
            .ok_or_else(|| anyhow!("No IDs cached for {}", cache_key))?;

        use rand::seq::SliceRandom;
        ids.choose(rng)
            .copied()
            .ok_or_else(|| anyhow!("No IDs available"))
    }
}
```

## CLI Interface Design

### Commands

```bash
# Seed database from configuration
simdb seed --config <config-file> --container <name>

# Seed with specific size preset
simdb seed --config <config-file> --container <name> --size medium

# Seed with custom row counts
simdb seed --config <config-file> --container <name> --rows users=1000,orders=5000

# Seed with deterministic seed for reproducibility
simdb seed --config <config-file> --container <name> --seed 12345

# Incremental seeding (add to existing data)
simdb seed --config <config-file> --container <name> --incremental

# Truncate tables before seeding
simdb seed --config <config-file> --container <name> --truncate
```

### Example Usage

```bash
# Seed using configuration
$ simdb seed --config my-schema.toml --container simdb-postgres-16-a3f9 --size small
Seeding database: simdb-postgres-16-a3f9
  ✓ Analyzing schema (2 tables, 1 foreign key)
  ✓ users: 1000 rows (100%) [=================] 2.3s
  ✓ orders: 5000 rows (100%) [=================] 8.7s
Seeding complete! 6000 total rows in 11.2s

# Seed with custom seed value for reproducibility
$ simdb seed --config my-schema.toml --container simdb-postgres-16-a3f9 --seed 42
Using seed: 42 (use this value to reproduce exact data)
Seeding database: simdb-postgres-16-a3f9
  ...

# Large dataset
$ simdb seed --config my-schema.toml --container simdb-postgres-16-a3f9 --size large
Seeding database: simdb-postgres-16-a3f9
  ✓ users: 10000 rows (100%) [=================] 18.5s
  ✓ orders: 100000 rows (100%) [===============] 125.3s
Seeding complete! 110000 total rows in 144.1s
```

## Testing Strategy

### Unit Tests
- `test_email_generation()`: Verify valid emails generated
- `test_sequential_generator()`: Verify sequential IDs
- `test_timestamp_generation()`: Verify timestamps in range
- `test_foreign_key_resolution()`: Verify FK references are valid
- `test_batch_insert_query_building()`: Verify correct SQL generated

### Integration Tests
- `test_seed_postgres()`: Seed PostgreSQL, verify row counts and constraints
- `test_seed_mysql()`: Seed MySQL, verify data
- `test_seed_sqlserver()`: Seed SQL Server, verify data
- `test_foreign_key_seeding()`: Seed tables with FK relationships, verify integrity
- `test_reproducible_seeding()`: Use same seed twice, verify identical data
- `test_incremental_seeding()`: Seed, then seed again incrementally
- `test_large_dataset()`: Seed 100K rows, verify performance

### Manual Testing
1. **Small Dataset**: Seed 100 rows, manually inspect data quality
2. **Foreign Keys**: Seed parent/child tables, verify relationships
3. **Reproducibility**: Use same seed value, compare data across runs
4. **Performance**: Seed 1M rows, measure time and resource usage
5. **Cross-Database**: Seed identical data to all three databases, compare

## Documentation Requirements
- **Data Generators Reference**: List of available generators and options
- **Configuration Examples**: Sample seeding configurations for common scenarios
- **Performance Guide**: Expected seeding rates and optimization tips
- **Troubleshooting**: Common errors (constraint violations, out of memory)

## Future Enhancements
- Import data from CSV/JSON files
- Custom generator plugins (user-provided Rust code)
- Data masking (anonymize production data for testing)
- Differential seeding (only seed changed tables)
- Streaming seeding for very large datasets (>1M rows)
- Graph-aware seeding for complex relationship networks
