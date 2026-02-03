use console::style;
use std::time::{Duration, Instant};

use crate::workload::stats::WorkloadStats;

/// Live progress display for workload execution
pub struct WorkloadProgressDisplay {
    start_time: Instant,
    target_tps: usize,
    duration: Option<Duration>,
    transaction_count: Option<u64>,
}

impl WorkloadProgressDisplay {
    pub fn new(
        target_tps: usize,
        duration: Option<Duration>,
        transaction_count: Option<u64>,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            target_tps,
            duration,
            transaction_count,
        }
    }

    /// Render the current progress
    pub fn render(&self, stats: &WorkloadStats) {
        let elapsed = self.start_time.elapsed();
        let snapshot = stats.snapshot();

        // Clear previous lines (move up and clear)
        print!("\x1B[2J\x1B[1;1H"); // Clear screen and move to top

        println!("{}", style("=".repeat(70)).dim());
        println!("{}", style("Workload Progress").cyan().bold());
        println!("{}", style("=".repeat(70)).dim());
        println!();

        // Time progress
        if let Some(duration) = self.duration {
            let percent = (elapsed.as_secs_f64() / duration.as_secs_f64() * 100.0).min(100.0);
            println!(
                "  {} Time: {:.1}s / {:.1}s ({:.1}%)",
                style("⏱").cyan(),
                elapsed.as_secs_f64(),
                duration.as_secs_f64(),
                percent
            );
            self.print_progress_bar(percent as usize);
        } else if let Some(count) = self.transaction_count {
            let percent = (snapshot.total as f64 / count as f64 * 100.0).min(100.0);
            println!(
                "  {} Transactions: {} / {} ({:.1}%)",
                style("📊").cyan(),
                snapshot.total,
                count,
                percent
            );
            self.print_progress_bar(percent as usize);
        } else {
            println!("  {} Time: {:.1}s", style("⏱").cyan(), elapsed.as_secs_f64());
        }

        println!();

        // Throughput
        println!(
            "  {} TPS: {} (target: {})",
            style("⚡").yellow(),
            style(format!("{:.1}", snapshot.tps)).green().bold(),
            style(self.target_tps).dim()
        );

        let tps_diff = snapshot.tps - self.target_tps as f64;
        let tps_indicator = if tps_diff.abs() < self.target_tps as f64 * 0.1 {
            style(format!("✓ On target")).green()
        } else if tps_diff > 0.0 {
            style(format!("▲ +{:.1} above target", tps_diff)).yellow()
        } else {
            style(format!("▼ {:.1} below target", tps_diff.abs())).red()
        };
        println!("     {}", tps_indicator);

        println!();

        // Success rate
        let success_color = if snapshot.success_rate >= 95.0 {
            style(format!("{:.1}%", snapshot.success_rate)).green()
        } else if snapshot.success_rate >= 80.0 {
            style(format!("{:.1}%", snapshot.success_rate)).yellow()
        } else {
            style(format!("{:.1}%", snapshot.success_rate)).red()
        };

        println!(
            "  {} Success: {} ({} / {} total)",
            style("✓").green(),
            success_color,
            snapshot.success,
            snapshot.total
        );

        if snapshot.failed > 0 {
            println!(
                "  {} Failed: {} transactions",
                style("✗").red(),
                style(snapshot.failed).red()
            );
        }

        println!();

        // Latency
        println!("  {} Latency:", style("⏲").cyan());
        if let Some(p50) = snapshot.p50 {
            println!("     P50: {:.2}ms", p50 as f64 / 1000.0);
        }
        if let Some(p95) = snapshot.p95 {
            println!("     P95: {:.2}ms", p95 as f64 / 1000.0);
        }
        if let Some(p99) = snapshot.p99 {
            println!("     P99: {:.2}ms", p99 as f64 / 1000.0);
        }
        if let Some(mean) = snapshot.mean {
            println!("     Mean: {:.2}ms", mean / 1000.0);
        }

        println!();

        // Operation counts
        if !snapshot.operation_counts.is_empty() {
            println!("  {} Operations:", style("📝").cyan());
            let total_ops: u64 = snapshot.operation_counts.values().sum();
            for (op, count) in &snapshot.operation_counts {
                let percentage = (*count as f64 / total_ops as f64) * 100.0;
                println!(
                    "     {}: {} ({:.1}%)",
                    op,
                    style(count).yellow(),
                    percentage
                );
            }
            println!();
        }

        // Errors (if any)
        if !snapshot.error_counts.is_empty() {
            println!("  {} Recent Errors:", style("⚠").red());
            for (error, count) in snapshot.error_counts.iter().take(5) {
                let error_short = if error.len() > 60 {
                    format!("{}...", &error[..57])
                } else {
                    error.clone()
                };
                println!("     {}: {}", style(count).red(), error_short);
            }
            println!();
        }

        println!("{}", style("─".repeat(70)).dim());
        println!("  {} Press Ctrl+C to stop", style("ℹ").blue());
    }

    /// Print a simple progress bar
    fn print_progress_bar(&self, percent: usize) {
        let width = 50;
        let filled = (percent * width / 100).min(width);
        let empty = width - filled;

        print!("     [");
        print!("{}", style("=".repeat(filled)).green());
        print!("{}", " ".repeat(empty));
        println!("]");
    }
}

/// Print final summary after workload completes
pub fn print_summary(stats: &WorkloadStats, pattern_name: &str) {
    let snapshot = stats.snapshot();

    println!();
    println!("{}", style("=".repeat(70)).dim());
    println!("{}", style("Workload Complete").green().bold());
    println!("{}", style("=".repeat(70)).dim());
    println!();

    println!("  Pattern: {}", style(pattern_name).cyan());
    println!("  Duration: {:.2}s", snapshot.elapsed.as_secs_f64());
    println!();

    println!("  {} Total Transactions: {}", style("📊").cyan(), style(snapshot.total).green().bold());
    println!("     Successful: {} ({:.1}%)", snapshot.success, snapshot.success_rate);
    println!("     Failed: {}", snapshot.failed);
    println!();

    println!("  {} Throughput: {:.1} TPS", style("⚡").yellow(), style(snapshot.tps).green().bold());
    println!();

    println!("  {} Latency:", style("⏲").cyan());
    if let Some(p50) = snapshot.p50 {
        println!("     P50: {:.2}ms", p50 as f64 / 1000.0);
    }
    if let Some(p95) = snapshot.p95 {
        println!("     P95: {:.2}ms", p95 as f64 / 1000.0);
    }
    if let Some(p99) = snapshot.p99 {
        println!("     P99: {:.2}ms", p99 as f64 / 1000.0);
    }
    if let Some(max) = snapshot.max {
        println!("     Max: {:.2}ms", max as f64 / 1000.0);
    }
    println!();

    if !snapshot.operation_counts.is_empty() {
        println!("  {} Operation Distribution:", style("📝").cyan());
        let total_ops: u64 = snapshot.operation_counts.values().sum();
        for (op, count) in &snapshot.operation_counts {
            let percentage = (*count as f64 / total_ops as f64) * 100.0;
            println!("     {}: {} ({:.1}%)", op, count, percentage);
        }
        println!();
    }

    if !snapshot.error_counts.is_empty() {
        println!("  {} Error Summary:", style("⚠").red());
        let total_errors: u64 = snapshot.error_counts.values().sum();
        println!("     Total errors: {}", total_errors);
        println!("     Unique error types: {}", snapshot.error_counts.len());
        println!();
    }

    println!("{}", style("=".repeat(70)).dim());
    println!();
}
