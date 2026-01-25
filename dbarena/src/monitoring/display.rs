use console::{style, Color};
use super::metrics::ContainerMetrics;

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let index = (bytes_f.log10() / 1024_f64.log10()).floor() as usize;
    let index = index.min(UNITS.len() - 1);

    let value = bytes_f / 1024_f64.powi(index as i32);

    if index == 0 {
        format!("{} {}", bytes, UNITS[index])
    } else {
        format!("{:.1} {}", value, UNITS[index])
    }
}

/// Format bytes per second as human-readable string
pub fn format_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec as u64))
}

/// Format percentage with color coding
pub fn format_percent(percent: f64, warning_threshold: f64, danger_threshold: f64) -> String {
    let color = if percent >= danger_threshold {
        Color::Red
    } else if percent >= warning_threshold {
        Color::Yellow
    } else {
        Color::Green
    };

    style(format!("{:.1}%", percent)).fg(color).to_string()
}

/// Display metrics as simple text output
pub fn display_metrics_simple(metrics: &ContainerMetrics) {
    println!("Container: {}", style(&metrics.container_name).bold().cyan());
    println!();

    println!("CPU:");
    println!("  Usage: {}", format_percent(metrics.cpu.usage_percent, 75.0, 90.0));
    println!("  Cores: {}", metrics.cpu.num_cores);
    println!();

    println!("Memory:");
    println!("  Usage: {} / {}",
        format_bytes(metrics.memory.usage),
        format_bytes(metrics.memory.limit)
    );
    println!("  Percent: {}", format_percent(metrics.memory.percent, 75.0, 90.0));
    println!();

    println!("Network I/O:");
    println!("  RX: {} ({})",
        format_bytes(metrics.network.rx_bytes),
        format_rate(metrics.network.rx_rate)
    );
    println!("  TX: {} ({})",
        format_bytes(metrics.network.tx_bytes),
        format_rate(metrics.network.tx_rate)
    );
    println!();

    println!("Block I/O:");
    println!("  Read:  {} ({})",
        format_bytes(metrics.block_io.read_bytes),
        format_rate(metrics.block_io.read_rate)
    );
    println!("  Write: {} ({})",
        format_bytes(metrics.block_io.write_bytes),
        format_rate(metrics.block_io.write_rate)
    );
}

/// Display metrics as a compact single-line table row
pub fn display_metrics_compact(metrics: &ContainerMetrics) {
    println!(
        "{:<20} {:>8} {:>15} {:>12}",
        truncate_string(&metrics.container_name, 20),
        format_percent(metrics.cpu.usage_percent, 75.0, 90.0),
        format!("{}/{}",
            format_bytes(metrics.memory.usage),
            format_bytes(metrics.memory.limit)
        ),
        format_rate(metrics.network.rx_rate + metrics.network.tx_rate)
    );
}

/// Display header for compact table
pub fn display_compact_header() {
    println!(
        "{:<20} {:>8} {:>15} {:>12}",
        "CONTAINER",
        "CPU",
        "MEMORY",
        "NETWORK"
    );
    println!("{}", "─".repeat(60));
}

/// Truncate string to max length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1536 * 1024), "1.5 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_rate() {
        assert_eq!(format_rate(0.0), "0 B/s");
        assert_eq!(format_rate(1024.0), "1.0 KB/s");
        assert_eq!(format_rate(1024.0 * 1024.0), "1.0 MB/s");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a very long string", 10), "this is...");
    }
}
