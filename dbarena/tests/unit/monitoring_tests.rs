use dbarena::monitoring::{
    ContainerMetrics, CpuMetrics, MemoryMetrics, NetworkMetrics, BlockIoMetrics,
    format_bytes, format_rate,
};

#[test]
fn test_cpu_metrics_creation() {
    let cpu = CpuMetrics {
        usage_percent: 45.5,
        num_cores: 4,
    };

    assert_eq!(cpu.usage_percent, 45.5);
    assert_eq!(cpu.num_cores, 4);
}

#[test]
fn test_memory_metrics_creation() {
    let memory = MemoryMetrics {
        usage: 1024 * 1024 * 512, // 512 MB
        limit: 1024 * 1024 * 1024 * 2, // 2 GB
        percent: 25.0,
    };

    assert_eq!(memory.usage, 536870912);
    assert_eq!(memory.limit, 2147483648);
    assert_eq!(memory.percent, 25.0);
}

#[test]
fn test_metrics_rate_calculation() {
    let previous = ContainerMetrics {
        container_id: "test-123".to_string(),
        container_name: "test-container".to_string(),
        timestamp: 1000,
        cpu: CpuMetrics {
            usage_percent: 50.0,
            num_cores: 2,
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
    };

    let mut current = ContainerMetrics {
        container_id: "test-123".to_string(),
        container_name: "test-container".to_string(),
        timestamp: 1005, // 5 seconds later
        cpu: CpuMetrics {
            usage_percent: 60.0,
            num_cores: 2,
        },
        memory: MemoryMetrics {
            usage: 1024 * 1024 * 600,
            limit: 1024 * 1024 * 1024 * 2,
            percent: 30.0,
        },
        network: NetworkMetrics {
            rx_bytes: 6000, // +5000 bytes
            tx_bytes: 3000, // +2500 bytes
            rx_rate: 0.0,
            tx_rate: 0.0,
        },
        block_io: BlockIoMetrics {
            read_bytes: 12000, // +10000 bytes
            write_bytes: 6000,  // +5000 bytes
            read_rate: 0.0,
            write_rate: 0.0,
        },
    };

    current.calculate_rates(&previous);

    // Network rates: delta / time
    assert_eq!(current.network.rx_rate, 1000.0); // 5000 / 5
    assert_eq!(current.network.tx_rate, 500.0);  // 2500 / 5

    // Block I/O rates
    assert_eq!(current.block_io.read_rate, 2000.0);  // 10000 / 5
    assert_eq!(current.block_io.write_rate, 1000.0); // 5000 / 5
}

#[test]
fn test_rate_calculation_with_zero_time_delta() {
    let previous = ContainerMetrics {
        container_id: "test".to_string(),
        container_name: "test".to_string(),
        timestamp: 1000,
        cpu: CpuMetrics {
            usage_percent: 50.0,
            num_cores: 2,
        },
        memory: MemoryMetrics {
            usage: 1024,
            limit: 2048,
            percent: 50.0,
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
    };

    let mut current = previous.clone();
    current.timestamp = 1000; // Same timestamp

    current.calculate_rates(&previous);

    // Should not crash and rates should remain 0
    assert_eq!(current.network.rx_rate, 0.0);
    assert_eq!(current.network.tx_rate, 0.0);
    assert_eq!(current.block_io.read_rate, 0.0);
    assert_eq!(current.block_io.write_rate, 0.0);
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(format_bytes(1536 * 1024 * 1024), "1.5 GB");
}

#[test]
fn test_format_rate() {
    assert_eq!(format_rate(0.0), "0 B/s");
    assert_eq!(format_rate(512.0), "512 B/s");
    assert_eq!(format_rate(1024.0), "1.0 KB/s");
    assert_eq!(format_rate(1024.0 * 1024.0), "1.0 MB/s");
    assert_eq!(format_rate(1024.0 * 1024.0 * 1024.0), "1.0 GB/s");
}

#[test]
fn test_metrics_serialization() {
    let metrics = ContainerMetrics {
        container_id: "abc123".to_string(),
        container_name: "my-container".to_string(),
        timestamp: 1234567890,
        cpu: CpuMetrics {
            usage_percent: 45.5,
            num_cores: 4,
        },
        memory: MemoryMetrics {
            usage: 1024 * 1024 * 512,
            limit: 1024 * 1024 * 1024 * 2,
            percent: 25.0,
        },
        network: NetworkMetrics {
            rx_bytes: 1000,
            tx_bytes: 500,
            rx_rate: 100.0,
            tx_rate: 50.0,
        },
        block_io: BlockIoMetrics {
            read_bytes: 2000,
            write_bytes: 1000,
            read_rate: 200.0,
            write_rate: 100.0,
        },
    };

    // Test JSON serialization
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("abc123"));
    assert!(json.contains("my-container"));
    assert!(json.contains("45.5"));

    // Test deserialization
    let deserialized: ContainerMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.container_id, metrics.container_id);
    assert_eq!(deserialized.container_name, metrics.container_name);
    assert_eq!(deserialized.cpu.usage_percent, metrics.cpu.usage_percent);
}

#[test]
fn test_network_counter_overflow_handling() {
    // Test when counters wrap around (shouldn't happen often but good to handle)
    let previous = ContainerMetrics {
        container_id: "test".to_string(),
        container_name: "test".to_string(),
        timestamp: 1000,
        cpu: CpuMetrics {
            usage_percent: 50.0,
            num_cores: 2,
        },
        memory: MemoryMetrics {
            usage: 1024,
            limit: 2048,
            percent: 50.0,
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
    };

    let mut current = ContainerMetrics {
        container_id: "test".to_string(),
        container_name: "test".to_string(),
        timestamp: 1002,
        cpu: CpuMetrics {
            usage_percent: 50.0,
            num_cores: 2,
        },
        memory: MemoryMetrics {
            usage: 1024,
            limit: 2048,
            percent: 50.0,
        },
        network: NetworkMetrics {
            rx_bytes: 500, // Counter reset/wrapped
            tx_bytes: 200, // Counter reset/wrapped
            rx_rate: 0.0,
            tx_rate: 0.0,
        },
        block_io: BlockIoMetrics {
            read_bytes: 1000, // Counter reset/wrapped
            write_bytes: 500,  // Counter reset/wrapped
            read_rate: 0.0,
            write_rate: 0.0,
        },
    };

    current.calculate_rates(&previous);

    // Should handle underflow gracefully (saturating_sub returns 0)
    assert_eq!(current.network.rx_rate, 0.0);
    assert_eq!(current.network.tx_rate, 0.0);
    assert_eq!(current.block_io.read_rate, 0.0);
    assert_eq!(current.block_io.write_rate, 0.0);
}

#[test]
fn test_large_values() {
    // Test with very large byte values (TB range)
    let large_bytes = 5 * 1024u64.pow(4); // 5 TB
    let formatted = format_bytes(large_bytes);
    assert!(formatted.contains("TB"));
    assert!(formatted.contains("5.0"));
}

#[test]
fn test_memory_percent_calculation() {
    let memory = MemoryMetrics {
        usage: 1024 * 1024 * 1024, // 1 GB
        limit: 1024 * 1024 * 1024 * 4, // 4 GB
        percent: 25.0,
    };

    // Verify percent is correct
    let calculated_percent = (memory.usage as f64 / memory.limit as f64) * 100.0;
    assert!((calculated_percent - memory.percent).abs() < 0.01);
}
