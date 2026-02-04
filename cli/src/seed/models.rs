use std::collections::HashMap;
use std::time::Duration;

/// Statistics for a seeding operation
#[derive(Debug, Clone)]
pub struct SeedStats {
    pub table: String,
    pub rows_inserted: usize,
    pub duration: Duration,
    pub rows_per_second: f64,
}

impl SeedStats {
    pub fn new(table: String, rows_inserted: usize, duration: Duration) -> Self {
        let rows_per_second = if duration.as_secs_f64() > 0.0 {
            rows_inserted as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            table,
            rows_inserted,
            duration,
            rows_per_second,
        }
    }
}

/// Represents a single row of data
pub type Row = HashMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_stats_calculation() {
        let stats = SeedStats::new(
            "users".to_string(),
            1000,
            Duration::from_secs(10),
        );

        assert_eq!(stats.table, "users");
        assert_eq!(stats.rows_inserted, 1000);
        assert_eq!(stats.duration, Duration::from_secs(10));
        assert_eq!(stats.rows_per_second, 100.0);
    }

    #[test]
    fn test_seed_stats_zero_duration() {
        let stats = SeedStats::new(
            "users".to_string(),
            1000,
            Duration::from_secs(0),
        );

        assert_eq!(stats.rows_per_second, 0.0);
    }
}
