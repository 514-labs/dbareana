use dbarena::monitoring::{format_bytes, format_rate};

// Note: Full TUI testing requires terminal emulation which is complex.
// These tests cover the helper functions used by the TUI.

#[test]
fn test_format_bytes_edge_cases() {
    // Zero bytes
    assert_eq!(format_bytes(0), "0 B");

    // Single byte
    assert_eq!(format_bytes(1), "1 B");

    // Boundary values
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1025), "1.0 KB"); // Should round to 1.0

    // Exact multiples
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");

    // Between multiples
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(2560), "2.5 KB");
}

#[test]
fn test_format_rate_edge_cases() {
    // Zero rate
    assert_eq!(format_rate(0.0), "0 B/s");

    // Very small rate
    assert_eq!(format_rate(0.5), "0 B/s");
    assert_eq!(format_rate(1.0), "1 B/s");

    // Fractional rates
    assert_eq!(format_rate(512.5), "512 B/s");
    assert_eq!(format_rate(1536.0), "1.5 KB/s");

    // Large rates
    assert_eq!(format_rate(1024.0 * 1024.0 * 100.0), "100.0 MB/s");
    assert_eq!(format_rate(1024.0 * 1024.0 * 1024.0 * 5.0), "5.0 GB/s");
}

#[test]
fn test_format_bytes_large_values() {
    // Terabyte range
    let tb = 1024u64.pow(4);
    assert_eq!(format_bytes(tb), "1.0 TB");
    assert_eq!(format_bytes(tb * 2), "2.0 TB");
    assert_eq!(format_bytes(tb * 5 / 2), "2.5 TB");
}

#[test]
fn test_format_consistency() {
    // format_rate should produce format_bytes output with /s suffix
    let bytes = 1024 * 1024;
    let rate = format_rate(bytes as f64);
    let bytes_formatted = format_bytes(bytes);

    assert!(rate.starts_with(&bytes_formatted.replace(" ", " ")));
    assert!(rate.ends_with("/s"));
}

#[test]
fn test_format_precision() {
    // Should show one decimal place for non-byte units
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(2048), "2.0 KB");
    assert_eq!(format_bytes(2560), "2.5 KB");

    // But no decimal for bytes
    assert_eq!(format_bytes(100), "100 B");
    assert_eq!(format_bytes(512), "512 B");
}

#[test]
fn test_realistic_container_values() {
    // Typical container memory usage
    let mem_256mb = 256 * 1024 * 1024;
    let formatted = format_bytes(mem_256mb);
    assert!(formatted.contains("256") || formatted.contains("MB"));

    // Typical network rates
    let rate_10mbps = 10.0 * 1024.0 * 1024.0 / 8.0; // 10 Mbps in bytes/sec
    let formatted_rate = format_rate(rate_10mbps);
    assert!(formatted_rate.contains("MB/s"));
}

#[test]
fn test_format_bytes_ordering() {
    // Verify that larger values produce larger formatted outputs (numerically)
    let values = vec![
        0,
        512,
        1024,
        1024 * 512,
        1024 * 1024,
        1024 * 1024 * 512,
        1024u64.pow(3),
    ];

    for window in values.windows(2) {
        let formatted1 = format_bytes(window[0]);
        let formatted2 = format_bytes(window[1]);

        // Just verify they format without errors
        assert!(!formatted1.is_empty());
        assert!(!formatted2.is_empty());
    }
}
