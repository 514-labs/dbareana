use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::container::DatabaseType;
use crate::database_metrics::collector::DockerDatabaseMetricsCollector;

/// Metadata about a database table
#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub name: String,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key: Option<String>,
    pub row_count_estimate: usize,
}

/// Metadata about a table column
#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

impl TableMetadata {
    /// Get non-primary-key columns that can be updated
    pub fn updatable_columns(&self) -> Vec<&ColumnMetadata> {
        self.columns
            .iter()
            .filter(|c| !c.is_primary_key)
            .collect()
    }

    /// Get the primary key column
    pub fn primary_key_column(&self) -> Option<&ColumnMetadata> {
        self.columns.iter().find(|c| c.is_primary_key)
    }
}

/// Collects and caches table metadata
pub struct MetadataCollector {
    collector: DockerDatabaseMetricsCollector,
    container_id: String,
    db_type: DatabaseType,
    cache: HashMap<String, TableMetadata>,
}

impl MetadataCollector {
    pub fn new(
        collector: DockerDatabaseMetricsCollector,
        container_id: String,
        db_type: DatabaseType,
    ) -> Self {
        Self {
            collector,
            container_id,
            db_type,
            cache: HashMap::new(),
        }
    }

    /// Get metadata for a table (cached)
    pub async fn get_metadata(&mut self, table: &str) -> Result<&TableMetadata> {
        if !self.cache.contains_key(table) {
            let metadata = self.collect_table_metadata(table).await?;
            self.cache.insert(table.to_string(), metadata);
        }

        self.cache
            .get(table)
            .ok_or_else(|| anyhow!("Failed to get cached metadata for table: {}", table))
    }

    /// Collect metadata for a table
    async fn collect_table_metadata(&self, table: &str) -> Result<TableMetadata> {
        let columns = self.get_columns(table).await?;
        let row_count = self.get_row_count(table).await.unwrap_or(1000);

        // Find primary key from columns
        let primary_key = columns
            .iter()
            .find(|c| c.is_primary_key)
            .map(|c| c.name.clone());

        Ok(TableMetadata {
            name: table.to_string(),
            columns,
            primary_key,
            row_count_estimate: row_count,
        })
    }

    /// Get column information for a table
    async fn get_columns(&self, table: &str) -> Result<Vec<ColumnMetadata>> {
        let query = match self.db_type {
            DatabaseType::Postgres => format!(
                "SELECT column_name, data_type, is_nullable, \
                 CASE WHEN column_name IN (\
                   SELECT a.attname FROM pg_index i \
                   JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey) \
                   WHERE i.indrelid = '{}'::regclass AND i.indisprimary\
                 ) THEN 'YES' ELSE 'NO' END as is_primary \
                 FROM information_schema.columns \
                 WHERE table_name = '{}' \
                 ORDER BY ordinal_position",
                table, table
            ),
            DatabaseType::MySQL => format!(
                "SELECT COLUMN_NAME as column_name, DATA_TYPE as data_type, \
                 IS_NULLABLE as is_nullable, COLUMN_KEY as is_primary \
                 FROM information_schema.COLUMNS \
                 WHERE TABLE_NAME = '{}' AND TABLE_SCHEMA = DATABASE() \
                 ORDER BY ORDINAL_POSITION",
                table
            ),
            DatabaseType::SQLServer => format!(
                "SELECT c.name as column_name, t.name as data_type, \
                 c.is_nullable, CASE WHEN ic.object_id IS NOT NULL THEN 'YES' ELSE 'NO' END as is_primary \
                 FROM sys.columns c \
                 JOIN sys.types t ON c.user_type_id = t.user_type_id \
                 LEFT JOIN sys.index_columns ic ON ic.object_id = c.object_id AND ic.column_id = c.column_id \
                 LEFT JOIN sys.indexes i ON ic.object_id = i.object_id AND ic.index_id = i.index_id AND i.is_primary_key = 1 \
                 WHERE c.object_id = OBJECT_ID('{}') \
                 ORDER BY c.column_id",
                table
            ),
        };

        let command = match self.db_type {
            DatabaseType::Postgres => vec!["psql", "-U", "postgres", "-t", "-A", "-F", "|", "-c", &query],
            DatabaseType::MySQL => vec!["mysql", "-uroot", "-proot", "-N", "-B", "-e", &query],
            DatabaseType::SQLServer => vec![
                "/opt/mssql-tools18/bin/sqlcmd",
                "-S", "localhost",
                "-U", "sa",
                "-P", "YourStrong@Passw0rd",
                "-C",
                "-h", "-1",
                "-s", "|",
                "-W",
                "-Q", &query,
            ],
        };

        let output = self.collector.exec_query(&self.container_id, command).await?;

        let mut columns = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                columns.push(ColumnMetadata {
                    name: parts[0].trim().to_string(),
                    data_type: parts[1].trim().to_string(),
                    is_nullable: parts[2].trim().eq_ignore_ascii_case("yes") || parts[2].trim() == "1",
                    is_primary_key: parts[3].trim().eq_ignore_ascii_case("yes") || parts[3].trim().eq_ignore_ascii_case("pri"),
                });
            }
        }

        if columns.is_empty() {
            return Err(anyhow!("No columns found for table: {}", table));
        }

        Ok(columns)
    }

    /// Get approximate row count for a table
    async fn get_row_count(&self, table: &str) -> Result<usize> {
        let query = match self.db_type {
            DatabaseType::Postgres => format!(
                "SELECT reltuples::bigint FROM pg_class WHERE relname = '{}'",
                table
            ),
            DatabaseType::MySQL => format!(
                "SELECT TABLE_ROWS FROM information_schema.TABLES \
                 WHERE TABLE_NAME = '{}' AND TABLE_SCHEMA = DATABASE()",
                table
            ),
            DatabaseType::SQLServer => format!(
                "SELECT SUM(p.rows) FROM sys.partitions p \
                 JOIN sys.tables t ON p.object_id = t.object_id \
                 WHERE t.name = '{}' AND p.index_id IN (0, 1)",
                table
            ),
        };

        let command = match self.db_type {
            DatabaseType::Postgres => vec!["psql", "-U", "postgres", "-t", "-A", "-c", &query],
            DatabaseType::MySQL => vec!["mysql", "-uroot", "-proot", "-N", "-B", "-e", &query],
            DatabaseType::SQLServer => vec![
                "/opt/mssql-tools18/bin/sqlcmd",
                "-S", "localhost",
                "-U", "sa",
                "-P", "YourStrong@Passw0rd",
                "-C",
                "-h", "-1",
                "-W",
                "-Q", &query,
            ],
        };

        let output = self.collector.exec_query(&self.container_id, command).await?;

        let count_str = output.lines().next().unwrap_or("0").trim();
        let count = count_str.parse::<usize>().unwrap_or(1000);

        Ok(count.max(1)) // At least 1 to avoid division by zero
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_metadata() {
        let metadata = TableMetadata {
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
                    name: "email".to_string(),
                    data_type: "varchar".to_string(),
                    is_nullable: false,
                    is_primary_key: false,
                },
            ],
            primary_key: Some("id".to_string()),
            row_count_estimate: 1000,
        };

        assert_eq!(metadata.updatable_columns().len(), 2);
        assert!(metadata.primary_key_column().is_some());
        assert_eq!(metadata.primary_key_column().unwrap().name, "id");
    }

    #[test]
    fn test_updatable_columns() {
        let metadata = TableMetadata {
            name: "test".to_string(),
            columns: vec![
                ColumnMetadata {
                    name: "id".to_string(),
                    data_type: "int".to_string(),
                    is_nullable: false,
                    is_primary_key: true,
                },
                ColumnMetadata {
                    name: "value".to_string(),
                    data_type: "varchar".to_string(),
                    is_nullable: true,
                    is_primary_key: false,
                },
            ],
            primary_key: Some("id".to_string()),
            row_count_estimate: 100,
        };

        let updatable = metadata.updatable_columns();
        assert_eq!(updatable.len(), 1);
        assert_eq!(updatable[0].name, "value");
    }
}
