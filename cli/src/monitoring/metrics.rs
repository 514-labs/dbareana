use serde::{Deserialize, Serialize};

/// CPU usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU usage percentage (0.0 - 100.0+, can exceed 100% on multi-core systems)
    pub usage_percent: f64,
    /// Number of CPU cores
    pub num_cores: u64,
    /// Total CPU usage in nanoseconds (for delta calculation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_usage: Option<u64>,
    /// System CPU usage in nanoseconds (for delta calculation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_usage: Option<u64>,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Current memory usage in bytes
    pub usage: u64,
    /// Memory limit in bytes
    pub limit: u64,
    /// Memory usage percentage (0.0 - 100.0)
    pub percent: f64,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Receive rate in bytes/sec (calculated from deltas)
    pub rx_rate: f64,
    /// Transmit rate in bytes/sec (calculated from deltas)
    pub tx_rate: f64,
}

/// Block I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockIoMetrics {
    /// Bytes read from disk
    pub read_bytes: u64,
    /// Bytes written to disk
    pub write_bytes: u64,
    /// Read rate in bytes/sec (calculated from deltas)
    pub read_rate: f64,
    /// Write rate in bytes/sec (calculated from deltas)
    pub write_rate: f64,
}

/// Complete container metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetrics {
    /// Container ID
    pub container_id: String,
    /// Container name
    pub container_name: String,
    /// Timestamp of metrics collection (Unix timestamp in seconds)
    pub timestamp: i64,
    /// CPU metrics
    pub cpu: CpuMetrics,
    /// Memory metrics
    pub memory: MemoryMetrics,
    /// Network I/O metrics
    pub network: NetworkMetrics,
    /// Block I/O metrics
    pub block_io: BlockIoMetrics,
    /// Number of PIDs (processes/threads)
    pub pids: u64,
}

impl ContainerMetrics {
    /// Calculate rate deltas between two metrics snapshots
    pub fn calculate_rates(&mut self, previous: &ContainerMetrics) {
        let time_delta = (self.timestamp - previous.timestamp) as f64;
        if time_delta <= 0.0 {
            return;
        }

        // Recalculate CPU percentage using our own deltas
        if let (Some(current_cpu), Some(prev_cpu), Some(current_sys), Some(prev_sys)) = (
            self.cpu.total_usage,
            previous.cpu.total_usage,
            self.cpu.system_usage,
            previous.cpu.system_usage,
        ) {
            let cpu_delta = current_cpu.saturating_sub(prev_cpu) as f64;
            let system_delta = current_sys.saturating_sub(prev_sys) as f64;

            if system_delta > 0.0 && cpu_delta > 0.0 {
                // Docker's formula: (cpu_delta / system_delta) * num_cpus * 100.0
                // This gives percentage where 100% = 1 full core
                self.cpu.usage_percent = (cpu_delta / system_delta) * self.cpu.num_cores as f64 * 100.0;
            }
        }

        // Calculate network rates
        let rx_delta = self.network.rx_bytes.saturating_sub(previous.network.rx_bytes) as f64;
        let tx_delta = self.network.tx_bytes.saturating_sub(previous.network.tx_bytes) as f64;
        self.network.rx_rate = rx_delta / time_delta;
        self.network.tx_rate = tx_delta / time_delta;

        // Calculate block I/O rates
        let read_delta = self.block_io.read_bytes.saturating_sub(previous.block_io.read_bytes) as f64;
        let write_delta = self.block_io.write_bytes.saturating_sub(previous.block_io.write_bytes) as f64;
        self.block_io.read_rate = read_delta / time_delta;
        self.block_io.write_rate = write_delta / time_delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rates() {
        let previous = ContainerMetrics {
            container_id: "test".to_string(),
            container_name: "test-container".to_string(),
            timestamp: 1000,
            cpu: CpuMetrics {
                usage_percent: 50.0,
                num_cores: 4,
                total_usage: Some(1000000000),
                system_usage: Some(10000000000),
            },
            memory: MemoryMetrics {
                usage: 1024 * 1024 * 512,
                limit: 1024 * 1024 * 1024 * 2,
                percent: 25.0,
            },
            network: NetworkMetrics {
                rx_bytes: 1000,
                tx_bytes: 500,
                rx_rate: 0.0,
                tx_rate: 0.0,
            },
            block_io: BlockIoMetrics {
                read_bytes: 2000,
                write_bytes: 1000,
                read_rate: 0.0,
                write_rate: 0.0,
            },
            pids: 10,
        };

        let mut current = ContainerMetrics {
            container_id: "test".to_string(),
            container_name: "test-container".to_string(),
            timestamp: 1002, // 2 seconds later
            cpu: CpuMetrics {
                usage_percent: 60.0,
                num_cores: 4,
                total_usage: Some(3000000000), // +2B ns = 2 seconds of CPU time
                system_usage: Some(18000000000), // +8B ns = 8 seconds of system time (4 cores * 2s)
            },
            memory: MemoryMetrics {
                usage: 1024 * 1024 * 600,
                limit: 1024 * 1024 * 1024 * 2,
                percent: 30.0,
            },
            network: NetworkMetrics {
                rx_bytes: 3000, // +2000 bytes
                tx_bytes: 1500, // +1000 bytes
                rx_rate: 0.0,
                tx_rate: 0.0,
            },
            block_io: BlockIoMetrics {
                read_bytes: 6000, // +4000 bytes
                write_bytes: 3000, // +2000 bytes
                read_rate: 0.0,
                write_rate: 0.0,
            },
            pids: 12,
        };

        current.calculate_rates(&previous);

        // CPU percentage should be recalculated
        // cpu_delta = 3B - 1B = 2B ns (2 seconds)
        // system_delta = 18B - 10B = 8B ns (8 seconds = 4 cores * 2s)
        // cpu_percent = (2B / 8B) * 4 * 100 = 100%
        assert_eq!(current.cpu.usage_percent, 100.0);

        // Network rates: delta / time
        assert_eq!(current.network.rx_rate, 1000.0); // 2000 / 2
        assert_eq!(current.network.tx_rate, 500.0); // 1000 / 2

        // Block I/O rates
        assert_eq!(current.block_io.read_rate, 2000.0); // 4000 / 2
        assert_eq!(current.block_io.write_rate, 1000.0); // 2000 / 2
    }
}
