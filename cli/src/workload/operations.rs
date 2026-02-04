use anyhow::{anyhow, Result};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::container::DatabaseType;
use crate::workload::metadata::TableMetadata;

/// SQL operation type
#[derive(Debug, Clone)]
pub enum Operation {
    Select(String),
    Insert(String),
    Update(String),
    Delete(String),
}

impl Operation {
    pub fn sql(&self) -> &str {
        match self {
            Operation::Select(sql) => sql,
            Operation::Insert(sql) => sql,
            Operation::Update(sql) => sql,
            Operation::Delete(sql) => sql,
        }
    }

    pub fn operation_type(&self) -> &str {
        match self {
            Operation::Select(_) => "SELECT",
            Operation::Insert(_) => "INSERT",
            Operation::Update(_) => "UPDATE",
            Operation::Delete(_) => "DELETE",
        }
    }
}

/// Generates realistic SQL operations
pub struct OperationGenerator {
    db_type: DatabaseType,
}

impl OperationGenerator {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// Generate a SELECT operation
    pub fn generate_select(
        &self,
        metadata: &TableMetadata,
        rng: &mut ChaCha8Rng,
    ) -> Result<Operation> {
        let pk = metadata
            .primary_key_column()
            .ok_or_else(|| anyhow!("No primary key found for table: {}", metadata.name))?;

        // Generate random ID within estimated range
        let id = if metadata.row_count_estimate > 0 {
            rng.gen_range(1..=metadata.row_count_estimate)
        } else {
            rng.gen_range(1..=1000)
        };

        let sql = format!(
            "SELECT * FROM {} WHERE {} = {} LIMIT 1",
            escape_identifier(&metadata.name, self.db_type),
            escape_identifier(&pk.name, self.db_type),
            id
        );

        Ok(Operation::Select(sql))
    }

    /// Generate an INSERT operation with realistic data
    pub fn generate_insert(
        &self,
        metadata: &TableMetadata,
        rng: &mut ChaCha8Rng,
    ) -> Result<Operation> {
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for col in &metadata.columns {
            // Skip auto-increment primary keys
            if col.is_primary_key && col.data_type.to_lowercase().contains("serial") {
                continue;
            }

            columns.push(escape_identifier(&col.name, self.db_type));

            // Generate value based on type
            let value = if col.is_primary_key {
                // For non-auto-increment PKs, generate a large random ID
                rng.gen_range(1000000..9999999).to_string()
            } else {
                generate_value_for_type(&col.data_type, rng)
            };

            values.push(value);
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            escape_identifier(&metadata.name, self.db_type),
            columns.join(", "),
            values.join(", ")
        );

        Ok(Operation::Insert(sql))
    }

    /// Generate an UPDATE operation
    pub fn generate_update(
        &self,
        metadata: &TableMetadata,
        rng: &mut ChaCha8Rng,
    ) -> Result<Operation> {
        let pk = metadata
            .primary_key_column()
            .ok_or_else(|| anyhow!("No primary key found for table: {}", metadata.name))?;

        let updatable = metadata.updatable_columns();
        if updatable.is_empty() {
            return Err(anyhow!("No updatable columns in table: {}", metadata.name));
        }

        // Pick random column to update
        let col_idx = rng.gen_range(0..updatable.len());
        let col = updatable[col_idx];

        // Generate new value
        let new_value = generate_value_for_type(&col.data_type, rng);

        // Generate random ID
        let id = if metadata.row_count_estimate > 0 {
            rng.gen_range(1..=metadata.row_count_estimate)
        } else {
            rng.gen_range(1..=1000)
        };

        let sql = format!(
            "UPDATE {} SET {} = {} WHERE {} = {}",
            escape_identifier(&metadata.name, self.db_type),
            escape_identifier(&col.name, self.db_type),
            new_value,
            escape_identifier(&pk.name, self.db_type),
            id
        );

        Ok(Operation::Update(sql))
    }

    /// Generate a DELETE operation (safe - uses non-existent ID)
    pub fn generate_delete(
        &self,
        metadata: &TableMetadata,
        rng: &mut ChaCha8Rng,
    ) -> Result<Operation> {
        let pk = metadata
            .primary_key_column()
            .ok_or_else(|| anyhow!("No primary key found for table: {}", metadata.name))?;

        // Use a high ID that likely doesn't exist (safe delete)
        let id = rng.gen_range(900000000..999999999);

        let sql = format!(
            "DELETE FROM {} WHERE {} = {}",
            escape_identifier(&metadata.name, self.db_type),
            escape_identifier(&pk.name, self.db_type),
            id
        );

        Ok(Operation::Delete(sql))
    }

    /// Generate a SELECT with JOIN (for OLAP/reporting patterns)
    pub fn generate_select_with_join(
        &self,
        table1: &TableMetadata,
        table2: &TableMetadata,
        rng: &mut ChaCha8Rng,
    ) -> Result<Operation> {
        let id = rng.gen_range(1..=100);

        let sql = format!(
            "SELECT t1.*, t2.* FROM {} t1 JOIN {} t2 ON t1.id = t2.id WHERE t1.id > {} LIMIT 10",
            escape_identifier(&table1.name, self.db_type),
            escape_identifier(&table2.name, self.db_type),
            id
        );

        Ok(Operation::Select(sql))
    }

    /// Generate a SELECT with aggregation
    pub fn generate_select_with_aggregation(
        &self,
        metadata: &TableMetadata,
        _rng: &mut ChaCha8Rng,
    ) -> Result<Operation> {
        let pk = metadata
            .primary_key_column()
            .ok_or_else(|| anyhow!("No primary key found for table: {}", metadata.name))?;

        let sql = format!(
            "SELECT COUNT(*), MAX({}) FROM {}",
            escape_identifier(&pk.name, self.db_type),
            escape_identifier(&metadata.name, self.db_type)
        );

        Ok(Operation::Select(sql))
    }
}

/// Escape identifier based on database type
fn escape_identifier(name: &str, db_type: DatabaseType) -> String {
    match db_type {
        DatabaseType::Postgres => format!("\"{}\"", name.replace('"', "\"\"")),
        DatabaseType::MySQL => format!("`{}`", name.replace('`', "``")),
        DatabaseType::SQLServer => format!("[{}]", name.replace(']', "]]")),
    }
}

/// Generate a realistic value for a column type
fn generate_value_for_type(data_type: &str, rng: &mut ChaCha8Rng) -> String {
    let type_lower = data_type.to_lowercase();

    if type_lower.contains("int") || type_lower.contains("serial") {
        rng.gen_range(1..1000).to_string()
    } else if type_lower.contains("decimal") || type_lower.contains("numeric") || type_lower.contains("float") || type_lower.contains("double") {
        format!("{:.2}", rng.gen_range(1.0..1000.0))
    } else if type_lower.contains("bool") {
        if rng.gen_bool(0.5) { "true" } else { "false" }.to_string()
    } else if type_lower.contains("date") || type_lower.contains("time") {
        "'2024-01-01 12:00:00'".to_string()
    } else {
        // String types
        let words = ["test", "data", "value", "sample", "item"];
        let word = words[rng.gen_range(0..words.len())];
        let num = rng.gen_range(1..9999);
        format!("'{} {}'", word, num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workload::metadata::ColumnMetadata;

    fn test_metadata() -> TableMetadata {
        TableMetadata {
            name: "users".to_string(),
            columns: vec![
                ColumnMetadata {
                    name: "id".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: false,
                    is_primary_key: true,
                },
                ColumnMetadata {
                    name: "name".to_string(),
                    data_type: "varchar".to_string(),
                    is_nullable: true,
                    is_primary_key: false,
                },
                ColumnMetadata {
                    name: "age".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: true,
                    is_primary_key: false,
                },
            ],
            primary_key: Some("id".to_string()),
            row_count_estimate: 1000,
        }
    }

    #[test]
    fn test_generate_select() {
        let gen = OperationGenerator::new(DatabaseType::Postgres);
        let metadata = test_metadata();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let op = gen.generate_select(&metadata, &mut rng).unwrap();
        let sql = op.sql();

        assert!(sql.contains("SELECT"));
        assert!(sql.contains("FROM"));
        assert!(sql.contains("users"));
        assert!(sql.contains("WHERE"));
        assert_eq!(op.operation_type(), "SELECT");
    }

    #[test]
    fn test_generate_insert() {
        let gen = OperationGenerator::new(DatabaseType::Postgres);
        let metadata = test_metadata();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let op = gen.generate_insert(&metadata, &mut rng).unwrap();
        let sql = op.sql();

        assert!(sql.contains("INSERT INTO"));
        assert!(sql.contains("users"));
        assert!(sql.contains("VALUES"));
        assert_eq!(op.operation_type(), "INSERT");
    }

    #[test]
    fn test_generate_update() {
        let gen = OperationGenerator::new(DatabaseType::Postgres);
        let metadata = test_metadata();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let op = gen.generate_update(&metadata, &mut rng).unwrap();
        let sql = op.sql();

        assert!(sql.contains("UPDATE"));
        assert!(sql.contains("users"));
        assert!(sql.contains("SET"));
        assert!(sql.contains("WHERE"));
        assert_eq!(op.operation_type(), "UPDATE");
    }

    #[test]
    fn test_generate_delete() {
        let gen = OperationGenerator::new(DatabaseType::Postgres);
        let metadata = test_metadata();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let op = gen.generate_delete(&metadata, &mut rng).unwrap();
        let sql = op.sql();

        assert!(sql.contains("DELETE FROM"));
        assert!(sql.contains("users"));
        assert!(sql.contains("WHERE"));
        assert_eq!(op.operation_type(), "DELETE");
    }

    #[test]
    fn test_escape_identifiers() {
        assert_eq!(
            escape_identifier("users", DatabaseType::Postgres),
            "\"users\""
        );
        assert_eq!(
            escape_identifier("users", DatabaseType::MySQL),
            "`users`"
        );
        assert_eq!(
            escape_identifier("users", DatabaseType::SQLServer),
            "[users]"
        );
    }

    #[test]
    fn test_generate_value_types() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let int_val = generate_value_for_type("integer", &mut rng);
        assert!(int_val.parse::<i32>().is_ok());

        let str_val = generate_value_for_type("varchar", &mut rng);
        assert!(str_val.starts_with('\''));
        assert!(str_val.ends_with('\''));

        let bool_val = generate_value_for_type("boolean", &mut rng);
        assert!(bool_val == "true" || bool_val == "false");
    }

    #[test]
    fn test_generate_aggregation() {
        let gen = OperationGenerator::new(DatabaseType::Postgres);
        let metadata = test_metadata();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let op = gen.generate_select_with_aggregation(&metadata, &mut rng).unwrap();
        let sql = op.sql();

        assert!(sql.contains("COUNT(*)"));
        assert!(sql.contains("MAX("));
    }
}
