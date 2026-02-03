use anyhow::{anyhow, Result};
use bollard::Docker;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::container::DatabaseType;
use crate::database_metrics::collector::DockerDatabaseMetricsCollector;

/// Resolves foreign key values by querying and caching existing data
pub struct ForeignKeyResolver {
    cache: Arc<Mutex<HashMap<String, Vec<String>>>>, // "table.column" -> IDs
    collector: DockerDatabaseMetricsCollector,
    container_id: String,
    db_type: DatabaseType,
}

impl ForeignKeyResolver {
    pub fn new(
        docker_client: Arc<Docker>,
        container_id: String,
        db_type: DatabaseType,
    ) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            collector: DockerDatabaseMetricsCollector::new(docker_client),
            container_id,
            db_type,
        }
    }

    /// Load all IDs from a referenced table and cache them
    pub async fn load_ids(&self, table: &str, column: &str) -> Result<()> {
        let key = format!("{}.{}", table, column);

        // Check if already cached
        {
            let cache = self.cache.lock().await;
            if cache.contains_key(&key) {
                return Ok(());
            }
        }

        // Query database for IDs
        let ids = self.fetch_ids(table, column).await?;

        if ids.is_empty() {
            return Err(anyhow!(
                "No rows found in table '{}' for foreign key reference",
                table
            ));
        }

        // Cache the IDs
        let mut cache = self.cache.lock().await;
        cache.insert(key, ids);

        Ok(())
    }

    /// Get a random ID from the cache
    pub async fn random_id(
        &self,
        table: &str,
        column: &str,
        rng: &mut ChaCha8Rng,
    ) -> Result<String> {
        let key = format!("{}.{}", table, column);

        let cache = self.cache.lock().await;
        let ids = cache
            .get(&key)
            .ok_or_else(|| anyhow!("Foreign key cache not loaded for {}", key))?;

        if ids.is_empty() {
            return Err(anyhow!("No IDs available for foreign key reference: {}", key));
        }

        let idx = rng.gen_range(0..ids.len());
        Ok(ids[idx].clone())
    }

    /// Fetch IDs from database
    async fn fetch_ids(&self, table: &str, column: &str) -> Result<Vec<String>> {
        let query = self.build_select_query(table, column);

        let command = match self.db_type {
            DatabaseType::Postgres => {
                vec!["psql", "-U", "postgres", "-t", "-A", "-c", &query]
            }
            DatabaseType::MySQL => {
                vec!["mysql", "-uroot", "-proot", "-N", "-B", "-e", &query]
            }
            DatabaseType::SQLServer => {
                vec![
                    "/opt/mssql-tools18/bin/sqlcmd",
                    "-S",
                    "localhost",
                    "-U",
                    "sa",
                    "-P",
                    "YourStrong@Passw0rd",
                    "-C",
                    "-h",
                    "-1",
                    "-W",
                    "-Q",
                    &query,
                ]
            }
        };

        let output = self
            .collector
            .exec_query(&self.container_id, command)
            .await?;

        // Parse output into IDs
        let ids: Vec<String> = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();

        Ok(ids)
    }

    /// Build SELECT query for fetching IDs
    fn build_select_query(&self, table: &str, column: &str) -> String {
        match self.db_type {
            DatabaseType::Postgres => {
                format!("SELECT \"{}\" FROM \"{}\"", column, table)
            }
            DatabaseType::MySQL => {
                format!("SELECT `{}` FROM `{}`", column, table)
            }
            DatabaseType::SQLServer => {
                format!("SELECT [{}] FROM [{}]", column, table)
            }
        }
    }

    /// Get the number of cached IDs for a table.column
    pub async fn cached_count(&self, table: &str, column: &str) -> usize {
        let key = format!("{}.{}", table, column);
        let cache = self.cache.lock().await;
        cache.get(&key).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[tokio::test]
    async fn test_cache_operations() {
        let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());
        let resolver = ForeignKeyResolver::new(
            docker,
            "test".to_string(),
            DatabaseType::Postgres,
        );

        // Manually populate cache for testing
        {
            let mut cache = resolver.cache.lock().await;
            cache.insert(
                "users.id".to_string(),
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
            );
        }

        // Test cached_count
        let count = resolver.cached_count("users", "id").await;
        assert_eq!(count, 3);

        // Test random_id
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let id = resolver.random_id("users", "id", &mut rng).await.unwrap();
        assert!(id == "1" || id == "2" || id == "3");
    }

    #[tokio::test]
    async fn test_random_id_not_loaded() {
        let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());
        let resolver = ForeignKeyResolver::new(
            docker,
            "test".to_string(),
            DatabaseType::Postgres,
        );

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let result = resolver.random_id("users", "id", &mut rng).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cache not loaded"));
    }

    #[test]
    fn test_build_select_query() {
        let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());

        let pg_resolver = ForeignKeyResolver::new(
            docker.clone(),
            "test".to_string(),
            DatabaseType::Postgres,
        );
        assert_eq!(
            pg_resolver.build_select_query("users", "id"),
            "SELECT \"id\" FROM \"users\""
        );

        let mysql_resolver = ForeignKeyResolver::new(
            docker.clone(),
            "test".to_string(),
            DatabaseType::MySQL,
        );
        assert_eq!(
            mysql_resolver.build_select_query("users", "id"),
            "SELECT `id` FROM `users`"
        );

        let mssql_resolver = ForeignKeyResolver::new(
            docker,
            "test".to_string(),
            DatabaseType::SQLServer,
        );
        assert_eq!(
            mssql_resolver.build_select_query("users", "id"),
            "SELECT [id] FROM [users]"
        );
    }
}
