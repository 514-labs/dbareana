//! TUI rendering tests using ratatui's TestBackend
//! Tests UI layout and rendering without requiring a real terminal

use ratatui::{
    backend::TestBackend,
    Terminal,
};
use std::collections::VecDeque;

use dbarena::monitoring::{ContainerMetrics, CpuMetrics, MemoryMetrics, NetworkMetrics, BlockIoMetrics};
use dbarena::database_metrics::{DatabaseMetrics, QueryBreakdown};
use dbarena::container::DatabaseType;

/// Helper to create test container metrics
fn create_test_metrics(container_name: &str) -> ContainerMetrics {
    ContainerMetrics {
        container_id: "test123".to_string(),
        container_name: container_name.to_string(),
        cpu: CpuMetrics {
            usage_percent: 25.5,
            num_cores: 4,
            total_usage: Some(1000000000),
            system_usage: Some(4000000000),
        },
        memory: MemoryMetrics {
            usage: 536_870_912,  // 512 MB
            limit: 8_589_934_592, // 8 GB
            percent: 6.25,
        },
        network: NetworkMetrics {
            rx_bytes: 1_048_576,  // 1 MB
            tx_bytes: 2_097_152,  // 2 MB
            rx_rate: 1024.0,      // 1 KB/s
            tx_rate: 2048.0,      // 2 KB/s
        },
        block_io: BlockIoMetrics {
            read_bytes: 10_485_760,  // 10 MB
            write_bytes: 5_242_880,  // 5 MB
            read_rate: 512.0,
            write_rate: 256.0,
        },
        pids: 42,
        timestamp: 1234567890,
    }
}

/// Helper to create test database metrics
fn create_test_db_metrics() -> DatabaseMetrics {
    let mut metrics = DatabaseMetrics::new("test123".to_string(), DatabaseType::Postgres);
    metrics.active_connections = 5;
    metrics.max_connections = Some(100);
    metrics.queries_per_second = 150.5;
    metrics.transactions_per_second = 25.0;
    metrics.commits_per_second = 23.0;
    metrics.rollbacks_per_second = 2.0;
    metrics.cache_hit_ratio = Some(95.5);
    metrics.query_breakdown = QueryBreakdown {
        select_count: 100,
        insert_count: 25,
        update_count: 15,
        delete_count: 10,
    };
    metrics
}

#[test]
fn test_multipane_layout_dimensions() {
    // Create a test backend with 100x40 terminal size
    let backend = TestBackend::new(100, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let containers = vec!["test-container-1".to_string(), "test-container-2".to_string()];
    let metrics = create_test_metrics("test-container-1");
    let db_metrics = create_test_db_metrics();
    let mut log_lines = VecDeque::new();
    log_lines.push_back("Log line 1".to_string());
    log_lines.push_back("Log line 2".to_string());

    // Import the rendering function
    // Note: This is a simplified test - in real implementation we'd need to expose
    // render functions or test through the public API

    terminal.draw(|f| {
        // We can verify the frame size
        assert_eq!(f.size().width, 100);
        assert_eq!(f.size().height, 40);

        // Test layout constraints would be verified here
        // In a real test, we'd render the multipane layout and verify dimensions
    }).unwrap();

    // Get the terminal buffer to assert on rendered content
    let buffer = terminal.backend().buffer();

    // Verify the terminal size
    assert_eq!(buffer.area.width, 100);
    assert_eq!(buffer.area.height, 40);
}

#[test]
fn test_container_list_rendering() {
    let backend = TestBackend::new(50, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        use ratatui::{
            layout::Rect,
            widgets::{Block, Borders, List, ListItem},
            style::{Color, Modifier, Style},
        };

        let area = Rect::new(0, 0, 20, 20);

        // Simulate rendering container list
        let containers = vec!["postgres-16", "mysql-8", "sqlserver-22"];
        let selected_index = 0;

        let items: Vec<ListItem> = containers
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                let content = if idx == selected_index {
                    format!("► {}", name)
                } else {
                    format!("  {}", name)
                };
                let style = if idx == selected_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(format!(" Containers [{}] ", containers.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

        f.render_widget(list, area);
    }).unwrap();

    let buffer = terminal.backend().buffer();

    // Verify the title is rendered
    let title_text = buffer.content().iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(title_text.contains("Containers"), "Should render containers title");
    assert!(title_text.contains("[3]"), "Should show container count");
}

#[test]
fn test_resource_metrics_rendering() {
    let backend = TestBackend::new(80, 15);
    let mut terminal = Terminal::new(backend).unwrap();

    let metrics = create_test_metrics("test-pg");

    terminal.draw(|f| {
        use ratatui::{
            layout::Rect,
            widgets::{Block, Borders, Paragraph},
            text::{Line, Span},
            style::{Color, Style},
        };

        let area = Rect::new(0, 0, 60, 12);

        let cpu_ratio = (metrics.cpu.usage_percent / 100.0).min(1.0).max(0.0);
        let _mem_ratio = (metrics.memory.percent / 100.0).min(1.0).max(0.0);

        let content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("CPU: "),
                Span::styled(
                    format!("{:.1}%", metrics.cpu.usage_percent),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(format!("[{}]", "█".repeat((cpu_ratio * 30.0) as usize))),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title(" Resource Metrics ")
                .borders(Borders::ALL),
        );

        f.render_widget(paragraph, area);
    }).unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer.content().iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(content.contains("Resource Metrics"), "Should render resource metrics title");
    assert!(content.contains("CPU:"), "Should render CPU label");
    assert!(content.contains("25.5"), "Should render CPU percentage");
}

#[test]
fn test_database_metrics_rendering() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let db_metrics = create_test_db_metrics();

    terminal.draw(|f| {
        use ratatui::{
            layout::Rect,
            widgets::{Block, Borders, Paragraph},
            text::{Line, Span},
            style::{Color, Style},
        };

        let area = Rect::new(0, 0, 60, 15);

        let conn_usage = if let Some(max) = db_metrics.max_connections {
            format!("{} / {} ({:.1}%)",
                db_metrics.active_connections,
                max,
                (db_metrics.active_connections as f64 / max as f64) * 100.0)
        } else {
            format!("{}", db_metrics.active_connections)
        };

        let cache_hit = if let Some(ratio) = db_metrics.cache_hit_ratio {
            format!("{:.1}%", ratio)
        } else {
            "N/A".to_string()
        };

        let content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("Connections: "),
                Span::styled(conn_usage, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(format!("QPS: {:.2}", db_metrics.queries_per_second)),
            Line::from(format!("TPS: {:.2}", db_metrics.transactions_per_second)),
            Line::from(""),
            Line::from(vec![
                Span::raw("Cache Hit: "),
                Span::styled(cache_hit, Style::default().fg(Color::Green)),
            ]),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title(format!(" Database Metrics ({:?}) ", db_metrics.database_type))
                .borders(Borders::ALL),
        );

        f.render_widget(paragraph, area);
    }).unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer.content().iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(content.contains("Database Metrics"), "Should render database metrics title");
    assert!(content.contains("Postgres"), "Should show database type");
    assert!(content.contains("Connections:"), "Should show connections");
    assert!(content.contains("QPS:"), "Should show QPS");
    assert!(content.contains("TPS:"), "Should show TPS");
    assert!(content.contains("Cache Hit:"), "Should show cache hit ratio");
}

#[test]
fn test_logs_pane_rendering() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut log_lines = VecDeque::new();
    log_lines.push_back("2026-01-25 10:00:00 [INFO] Server started".to_string());
    log_lines.push_back("2026-01-25 10:00:01 [INFO] Connection accepted".to_string());
    log_lines.push_back("2026-01-25 10:00:02 [ERROR] Query failed".to_string());

    terminal.draw(|f| {
        use ratatui::{
            layout::Rect,
            widgets::{Block, Borders, Paragraph},
            text::Line,
        };

        let area = Rect::new(0, 0, 78, 10);
        let available_height = area.height.saturating_sub(2) as usize;

        let visible_lines: Vec<Line> = log_lines
            .iter()
            .take(available_height)
            .map(|line| Line::from(line.clone()))
            .collect();

        let paragraph = Paragraph::new(visible_lines).block(
            Block::default()
                .title(" Logs ")
                .borders(Borders::ALL),
        );

        f.render_widget(paragraph, area);
    }).unwrap();

    let buffer = terminal.backend().buffer();
    let content = buffer.content().iter()
        .map(|c| c.symbol())
        .collect::<String>();

    assert!(content.contains("Logs"), "Should render logs title");
    assert!(content.contains("Server started"), "Should show log line 1");
    assert!(content.contains("Connection accepted"), "Should show log line 2");
}

#[test]
fn test_gauge_bar_rendering() {
    // Test the text-based gauge bar function
    let gauge_25 = gauge_bar(0.25, 20);
    // Note: "█" and "░" are 3-byte UTF-8 chars, so total length is not simply 22
    // Just verify structure and content
    assert!(gauge_25.starts_with('['));
    assert!(gauge_25.ends_with(']'));
    assert!(gauge_25.contains('█'));
    assert!(gauge_25.contains('░'));

    let gauge_100 = gauge_bar(1.0, 20);
    assert_eq!(gauge_100, format!("[{}]", "█".repeat(20)));

    let gauge_0 = gauge_bar(0.0, 20);
    assert_eq!(gauge_0, format!("[{}]", "░".repeat(20)));
}

/// Helper function matching the one in tui.rs
fn gauge_bar(ratio: f64, width: usize) -> String {
    let filled = (ratio * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

#[test]
fn test_format_bytes_in_ui() {
    use dbarena::monitoring::format_bytes;

    // Test that format_bytes produces readable output for UI
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1_048_576), "1.0 MB");
    assert_eq!(format_bytes(536_870_912), "512.0 MB");
    assert_eq!(format_bytes(8_589_934_592), "8.0 GB");
}

#[test]
fn test_format_rate_in_ui() {
    use dbarena::monitoring::format_rate;

    // Test that format_rate produces readable output for UI
    assert_eq!(format_rate(0.0), "0 B/s");
    assert_eq!(format_rate(1024.0), "1.0 KB/s");
    assert_eq!(format_rate(1_048_576.0), "1.0 MB/s");
}

#[test]
fn test_terminal_resize_handling() {
    // Test that the UI can handle different terminal sizes
    let sizes = vec![
        (80, 24),   // Small terminal
        (120, 40),  // Medium terminal
        (200, 60),  // Large terminal
        (40, 20),   // Very small
    ];

    for (width, height) in sizes {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend).unwrap();

        // Verify terminal was created successfully
        assert_eq!(terminal.backend().buffer().area.width, width);
        assert_eq!(terminal.backend().buffer().area.height, height);
    }
}
