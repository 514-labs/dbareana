use anyhow::{anyhow, Result};
use bollard::Docker;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::container::DatabaseType;
use crate::database_metrics::collector::DockerDatabaseMetricsCollector;
use crate::seed::config::SeedRule;
use crate::seed::dependency::DependencyResolver;
use crate::seed::foreign_key::ForeignKeyResolver;
use crate::seed::generator::{create_generator, DataGenerator, ForeignKeyInfo};
use crate::seed::models::{Row, SeedStats};
use crate::seed::sql_builder::build_batch_insert;

/// Main seeding engine
pub struct SeedingEngine {
    container_id: String,
    db_type: DatabaseType,
    docker_client: Arc<Docker>,
    rng: ChaCha8Rng,
    seed: u64,
    batch_size: usize,
    collector: DockerDatabaseMetricsCollector,
    fk_resolver: Arc<ForeignKeyResolver>,
}

impl SeedingEngine {
    pub fn new(
        container_id: String,
        db_type: DatabaseType,
        docker_client: Arc<Docker>,
        seed: u64,
        batch_size: usize,
    ) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        let collector = DockerDatabaseMetricsCollector::new(docker_client.clone());
        let fk_resolver = Arc::new(ForeignKeyResolver::new(
            docker_client.clone(),
            container_id.clone(),
            db_type,
        ));

        Self {
            container_id,
            db_type,
            docker_client,
            rng,
            seed,
            batch_size,
            collector,
            fk_resolver,
        }
    }

    /// Seed multiple tables with dependency resolution and parallel execution
    pub async fn seed_all(&mut self, rules: &[SeedRule]) -> Result<Vec<SeedStats>> {
        // Build dependency resolver
        let mut dep_resolver = DependencyResolver::new();

        // Analyze foreign key dependencies
        for rule in rules {
            dep_resolver.add_table(rule.name.clone());

            for col_rule in &rule.columns {
                if col_rule.generator == "foreign_key" {
                    if let Ok(fk_info) = ForeignKeyInfo::from_options(&col_rule.options) {
                        dep_resolver.add_dependency(rule.name.clone(), fk_info.table.clone());
                    }
                }
            }
        }

        // Resolve seeding order
        let table_names: Vec<String> = rules.iter().map(|r| r.name.clone()).collect();
        let levels = dep_resolver.resolve_order(&table_names)?;

        println!("Seeding {} tables in {} levels", rules.len(), levels.len());

        let mut all_stats = Vec::new();

        // Seed each level (tables in same level can be seeded in parallel)
        for (level_idx, level_tables) in levels.iter().enumerate() {
            println!(
                "\nLevel {}: Seeding {} table(s) in parallel: {}",
                level_idx + 1,
                level_tables.len(),
                level_tables.join(", ")
            );

            // Load foreign key caches for this level
            for table_name in level_tables {
                let rule = rules.iter().find(|r| &r.name == table_name).unwrap();
                self.load_foreign_keys(rule).await?;
            }

            // Seed tables in parallel
            let mut futures = Vec::new();

            for table_name in level_tables {
                let rule = rules
                    .iter()
                    .find(|r| &r.name == table_name)
                    .unwrap()
                    .clone();

                // Create a new engine instance for parallel execution
                let mut engine = Self::new(
                    self.container_id.clone(),
                    self.db_type,
                    self.docker_client.clone(),
                    self.seed,
                    self.batch_size,
                );

                // Share the FK resolver
                engine.fk_resolver = self.fk_resolver.clone();

                futures.push(async move { engine.seed_table(&rule).await });
            }

            // Wait for all tables in this level to complete
            let level_stats = join_all(futures).await;

            for stat in level_stats {
                all_stats.push(stat?);
            }
        }

        Ok(all_stats)
    }

    /// Load foreign key caches for a table's dependencies
    async fn load_foreign_keys(&self, rule: &SeedRule) -> Result<()> {
        for col_rule in &rule.columns {
            if col_rule.generator == "foreign_key" {
                if let Ok(fk_info) = ForeignKeyInfo::from_options(&col_rule.options) {
                    self.fk_resolver
                        .load_ids(&fk_info.table, &fk_info.column)
                        .await?;
                }
            }
        }
        Ok(())
    }

    /// Seed a single table with generated data
    pub async fn seed_table(&mut self, rule: &SeedRule) -> Result<SeedStats> {
        let start = Instant::now();

        // Create progress bar
        let pb = ProgressBar::new(rule.count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} rows ({per_sec})")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(format!("Seeding table: {}", rule.name));

        // Build generators for each column
        let mut generators: HashMap<String, Box<dyn DataGenerator>> = HashMap::new();
        let mut fk_columns: HashMap<String, ForeignKeyInfo> = HashMap::new();

        for col_rule in &rule.columns {
            if col_rule.generator == "foreign_key" {
                // Handle FK separately since it needs async
                let fk_info = ForeignKeyInfo::from_options(&col_rule.options)?;
                fk_columns.insert(col_rule.name.clone(), fk_info);
            } else {
                let generator = create_generator(&col_rule.generator, &col_rule.options)?;
                generators.insert(col_rule.name.clone(), generator);
            }
        }

        let column_names: Vec<String> = rule.columns.iter().map(|c| c.name.clone()).collect();

        // Generate and insert data in batches
        let mut total_inserted = 0;
        let mut remaining = rule.count;

        while remaining > 0 {
            let batch_count = remaining.min(self.batch_size);

            // Generate batch
            let batch = self
                .generate_batch_with_fk(&column_names, &generators, &fk_columns, batch_count)
                .await?;

            // Insert batch
            self.insert_batch(&rule.name, &column_names, &batch).await?;

            total_inserted += batch_count;
            remaining -= batch_count;
            pb.set_position(total_inserted as u64);
        }

        pb.finish_with_message(format!("Completed seeding table: {}", rule.name));

        let duration = start.elapsed();
        Ok(SeedStats::new(rule.name.clone(), total_inserted, duration))
    }

    /// Generate a batch of rows with FK support
    async fn generate_batch_with_fk(
        &mut self,
        columns: &[String],
        generators: &HashMap<String, Box<dyn DataGenerator>>,
        fk_columns: &HashMap<String, ForeignKeyInfo>,
        count: usize,
    ) -> Result<Vec<Row>> {
        let mut rows = Vec::with_capacity(count);

        for _ in 0..count {
            let mut row = Row::new();

            for col_name in columns {
                let value = if let Some(fk_info) = fk_columns.get(col_name) {
                    // Generate FK value
                    self.fk_resolver
                        .random_id(&fk_info.table, &fk_info.column, &mut self.rng)
                        .await?
                } else {
                    // Generate regular value
                    let generator = generators
                        .get(col_name)
                        .ok_or_else(|| anyhow!("Generator not found for column: {}", col_name))?;
                    generator.generate(&mut self.rng)?
                };

                row.insert(col_name.clone(), value);
            }

            rows.push(row);
        }

        Ok(rows)
    }

    /// Insert a batch of rows into the database
    async fn insert_batch(
        &self,
        table: &str,
        columns: &[String],
        rows: &[Row],
    ) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Build INSERT statement
        let sql = build_batch_insert(self.db_type, table, columns, rows)?;

        // Execute SQL
        self.execute_sql(&sql).await?;

        Ok(())
    }

    /// Execute SQL via Docker exec
    async fn execute_sql(&self, sql: &str) -> Result<()> {
        let command = match self.db_type {
            DatabaseType::Postgres => {
                vec![
                    "psql",
                    "-U",
                    "postgres",
                    "-c",
                    sql,
                ]
            }
            DatabaseType::MySQL => {
                vec![
                    "mysql",
                    "-uroot",
                    "-proot",
                    "-e",
                    sql,
                ]
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
                    "-Q",
                    sql,
                ]
            }
        };

        let output = self
            .collector
            .exec_query(&self.container_id, command)
            .await?;

        // Check for errors in output
        if output.to_lowercase().contains("error") {
            return Err(anyhow!("SQL execution error: {}", output));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::generator::SequentialGenerator;

    #[tokio::test]
    async fn test_generate_batch_with_fk() {
        let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());
        let mut engine = SeedingEngine::new(
            "test".to_string(),
            DatabaseType::Postgres,
            docker,
            42,
            1000,
        );

        let columns = vec!["id".to_string(), "name".to_string()];

        let mut generators: HashMap<String, Box<dyn DataGenerator>> = HashMap::new();
        generators.insert("id".to_string(), Box::new(SequentialGenerator::new(1)));
        generators.insert("name".to_string(), Box::new(SequentialGenerator::new(100)));

        let fk_columns = HashMap::new(); // No FK columns in this test

        let batch = engine
            .generate_batch_with_fk(&columns, &generators, &fk_columns, 3)
            .await
            .unwrap();

        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].get("id").unwrap(), "1");
        assert_eq!(batch[1].get("id").unwrap(), "2");
        assert_eq!(batch[2].get("id").unwrap(), "3");
    }

    #[tokio::test]
    async fn test_deterministic_generation() {
        use crate::seed::generator::RandomIntGenerator;

        let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());

        let mut engine1 = SeedingEngine::new(
            "test".to_string(),
            DatabaseType::Postgres,
            docker.clone(),
            42,
            1000,
        );

        let mut engine2 = SeedingEngine::new(
            "test".to_string(),
            DatabaseType::Postgres,
            docker,
            42,
            1000,
        );

        let columns = vec!["value".to_string()];
        let mut generators1: HashMap<String, Box<dyn DataGenerator>> = HashMap::new();
        generators1.insert("value".to_string(), Box::new(RandomIntGenerator::new(1, 100)));

        let mut generators2: HashMap<String, Box<dyn DataGenerator>> = HashMap::new();
        generators2.insert("value".to_string(), Box::new(RandomIntGenerator::new(1, 100)));

        let fk_columns = HashMap::new();

        let batch1 = engine1
            .generate_batch_with_fk(&columns, &generators1, &fk_columns, 5)
            .await
            .unwrap();
        let batch2 = engine2
            .generate_batch_with_fk(&columns, &generators2, &fk_columns, 5)
            .await
            .unwrap();

        // Same seed should produce identical random values
        for i in 0..5 {
            assert_eq!(
                batch1[i].get("value").unwrap(),
                batch2[i].get("value").unwrap(),
                "Row {} should match with same seed",
                i
            );
        }
    }
}
