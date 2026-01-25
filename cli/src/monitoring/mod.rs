//! Performance monitoring module
//!
//! Provides real-time CPU, memory, network, and I/O metrics collection
//! for Docker containers with both simple text output and interactive TUI.

pub mod metrics;
pub mod collector;
pub mod docker_stats;
pub mod display;
pub mod logs;
pub mod tui;

pub use metrics::{ContainerMetrics, CpuMetrics, MemoryMetrics, NetworkMetrics, BlockIoMetrics};
pub use collector::MetricsCollector;
pub use docker_stats::DockerStatsCollector;
pub use display::{display_metrics_simple, display_metrics_compact, display_compact_header, format_bytes, format_rate};
pub use logs::LogStreamer;
pub use tui::StatsTui;
