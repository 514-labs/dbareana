use hdrhistogram::Histogram;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Metric sample from a worker
#[derive(Debug, Clone)]
pub struct MetricSample {
    pub worker_id: usize,
    pub operation_type: String,
    pub success: bool,
    pub latency_us: u64,
    pub error: Option<String>,
}

/// Workload statistics with histogram-based latency tracking
pub struct WorkloadStats {
    pub total_transactions: AtomicU64,
    pub successful_transactions: AtomicU64,
    pub failed_transactions: AtomicU64,
    pub latency_histogram: Mutex<Histogram<u64>>,
    pub errors: Mutex<HashMap<String, u64>>,
    pub operation_counts: Mutex<HashMap<String, u64>>,
    pub start_time: Instant,
}

impl WorkloadStats {
    pub fn new() -> Self {
        Self {
            total_transactions: AtomicU64::new(0),
            successful_transactions: AtomicU64::new(0),
            failed_transactions: AtomicU64::new(0),
            latency_histogram: Mutex::new(
                Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).unwrap(),
            ),
            errors: Mutex::new(HashMap::new()),
            operation_counts: Mutex::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    /// Record a successful transaction
    pub fn record_success(&self, operation_type: &str, latency: Duration) {
        self.total_transactions.fetch_add(1, Ordering::SeqCst);
        self.successful_transactions.fetch_add(1, Ordering::SeqCst);

        let latency_us = latency.as_micros() as u64;
        if let Ok(mut hist) = self.latency_histogram.lock() {
            let _ = hist.record(latency_us);
        }

        if let Ok(mut counts) = self.operation_counts.lock() {
            *counts.entry(operation_type.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a failed transaction
    pub fn record_failure(&self, operation_type: &str, error: &str) {
        self.total_transactions.fetch_add(1, Ordering::SeqCst);
        self.failed_transactions.fetch_add(1, Ordering::SeqCst);

        if let Ok(mut errors) = self.errors.lock() {
            *errors.entry(error.to_string()).or_insert(0) += 1;
        }

        if let Ok(mut counts) = self.operation_counts.lock() {
            *counts.entry(operation_type.to_string()).or_insert(0) += 1;
        }
    }

    /// Record a metric sample from a worker
    pub fn record_sample(&self, sample: MetricSample) {
        if sample.success {
            self.record_success(&sample.operation_type, Duration::from_micros(sample.latency_us));
        } else if let Some(error) = &sample.error {
            self.record_failure(&sample.operation_type, error);
        }
    }

    /// Get total transaction count
    pub fn total(&self) -> u64 {
        self.total_transactions.load(Ordering::SeqCst)
    }

    /// Get successful transaction count
    pub fn success_count(&self) -> u64 {
        self.successful_transactions.load(Ordering::SeqCst)
    }

    /// Get failed transaction count
    pub fn failure_count(&self) -> u64 {
        self.failed_transactions.load(Ordering::SeqCst)
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 100.0;
        }
        (self.success_count() as f64 / total as f64) * 100.0
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get transactions per second
    pub fn tps(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total() as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get latency percentile in microseconds
    pub fn latency_percentile(&self, percentile: f64) -> Option<u64> {
        if let Ok(hist) = self.latency_histogram.lock() {
            Some(hist.value_at_quantile(percentile))
        } else {
            None
        }
    }

    /// Get mean latency in microseconds
    pub fn mean_latency(&self) -> Option<f64> {
        if let Ok(hist) = self.latency_histogram.lock() {
            Some(hist.mean())
        } else {
            None
        }
    }

    /// Get max latency in microseconds
    pub fn max_latency(&self) -> Option<u64> {
        if let Ok(hist) = self.latency_histogram.lock() {
            Some(hist.max())
        } else {
            None
        }
    }

    /// Get min latency in microseconds
    pub fn min_latency(&self) -> Option<u64> {
        if let Ok(hist) = self.latency_histogram.lock() {
            Some(hist.min())
        } else {
            None
        }
    }

    /// Get operation counts
    pub fn operation_counts(&self) -> HashMap<String, u64> {
        if let Ok(counts) = self.operation_counts.lock() {
            counts.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get error counts
    pub fn error_counts(&self) -> HashMap<String, u64> {
        if let Ok(errors) = self.errors.lock() {
            errors.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get a summary snapshot
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            total: self.total(),
            success: self.success_count(),
            failed: self.failure_count(),
            success_rate: self.success_rate(),
            elapsed: self.elapsed(),
            tps: self.tps(),
            p50: self.latency_percentile(0.50),
            p95: self.latency_percentile(0.95),
            p99: self.latency_percentile(0.99),
            mean: self.mean_latency(),
            min: self.min_latency(),
            max: self.max_latency(),
            operation_counts: self.operation_counts(),
            error_counts: self.error_counts(),
        }
    }
}

impl Default for WorkloadStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable snapshot of stats for reporting
#[derive(Debug, Clone)]
pub struct StatsSnapshot {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub success_rate: f64,
    pub elapsed: Duration,
    pub tps: f64,
    pub p50: Option<u64>,
    pub p95: Option<u64>,
    pub p99: Option<u64>,
    pub mean: Option<f64>,
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub operation_counts: HashMap<String, u64>,
    pub error_counts: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_success() {
        let stats = WorkloadStats::new();

        stats.record_success("select", Duration::from_millis(10));
        stats.record_success("insert", Duration::from_millis(20));

        assert_eq!(stats.total(), 2);
        assert_eq!(stats.success_count(), 2);
        assert_eq!(stats.failure_count(), 0);
        assert_eq!(stats.success_rate(), 100.0);
    }

    #[test]
    fn test_record_failure() {
        let stats = WorkloadStats::new();

        stats.record_success("select", Duration::from_millis(10));
        stats.record_failure("insert", "connection timeout");

        assert_eq!(stats.total(), 2);
        assert_eq!(stats.success_count(), 1);
        assert_eq!(stats.failure_count(), 1);
        assert_eq!(stats.success_rate(), 50.0);

        let errors = stats.error_counts();
        assert_eq!(errors.get("connection timeout"), Some(&1));
    }

    #[test]
    fn test_latency_percentiles() {
        let stats = WorkloadStats::new();

        // Record some latencies
        for i in 1..=100 {
            stats.record_success("select", Duration::from_millis(i));
        }

        let p50 = stats.latency_percentile(0.50).unwrap();
        let p95 = stats.latency_percentile(0.95).unwrap();
        let p99 = stats.latency_percentile(0.99).unwrap();

        // Verify percentiles are in expected ranges
        assert!(p50 > 40_000 && p50 < 60_000); // ~50ms
        assert!(p95 > 90_000 && p95 < 100_000); // ~95ms
        assert!(p99 > 95_000); // ~99ms
    }

    #[test]
    fn test_operation_counts() {
        let stats = WorkloadStats::new();

        stats.record_success("select", Duration::from_millis(10));
        stats.record_success("select", Duration::from_millis(10));
        stats.record_success("insert", Duration::from_millis(20));

        let counts = stats.operation_counts();
        assert_eq!(counts.get("select"), Some(&2));
        assert_eq!(counts.get("insert"), Some(&1));
    }

    #[test]
    fn test_tps_calculation() {
        let stats = WorkloadStats::new();

        // Record transactions
        for _ in 0..100 {
            stats.record_success("select", Duration::from_millis(1));
        }

        // TPS should be calculated based on elapsed time
        let tps = stats.tps();
        assert!(tps > 0.0);
    }

    #[test]
    fn test_snapshot() {
        let stats = WorkloadStats::new();

        stats.record_success("select", Duration::from_millis(10));
        stats.record_success("insert", Duration::from_millis(20));
        stats.record_failure("update", "deadlock");

        let snapshot = stats.snapshot();

        assert_eq!(snapshot.total, 3);
        assert_eq!(snapshot.success, 2);
        assert_eq!(snapshot.failed, 1);
        assert!((snapshot.success_rate - 66.67).abs() < 0.1);
        assert!(snapshot.p50.is_some());
        assert_eq!(snapshot.operation_counts.get("select"), Some(&1));
    }

    #[test]
    fn test_record_sample() {
        let stats = WorkloadStats::new();

        let sample = MetricSample {
            worker_id: 1,
            operation_type: "select".to_string(),
            success: true,
            latency_us: 5000,
            error: None,
        };

        stats.record_sample(sample);

        assert_eq!(stats.success_count(), 1);
        assert_eq!(stats.total(), 1);
    }

    #[test]
    fn test_mean_latency() {
        let stats = WorkloadStats::new();

        stats.record_success("select", Duration::from_millis(10));
        stats.record_success("select", Duration::from_millis(20));
        stats.record_success("select", Duration::from_millis(30));

        let mean = stats.mean_latency().unwrap();
        // Mean should be around 20ms (20,000 microseconds)
        assert!(mean > 15_000.0 && mean < 25_000.0);
    }
}
