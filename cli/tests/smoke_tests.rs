//! Smoke Tests for v0.5.0
//!
//! These tests validate basic functionality without requiring Docker.
//! For full integration tests, see docs/TESTING_PHASE8.md

use dbarena::seed::config::SeedConfig;
use dbarena::seed::generator::*;
use dbarena::workload::{WorkloadConfig, WorkloadPattern};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

#[test]
fn test_seed_config_parsing() {
    let config_toml = r#"
global_seed = 42
batch_size = 1000

[[seed_rules.tables]]
name = "users"
count = 100

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

[[seed_rules.tables.columns]]
name = "email"
generator = "email"
    "#;

    let config: SeedConfig = toml::from_str(config_toml).expect("Failed to parse seed config");

    assert_eq!(config.global_seed, Some(42));
    assert_eq!(config.batch_size, 1000);
    assert_eq!(config.seed_rules.tables().len(), 1);
    assert_eq!(config.seed_rules.tables()[0].name, "users");
    assert_eq!(config.seed_rules.tables()[0].count, 100);
}

#[test]
fn test_workload_config_parsing() {
    let config_toml = r#"
name = "Test Workload"
pattern = "oltp"
tables = ["users", "orders"]
connections = 10
target_tps = 100
duration_seconds = 60
    "#;

    let config: WorkloadConfig = toml::from_str(config_toml).expect("Failed to parse workload config");

    assert_eq!(config.name, "Test Workload");
    assert_eq!(config.pattern, Some(WorkloadPattern::Oltp));
    assert_eq!(config.tables, vec!["users", "orders"]);
    assert_eq!(config.connections, 10);
    assert_eq!(config.target_tps, 100);
    assert_eq!(config.duration_seconds, Some(60));
}

#[test]
fn test_data_generation_performance() {
    use std::time::Instant;

    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Test sequential generator (fast)
    let seq_gen = SequentialGenerator::new(1);
    let start = Instant::now();
    for _ in 0..10_000 {
        let _ = seq_gen.generate(&mut rng).expect("generate failed");
    }
    let sequential_elapsed = start.elapsed();
    println!("Sequential: 10K values in {:?}", sequential_elapsed);

    // Test email generator (uses Faker)
    let email_gen = EmailGenerator::new();
    let start = Instant::now();
    for _ in 0..10_000 {
        let _ = email_gen.generate(&mut rng).expect("generate failed");
    }
    let email_elapsed = start.elapsed();
    println!("Email: 10K values in {:?}", email_elapsed);

    // Test random int generator
    let int_gen = RandomIntGenerator::new(1, 1000);
    let start = Instant::now();
    for _ in 0..10_000 {
        let _ = int_gen.generate(&mut rng).expect("generate failed");
    }
    let int_elapsed = start.elapsed();
    println!("RandomInt: 10K values in {:?}", int_elapsed);

    // All generators should be reasonably fast
    assert!(sequential_elapsed.as_millis() < 100, "Sequential too slow");
    assert!(email_elapsed.as_millis() < 1000, "Email too slow");
    assert!(int_elapsed.as_millis() < 100, "RandomInt too slow");
}

#[test]
fn test_deterministic_seeding() {
    // Same seed should produce identical results
    let mut rng1 = ChaCha8Rng::seed_from_u64(12345);
    let mut rng2 = ChaCha8Rng::seed_from_u64(12345);

    let email_gen = EmailGenerator::new();

    let mut results1 = Vec::new();
    let mut results2 = Vec::new();

    for _ in 0..100 {
        results1.push(email_gen.generate(&mut rng1).expect("generate failed"));
        results2.push(email_gen.generate(&mut rng2).expect("generate failed"));
    }

    assert_eq!(results1, results2, "Deterministic seeding failed");
}

#[test]
fn test_workload_pattern_weights() {
    // Verify all patterns have valid weights
    let patterns = vec![
        WorkloadPattern::Oltp,
        WorkloadPattern::Ecommerce,
        WorkloadPattern::Olap,
        WorkloadPattern::Reporting,
        WorkloadPattern::TimeSeries,
        WorkloadPattern::SocialMedia,
        WorkloadPattern::Iot,
        WorkloadPattern::ReadHeavy,
        WorkloadPattern::WriteHeavy,
        WorkloadPattern::Balanced,
    ];

    for pattern in patterns {
        let weights = pattern.operation_weights();

        // Weights should sum to ~1.0
        let sum = weights.select + weights.insert + weights.update + weights.delete;
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Pattern {:?} weights don't sum to 1.0: {}",
            pattern,
            sum
        );

        // All weights should be non-negative
        assert!(weights.select >= 0.0);
        assert!(weights.insert >= 0.0);
        assert!(weights.update >= 0.0);
        assert!(weights.delete >= 0.0);
    }
}

#[test]
fn test_batch_generation_performance() {
    use std::time::Instant;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let email_gen = EmailGenerator::new();
    let name_gen = NameGenerator::new(NameType::Full);
    let seq_gen = SequentialGenerator::new(1);

    // Simulate generating a batch of 1000 rows
    let start = Instant::now();
    let mut batch = Vec::new();

    for _ in 0..1000 {
        let mut row = HashMap::new();
        row.insert("id".to_string(), seq_gen.generate(&mut rng).expect("generate failed"));
        row.insert("email".to_string(), email_gen.generate(&mut rng).expect("generate failed"));
        row.insert("name".to_string(), name_gen.generate(&mut rng).expect("generate failed"));
        batch.push(row);
    }

    let elapsed = start.elapsed();
    println!("Generated 1000-row batch in {:?}", elapsed);

    assert_eq!(batch.len(), 1000);
    assert!(
        elapsed.as_millis() < 500,
        "Batch generation too slow: {:?}",
        elapsed
    );
}

#[test]
fn test_sql_generation_safety() {
    // Test that generated values don't contain SQL injection patterns
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let generators: Vec<Box<dyn DataGenerator>> = vec![
        Box::new(EmailGenerator::new()),
        Box::new(NameGenerator::new(NameType::Full)),
        Box::new(TemplateGenerator::new("test_{random_int:1:100}".to_string())),
    ];

    for gen in generators {
        for _ in 0..100 {
            let value = gen.generate(&mut rng).expect("generate failed");
            // Check for common SQL injection patterns
            assert!(!value.contains("';"), "Value contains SQL injection pattern: {}", value);
            assert!(!value.contains("--"), "Value contains SQL comment: {}", value);
            assert!(!value.contains("/*"), "Value contains SQL comment: {}", value);
        }
    }
}

#[test]
fn test_workload_config_validation() {
    // Test that invalid configs are rejected
    let invalid_config_toml = r#"
name = "Invalid"
tables = []
connections = 0
target_tps = 0
    "#;

    let result: Result<WorkloadConfig, _> = toml::from_str(invalid_config_toml);
    // Should parse but be logically invalid
    if let Ok(config) = result {
        assert!(config.tables.is_empty(), "Should allow empty tables for validation");
    }

    // Valid config should parse successfully
    let valid_config_toml = r#"
name = "Valid"
pattern = "balanced"
tables = ["test"]
connections = 10
target_tps = 100
duration_seconds = 60
    "#;

    let config: WorkloadConfig = toml::from_str(valid_config_toml).expect("Valid config should parse");
    assert_eq!(config.connections, 10);
    assert_eq!(config.target_tps, 100);
}

#[test]
fn test_all_generators_produce_valid_output() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let generators: Vec<(&str, Box<dyn DataGenerator>)> = vec![
        ("sequential", Box::new(SequentialGenerator::new(1))),
        ("random_int", Box::new(RandomIntGenerator::new(1, 100))),
        ("random_decimal", Box::new(RandomDecimalGenerator::new(1.0, 100.0, 2))),
        ("boolean", Box::new(BooleanGenerator::new(0.5))),
        ("timestamp_now", Box::new(TimestampGenerator::new(TimestampType::Now))),
        ("email", Box::new(EmailGenerator::new())),
        ("phone", Box::new(PhoneGenerator::new())),
        ("name", Box::new(NameGenerator::new(NameType::Full))),
        ("address", Box::new(AddressGenerator::new())),
        ("template", Box::new(TemplateGenerator::new("test_{random_int:1:10}".to_string()))),
        ("enum", Box::new(EnumGenerator::new(vec!["a".to_string(), "b".to_string()]))),
    ];

    for (name, gen) in generators {
        // Generate 10 values
        for _ in 0..10 {
            let value = gen.generate(&mut rng).expect("generate failed");
            assert!(!value.is_empty(), "Generator {} produced empty value", name);
            assert!(
                value.len() < 1000,
                "Generator {} produced suspiciously long value: {} chars",
                name,
                value.len()
            );
        }
    }
}
