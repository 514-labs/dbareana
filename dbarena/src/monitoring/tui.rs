use std::collections::VecDeque;
use std::io;
use std::time::Duration;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, Sparkline},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::error::Result;
use super::collector::MetricsCollector;
use super::metrics::ContainerMetrics;
use super::display::{format_bytes, format_rate};

const HISTORY_SIZE: usize = 60;

/// TUI application state
pub struct StatsTui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    metrics_history: VecDeque<ContainerMetrics>,
    paused: bool,
    show_help: bool,
}

impl StatsTui {
    /// Create a new TUI instance
    pub fn new() -> Result<Self> {
        enable_raw_mode()
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to enable raw mode: {}", e)))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to setup terminal: {}", e)))?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to create terminal: {}", e)))?;

        Ok(Self {
            terminal,
            metrics_history: VecDeque::with_capacity(HISTORY_SIZE),
            paused: false,
            show_help: false,
        })
    }

    /// Run the TUI for a single container
    pub async fn run_single(
        &mut self,
        collector: &impl MetricsCollector,
        container_id: &str,
    ) -> Result<()> {
        let mut previous_metrics: Option<ContainerMetrics> = None;
        let mut last_collection = tokio::time::Instant::now();
        let collection_interval = Duration::from_secs(2);

        loop {
            // Collect metrics if interval elapsed and not paused
            let now = tokio::time::Instant::now();
            if !self.paused && now.duration_since(last_collection) >= collection_interval {
                match collector.collect(container_id).await {
                    Ok(mut metrics) => {
                        // Calculate rates if we have previous metrics
                        if let Some(prev) = &previous_metrics {
                            metrics.calculate_rates(prev);
                        }

                        previous_metrics = Some(metrics.clone());
                        self.metrics_history.push_back(metrics);

                        // Keep only last HISTORY_SIZE data points
                        while self.metrics_history.len() > HISTORY_SIZE {
                            self.metrics_history.pop_front();
                        }

                        last_collection = now;
                    }
                    Err(e) => {
                        self.cleanup()?;
                        return Err(e);
                    }
                }
            }

            // Render TUI
            let show_help = self.show_help;
            let metrics_history = &self.metrics_history;
            let paused = self.paused;

            self.terminal
                .draw(|f| render_single_frame(f, metrics_history, paused, show_help))
                .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to draw: {}", e)))?;

            // Handle input with short timeout for responsive controls
            if crossterm::event::poll(Duration::from_millis(100))
                .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Event poll failed: {}", e)))?
            {
                if let Event::Key(key) = event::read()
                    .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to read event: {}", e)))?
                {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('f') => {
                            self.paused = !self.paused;
                            if !self.paused {
                                // Reset timer when unpausing
                                last_collection = tokio::time::Instant::now();
                            }
                        }
                        KeyCode::Char('r') => {
                            self.metrics_history.clear();
                            previous_metrics = None;
                            last_collection = tokio::time::Instant::now();
                        }
                        KeyCode::Char('h') | KeyCode::Char('?') => self.show_help = !self.show_help,
                        KeyCode::Esc => {
                            if self.show_help {
                                self.show_help = false;
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                // Small sleep to prevent busy-waiting when no events
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }

        self.cleanup()?;
        Ok(())
    }

}

/// Render single container view frame
fn render_single_frame(
    f: &mut Frame,
    metrics_history: &VecDeque<ContainerMetrics>,
    paused: bool,
    show_help: bool,
) {
    if show_help {
        render_help(f);
        return;
    }

    let current = match metrics_history.back() {
        Some(m) => m,
        None => {
            let block = Block::default()
                .title("dbarena stats")
                .borders(Borders::ALL);
            let text = Paragraph::new("Collecting metrics...").block(block);
            f.render_widget(text, f.size());
            return;
        }
    };

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(5),  // Gauges
            Constraint::Length(8),  // CPU chart
            Constraint::Length(8),  // Memory chart
            Constraint::Length(5),  // I/O stats
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    // Header
    render_header(f, chunks[0], current, paused);

    // Gauges
    render_gauges(f, chunks[1], current);

    // CPU history chart
    render_cpu_chart(f, chunks[2], metrics_history);

    // Memory history chart
    render_memory_chart(f, chunks[3], metrics_history);

    // I/O stats
    render_io_stats(f, chunks[4], current);

    // Footer
    render_footer(f, chunks[5]);
}

fn render_header(f: &mut Frame, area: Rect, metrics: &ContainerMetrics, paused: bool) {
    let block = Block::default()
        .title(format!(" dbarena stats - {} ", metrics.container_name))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let status = if paused {
        Span::styled(" [PAUSED] ", Style::default().fg(Color::Yellow))
    } else {
        Span::styled(" [LIVE] ", Style::default().fg(Color::Green))
    };

    let text = Line::from(vec![
        Span::raw("Container: "),
        Span::styled(&metrics.container_id[..12], Style::default().fg(Color::Blue)),
        Span::raw("  "),
        status,
    ]);

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_gauges(f: &mut Frame, area: Rect, metrics: &ContainerMetrics) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // CPU gauge
        // Clamp ratio to 0.0-1.0 for display, but show actual percentage in label
        let cpu_ratio = (metrics.cpu.usage_percent / 100.0).min(1.0).max(0.0);
        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU Usage").borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(if metrics.cpu.usage_percent > 90.0 {
                        Color::Red
                    } else if metrics.cpu.usage_percent > 75.0 {
                        Color::Yellow
                    } else {
                        Color::Green
                    })
            )
            .ratio(cpu_ratio)
            .label(format!("{:.1}%", metrics.cpu.usage_percent));
        f.render_widget(cpu_gauge, chunks[0]);

        // Memory gauge
        let memory_gauge = Gauge::default()
            .block(Block::default().title("Memory Usage").borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(if metrics.memory.percent > 90.0 {
                        Color::Red
                    } else if metrics.memory.percent > 75.0 {
                        Color::Yellow
                    } else {
                        Color::Green
                    })
            )
            .ratio(metrics.memory.percent / 100.0)
            .label(format!(
                "{} / {} ({:.1}%)",
                format_bytes(metrics.memory.usage),
                format_bytes(metrics.memory.limit),
                metrics.memory.percent
            ));
        f.render_widget(memory_gauge, chunks[1]);
}

fn render_cpu_chart(f: &mut Frame, area: Rect, metrics_history: &VecDeque<ContainerMetrics>) {
    // Clamp CPU values to 0-100 range for chart display
    let data: Vec<u64> = metrics_history
        .iter()
        .map(|m| (m.cpu.usage_percent.min(100.0).max(0.0)) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title("CPU History (60s) - capped at 100% for display")
                .borders(Borders::ALL)
        )
        .data(&data)
        .style(Style::default().fg(Color::Cyan))
        .max(100);

    f.render_widget(sparkline, area);
}

fn render_memory_chart(f: &mut Frame, area: Rect, metrics_history: &VecDeque<ContainerMetrics>) {
    let data: Vec<u64> = metrics_history
        .iter()
        .map(|m| m.memory.percent as u64)
        .collect();

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title("Memory History (60s)")
                .borders(Borders::ALL)
        )
        .data(&data)
        .style(Style::default().fg(Color::Green))
        .max(100);

    f.render_widget(sparkline, area);
}

fn render_io_stats(f: &mut Frame, area: Rect, metrics: &ContainerMetrics) {
        // Create bindings to extend lifetime of formatted strings
        let net_rx_bytes = format_bytes(metrics.network.rx_bytes);
        let net_rx_rate = format_rate(metrics.network.rx_rate);
        let net_tx_bytes = format_bytes(metrics.network.tx_bytes);
        let net_tx_rate = format_rate(metrics.network.tx_rate);
        let block_read_bytes = format_bytes(metrics.block_io.read_bytes);
        let block_read_rate = format_rate(metrics.block_io.read_rate);
        let block_write_bytes = format_bytes(metrics.block_io.write_bytes);
        let block_write_rate = format_rate(metrics.block_io.write_rate);

        let rows = vec![
            Row::new(vec!["Network RX", &net_rx_bytes, &net_rx_rate]),
            Row::new(vec!["Network TX", &net_tx_bytes, &net_tx_rate]),
            Row::new(vec!["Block Read", &block_read_bytes, &block_read_rate]),
            Row::new(vec!["Block Write", &block_write_bytes, &block_write_rate]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Length(15), Constraint::Length(15), Constraint::Length(15)],
        )
        .block(Block::default().title("I/O Statistics").borders(Borders::ALL))
        .header(Row::new(vec!["Type", "Total", "Rate"]).style(Style::default().fg(Color::Yellow)));

        f.render_widget(table, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let text = "q: quit | f: freeze/resume | r: reset history | h: help";

    let footer = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, area);
}

fn render_help(f: &mut Frame) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("  Keyboard Shortcuts", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(Color::Yellow)),
            Span::raw("      Quit the application"),
        ]),
        Line::from(vec![
            Span::styled("  f", Style::default().fg(Color::Yellow)),
            Span::raw("      Freeze/resume live updates"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Yellow)),
            Span::raw("      Reset history data"),
        ]),
        Line::from(vec![
            Span::styled("  h, ?", Style::default().fg(Color::Yellow)),
            Span::raw("    Show/hide this help"),
        ]),
        Line::from(""),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Cyan))
        );

    f.render_widget(help, f.size());
}

impl StatsTui {
    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to disable raw mode: {}", e)))?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to cleanup terminal: {}", e)))?;
        self.terminal
            .show_cursor()
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to show cursor: {}", e)))?;
        Ok(())
    }
}

impl Drop for StatsTui {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
