use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Table, Sparkline},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;

use crate::error::Result;
use crate::database_metrics::{DatabaseMetrics, DatabaseMetricsCollector};
use super::collector::MetricsCollector;
use super::logs::LogStreamer;
use super::metrics::ContainerMetrics;
use super::display::{format_bytes, format_rate};

const HISTORY_SIZE: usize = 60;
const LOG_BUFFER_SIZE: usize = 100;

/// View mode for multi-container TUI
#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    List,
    Detail,
    MultiPane,
}

/// Active pane in multi-pane view
#[derive(Debug, Clone, Copy, PartialEq)]
enum PaneType {
    Containers,
    Resource,
    Database,
    Logs,
}

/// TUI application state
pub struct StatsTui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    metrics_history: VecDeque<ContainerMetrics>,
    paused: bool,
    show_help: bool,
    collection_interval: Duration,
    view_mode: ViewMode,
    detail_container_id: Option<String>,

    // Multi-pane view state
    database_metrics_history: VecDeque<DatabaseMetrics>,
    log_lines: VecDeque<String>,
    active_pane: PaneType,
    show_logs: bool,
    log_scroll_offset: usize,
    selected_container_index: usize,
}

impl StatsTui {
    /// Create a new TUI instance with specified collection interval in milliseconds
    pub fn new(collection_interval_ms: u64) -> Result<Self> {
        enable_raw_mode()
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to enable raw mode: {}", e)))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to setup terminal: {}", e)))?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to create terminal: {}", e)))?;

        // Clamp interval to minimum 500ms to avoid stale Docker stats
        // Docker's internal stats update at ~1s, faster polling causes inconsistent data
        let safe_interval = collection_interval_ms.max(500);

        Ok(Self {
            terminal,
            metrics_history: VecDeque::with_capacity(HISTORY_SIZE),
            paused: false,
            show_help: false,
            collection_interval: Duration::from_millis(safe_interval),
            view_mode: ViewMode::List,
            detail_container_id: None,
            database_metrics_history: VecDeque::with_capacity(HISTORY_SIZE),
            log_lines: VecDeque::with_capacity(LOG_BUFFER_SIZE),
            active_pane: PaneType::Containers,
            show_logs: true,
            log_scroll_offset: 0,
            selected_container_index: 0,
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
        let collection_interval = self.collection_interval;

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
                .draw(|f| render_single_frame(f, metrics_history, paused, show_help, false))
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

    /// Run the TUI for multiple containers
    pub async fn run_multi(
        &mut self,
        collector: &impl MetricsCollector,
    ) -> Result<()> {
        let mut all_metrics: Vec<ContainerMetrics> = Vec::new();
        let mut selected_index = 0;
        let mut last_collection = tokio::time::Instant::now();
        let mut previous_metrics: Option<ContainerMetrics> = None;
        let collection_interval = self.collection_interval;

        self.view_mode = ViewMode::List;

        loop {
            let now = tokio::time::Instant::now();

            // Collect metrics based on view mode
            if !self.paused && now.duration_since(last_collection) >= collection_interval {
                match self.view_mode {
                    ViewMode::List => {
                        // Collect all containers
                        match collector.collect_all().await {
                            Ok(metrics) => {
                                all_metrics = metrics;
                                last_collection = now;
                            }
                            Err(e) => {
                                self.cleanup()?;
                                return Err(e);
                            }
                        }
                    }
                    ViewMode::Detail => {
                        // Collect single container
                        if let Some(container_id) = &self.detail_container_id {
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
                    }
                    ViewMode::MultiPane => {
                        // MultiPane mode is handled by run_multipane, not run_multi
                        // This shouldn't happen
                    }
                }
            }

            // Render TUI based on view mode
            let show_help = self.show_help;
            match self.view_mode {
                ViewMode::List => {
                    let paused = self.paused;
                    let all_metrics_ref = &all_metrics;
                    self.terminal.draw(|f| {
                        render_multi_frame(f, all_metrics_ref, selected_index, paused);
                    }).map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to draw TUI: {}", e)))?;
                }
                ViewMode::Detail => {
                    let paused = self.paused;
                    let metrics_history = &self.metrics_history;
                    self.terminal.draw(|f| {
                        render_single_frame(f, metrics_history, paused, show_help, true);
                    }).map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to draw TUI: {}", e)))?;
                }
                ViewMode::MultiPane => {
                    // MultiPane mode is handled by run_multipane, not run_multi
                    // This shouldn't happen
                }
            }

            // Handle input (non-blocking with timeout)
            if event::poll(Duration::from_millis(100))
                .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to poll events: {}", e)))? {
                if let Event::Key(key) = event::read()
                    .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to read event: {}", e)))? {
                    match self.view_mode {
                        ViewMode::List => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => break,
                                KeyCode::Char('f') => {
                                    self.paused = !self.paused;
                                }
                                KeyCode::Char('r') => {
                                    // Force refresh
                                    last_collection = tokio::time::Instant::now() - collection_interval;
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if selected_index > 0 {
                                        selected_index -= 1;
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if selected_index < all_metrics.len().saturating_sub(1) {
                                        selected_index += 1;
                                    }
                                }
                                KeyCode::Char(' ') | KeyCode::Enter => {
                                    // Switch to detail view for selected container
                                    if let Some(metrics) = all_metrics.get(selected_index) {
                                        self.view_mode = ViewMode::Detail;
                                        self.detail_container_id = Some(metrics.container_id.clone());
                                        self.metrics_history.clear();
                                        previous_metrics = None;
                                        last_collection = tokio::time::Instant::now() - collection_interval; // Force immediate collection
                                    }
                                }
                                KeyCode::Char('h') => {
                                    self.show_help = !self.show_help;
                                }
                                _ => {}
                            }
                        }
                        ViewMode::Detail => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Esc => {
                                    // Go back to list view
                                    self.view_mode = ViewMode::List;
                                    self.detail_container_id = None;
                                    self.metrics_history.clear();
                                    previous_metrics = None;
                                    last_collection = tokio::time::Instant::now() - collection_interval; // Force immediate collection
                                }
                                KeyCode::Char('f') | KeyCode::Char(' ') => {
                                    self.paused = !self.paused;
                                }
                                KeyCode::Char('r') => {
                                    // Force refresh
                                    last_collection = tokio::time::Instant::now() - collection_interval;
                                    self.metrics_history.clear();
                                    previous_metrics = None;
                                }
                                KeyCode::Char('h') => {
                                    self.show_help = !self.show_help;
                                }
                                _ => {}
                            }
                        }
                        ViewMode::MultiPane => {
                            // MultiPane mode is handled by run_multipane, not run_multi
                            // This shouldn't happen
                        }
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

    /// Run the enhanced multi-pane TUI with database metrics and logs
    pub async fn run_multipane(
        &mut self,
        collector: &impl MetricsCollector,
        db_collector: &impl DatabaseMetricsCollector,
        container_id: Option<String>,
    ) -> Result<()> {
        use crate::container::DatabaseType;
        use bollard::Docker;

        self.view_mode = ViewMode::MultiPane;

        let mut all_containers = Vec::new();
        let mut last_collection = tokio::time::Instant::now();
        let mut previous_resource_metrics: Option<ContainerMetrics> = None;
        let collection_interval = self.collection_interval;

        // Get Docker client for log streaming
        let docker = Arc::new(Docker::connect_with_local_defaults()?);
        let log_streamer = LogStreamer::new(docker.clone());
        let mut log_stream: Option<futures::stream::BoxStream<String>> = None;

        // Determine if we're monitoring a single container or all containers
        let single_container = container_id.is_some();
        if let Some(ref id) = container_id {
            self.detail_container_id = Some(id.clone());
        }

        loop {
            let now = tokio::time::Instant::now();

            // Collect metrics if interval elapsed and not paused
            if !self.paused && now.duration_since(last_collection) >= collection_interval {
                // Get the current container to monitor
                let target_container_id = if single_container {
                    container_id.clone()
                } else {
                    // Multi-container mode: collect all, use selected
                    match collector.collect_all().await {
                        Ok(metrics) => {
                            all_containers = metrics;
                            // Clamp selected index
                            if self.selected_container_index >= all_containers.len() {
                                self.selected_container_index = all_containers.len().saturating_sub(1);
                            }
                            all_containers.get(self.selected_container_index).map(|m| m.container_id.clone())
                        }
                        Err(_) => None,
                    }
                };

                if let Some(ref cid) = target_container_id {
                    // Collect resource metrics
                    match collector.collect(cid).await {
                        Ok(mut metrics) => {
                            if let Some(prev) = &previous_resource_metrics {
                                metrics.calculate_rates(prev);
                            }
                            previous_resource_metrics = Some(metrics.clone());
                            self.metrics_history.push_back(metrics);

                            while self.metrics_history.len() > HISTORY_SIZE {
                                self.metrics_history.pop_front();
                            }
                        }
                        Err(_) => {}
                    }

                    // Collect database metrics (try to detect database type)
                    // For now, try each database type and use the first that succeeds
                    for db_type in &[DatabaseType::Postgres, DatabaseType::MySQL, DatabaseType::SQLServer] {
                        match db_collector.collect(cid, *db_type).await {
                            Ok(db_metrics) => {
                                self.database_metrics_history.push_back(db_metrics);
                                while self.database_metrics_history.len() > HISTORY_SIZE {
                                    self.database_metrics_history.pop_front();
                                }
                                break; // Found the right database type
                            }
                            Err(_) => continue, // Try next type
                        }
                    }

                    // Start log stream if not already started
                    if log_stream.is_none() && self.show_logs {
                        match log_streamer.stream_logs(cid, 50).await {
                            Ok(stream) => {
                                log_stream = Some(stream);
                            }
                            Err(_) => {}
                        }
                    }
                }

                last_collection = now;
            }

            // Poll log stream (non-blocking)
            if let Some(ref mut stream) = log_stream {
                // Try to get up to 10 log lines per render cycle
                for _ in 0..10 {
                    match tokio::time::timeout(Duration::from_millis(1), stream.next()).await {
                        Ok(Some(line)) => {
                            self.log_lines.push_back(line);
                            while self.log_lines.len() > LOG_BUFFER_SIZE {
                                self.log_lines.pop_front();
                            }
                            // Auto-scroll to bottom if not manually scrolled
                            if self.active_pane != PaneType::Logs {
                                self.log_scroll_offset = 0;
                            }
                        }
                        _ => break, // No more logs available or timeout
                    }
                }
            }

            // Render multi-pane view
            let containers_list: Vec<String> = if single_container {
                if let Some(ref cid) = container_id {
                    vec![cid.clone()]
                } else {
                    vec![]
                }
            } else {
                all_containers.iter().map(|m| m.container_name.clone()).collect()
            };

            let resource_metrics = self.metrics_history.back().cloned();
            let db_metrics = self.database_metrics_history.back().cloned();
            let active_pane = self.active_pane;
            let show_logs = self.show_logs;
            let paused = self.paused;
            let selected_idx = self.selected_container_index;
            let log_lines = &self.log_lines;
            let log_scroll = self.log_scroll_offset;

            self.terminal.draw(|f| {
                render_multipane_frame(
                    f,
                    &containers_list,
                    selected_idx,
                    resource_metrics.as_ref(),
                    db_metrics.as_ref(),
                    log_lines,
                    log_scroll,
                    active_pane,
                    show_logs,
                    paused,
                );
            }).map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to draw: {}", e)))?;

            // Handle input
            if event::poll(Duration::from_millis(100))
                .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to poll events: {}", e)))? {
                if let Event::Key(key) = event::read()
                    .map_err(|e| crate::error::DBArenaError::MonitoringError(format!("Failed to read event: {}", e)))? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => break,
                        KeyCode::Char('f') => self.paused = !self.paused,
                        KeyCode::Char('r') => {
                            self.metrics_history.clear();
                            self.database_metrics_history.clear();
                            previous_resource_metrics = None;
                            last_collection = tokio::time::Instant::now();
                        }
                        KeyCode::Char('l') => self.show_logs = !self.show_logs,
                        KeyCode::Char('h') | KeyCode::Char('?') => self.show_help = !self.show_help,
                        KeyCode::Tab => {
                            // Cycle forward through panes
                            self.active_pane = match self.active_pane {
                                PaneType::Containers => PaneType::Resource,
                                PaneType::Resource => PaneType::Database,
                                PaneType::Database if self.show_logs => PaneType::Logs,
                                PaneType::Database => PaneType::Containers,
                                PaneType::Logs => PaneType::Containers,
                            };
                        }
                        KeyCode::BackTab => {
                            // Cycle backward through panes (Shift+Tab)
                            self.active_pane = match self.active_pane {
                                PaneType::Containers if self.show_logs => PaneType::Logs,
                                PaneType::Containers => PaneType::Database,
                                PaneType::Resource => PaneType::Containers,
                                PaneType::Database => PaneType::Resource,
                                PaneType::Logs => PaneType::Database,
                            };
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            match self.active_pane {
                                PaneType::Containers => {
                                    if !single_container && self.selected_container_index > 0 {
                                        self.selected_container_index -= 1;
                                        // Switch log stream to new container
                                        log_stream = None;
                                        self.log_lines.clear();
                                        self.metrics_history.clear();
                                        self.database_metrics_history.clear();
                                        previous_resource_metrics = None;
                                    }
                                }
                                PaneType::Logs => {
                                    if self.log_scroll_offset < self.log_lines.len().saturating_sub(1) {
                                        self.log_scroll_offset += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            match self.active_pane {
                                PaneType::Containers => {
                                    if !single_container && self.selected_container_index < all_containers.len().saturating_sub(1) {
                                        self.selected_container_index += 1;
                                        // Switch log stream to new container
                                        log_stream = None;
                                        self.log_lines.clear();
                                        self.metrics_history.clear();
                                        self.database_metrics_history.clear();
                                        previous_resource_metrics = None;
                                    }
                                }
                                PaneType::Logs => {
                                    if self.log_scroll_offset > 0 {
                                        self.log_scroll_offset -= 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            } else {
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
    in_multi_mode: bool,
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
            Constraint::Length(8),  // I/O stats (5 data rows + header + borders)
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
    render_footer(f, chunks[5], in_multi_mode);
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
                .title("CPU History - capped at 100% for display")
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
                .title("Memory History")
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
        let pids_str = metrics.pids.to_string();

        let rows = vec![
            Row::new(vec!["Network RX", &net_rx_bytes, &net_rx_rate]),
            Row::new(vec!["Network TX", &net_tx_bytes, &net_tx_rate]),
            Row::new(vec!["Disk Read", &block_read_bytes, &block_read_rate]),
            Row::new(vec!["Disk Write", &block_write_bytes, &block_write_rate]),
            Row::new(vec!["PIDs", &pids_str, ""]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Length(15), Constraint::Length(15), Constraint::Length(15)],
        )
        .block(Block::default().title("I/O & Process Statistics").borders(Borders::ALL))
        .header(Row::new(vec!["Type", "Total", "Rate"]).style(Style::default().fg(Color::Yellow)));

        f.render_widget(table, area);
}

fn render_footer(f: &mut Frame, area: Rect, in_multi_mode: bool) {
    let text = if in_multi_mode {
        "Esc: back to list | f: freeze/resume | r: reset history | h: help | q: quit"
    } else {
        "q: quit | f: freeze/resume | r: reset history | h: help"
    };

    let footer = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, area);
}

/// Render multi-container view frame
fn render_multi_frame(
    f: &mut Frame,
    all_metrics: &[ContainerMetrics],
    selected_index: usize,
    paused: bool,
) {
    if all_metrics.is_empty() {
        let block = Block::default()
            .title("dbarena stats --all")
            .borders(Borders::ALL);
        let text = Paragraph::new("Collecting metrics...").block(block);
        f.render_widget(text, f.size());
        return;
    }

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(10),    // Container list
            Constraint::Length(1),  // Footer
        ])
        .split(f.size());

    // Header
    let pause_indicator = if paused { " [PAUSED]" } else { "" };
    let header = Paragraph::new(Line::from(vec![
        Span::raw("dbarena stats --all"),
        Span::styled(
            pause_indicator,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Container list table
    let header_cells = ["Container", "CPU", "Memory", "Network", "Status"]
        .iter()
        .map(|h| ratatui::widgets::Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header_row = Row::new(header_cells).height(1);

    let rows = all_metrics.iter().enumerate().map(|(idx, m)| {
        let cpu_str = format!("{:.1}%", m.cpu.usage_percent);
        let mem_str = format!("{}/{}", format_bytes(m.memory.usage), format_bytes(m.memory.limit));
        let net_str = format_rate(m.network.rx_rate + m.network.tx_rate);
        let status_str = "Running"; // TODO: Get actual status

        let style = if idx == selected_index {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        Row::new(vec![
            m.container_name.clone(),
            cpu_str,
            mem_str,
            net_str,
            status_str.to_string(),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ],
    )
    .header(header_row)
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(table, chunks[1]);

    // Footer with instructions
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("[↑/↓: navigate]  "),
        Span::raw("[Space/Enter: details]  "),
        Span::raw("[f: freeze]  "),
        Span::raw("[r: refresh]  "),
        Span::raw("[q: quit]"),
    ]))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(footer, chunks[2]);
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

/// Render multi-pane dashboard frame
#[allow(clippy::too_many_arguments)]
fn render_multipane_frame(
    f: &mut Frame,
    containers: &[String],
    selected_container: usize,
    resource_metrics: Option<&ContainerMetrics>,
    db_metrics: Option<&DatabaseMetrics>,
    log_lines: &VecDeque<String>,
    log_scroll_offset: usize,
    active_pane: PaneType,
    show_logs: bool,
    paused: bool,
) {
    // Main layout: left sidebar (20%) | right content (80%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.size());

    // Right side layout: resource metrics (30%) | db metrics (30%) | logs (40%)
    let right_constraints = if show_logs {
        vec![
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ]
    } else {
        vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]
    };

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(right_constraints)
        .split(main_chunks[1]);

    // Render container list
    render_container_list_pane(f, main_chunks[0], containers, selected_container, active_pane == PaneType::Containers);

    // Render resource metrics
    render_resource_metrics_pane(
        f,
        right_chunks[0],
        resource_metrics,
        active_pane == PaneType::Resource,
        paused,
    );

    // Render database metrics
    render_database_metrics_pane(
        f,
        right_chunks[1],
        db_metrics,
        active_pane == PaneType::Database,
    );

    // Render logs if enabled
    if show_logs {
        render_logs_pane(
            f,
            right_chunks[2],
            log_lines,
            log_scroll_offset,
            active_pane == PaneType::Logs,
        );
    }
}

fn render_container_list_pane(
    f: &mut Frame,
    area: Rect,
    containers: &[String],
    selected_index: usize,
    is_active: bool,
) {
    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

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
            .border_style(border_style),
    );

    f.render_widget(list, area);
}

fn render_resource_metrics_pane(
    f: &mut Frame,
    area: Rect,
    metrics: Option<&ContainerMetrics>,
    is_active: bool,
    paused: bool,
) {
    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    let status = if paused { " [PAUSED]" } else { "" };

    if let Some(m) = metrics {
        let cpu_ratio = (m.cpu.usage_percent / 100.0).min(1.0).max(0.0);
        let mem_ratio = (m.memory.percent / 100.0).min(1.0).max(0.0);

        let content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("CPU: "),
                Span::styled(
                    format!("{:.1}%", m.cpu.usage_percent),
                    Style::default().fg(if m.cpu.usage_percent > 80.0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ]),
            Line::from(gauge_bar(cpu_ratio, 30)),
            Line::from(""),
            Line::from(vec![
                Span::raw("Memory: "),
                Span::styled(
                    format!(
                        "{} / {} ({:.1}%)",
                        format_bytes(m.memory.usage),
                        format_bytes(m.memory.limit),
                        m.memory.percent
                    ),
                    Style::default().fg(if m.memory.percent > 80.0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ]),
            Line::from(gauge_bar(mem_ratio, 30)),
            Line::from(""),
            Line::from(format!(
                "Network: RX {} | TX {}",
                format_rate(m.network.rx_rate),
                format_rate(m.network.tx_rate)
            )),
            Line::from(format!(
                "Disk: R {} | W {}",
                format_rate(m.block_io.read_rate),
                format_rate(m.block_io.write_rate)
            )),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title(format!(" Resource Metrics{} ", status))
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("Collecting metrics...").block(
            Block::default()
                .title(" Resource Metrics ")
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        f.render_widget(paragraph, area);
    }
}

fn render_database_metrics_pane(
    f: &mut Frame,
    area: Rect,
    metrics: Option<&DatabaseMetrics>,
    is_active: bool,
) {
    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    if let Some(m) = metrics {
        let conn_usage = if let Some(max) = m.max_connections {
            if max > 0 {
                format!("{} / {} ({:.1}%)", m.active_connections, max, (m.active_connections as f64 / max as f64) * 100.0)
            } else {
                format!("{}", m.active_connections)
            }
        } else {
            format!("{}", m.active_connections)
        };

        let cache_hit = if let Some(ratio) = m.cache_hit_ratio {
            format!("{:.1}%", ratio)
        } else {
            "N/A".to_string()
        };

        let repl_status = m.replication_status.as_ref().map(|s| s.as_str()).unwrap_or("None");

        let content = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("Connections: "),
                Span::styled(conn_usage, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(format!("QPS: {:.2}", m.queries_per_second)),
            Line::from(format!("TPS: {:.2}", m.transactions_per_second)),
            Line::from(""),
            Line::from(vec![
                Span::raw("Cache Hit: "),
                Span::styled(cache_hit, Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(format!("SELECT: {} | INSERT: {}", m.query_breakdown.select_count, m.query_breakdown.insert_count)),
            Line::from(format!("UPDATE: {} | DELETE: {}", m.query_breakdown.update_count, m.query_breakdown.delete_count)),
            Line::from(""),
            Line::from(vec![
                Span::raw("Replication: "),
                Span::styled(repl_status, Style::default().fg(Color::Yellow)),
            ]),
        ];

        let paragraph = Paragraph::new(content).block(
            Block::default()
                .title(format!(" Database Metrics ({:?}) ", m.database_type))
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No database metrics available\n(container may not be a supported database)").block(
            Block::default()
                .title(" Database Metrics ")
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        f.render_widget(paragraph, area);
    }
}

fn render_logs_pane(
    f: &mut Frame,
    area: Rect,
    log_lines: &VecDeque<String>,
    scroll_offset: usize,
    is_active: bool,
) {
    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };

    // Calculate visible area
    let available_height = area.height.saturating_sub(2) as usize; // Minus borders

    // Get visible log lines (reverse so newest is at bottom)
    let total_lines = log_lines.len();
    let start_idx = if scroll_offset == 0 {
        total_lines.saturating_sub(available_height)
    } else {
        total_lines.saturating_sub(available_height + scroll_offset)
    };

    let visible_lines: Vec<Line> = log_lines
        .iter()
        .skip(start_idx)
        .take(available_height)
        .map(|line| {
            // Truncate long lines to fit
            let truncated = if line.len() > (area.width as usize).saturating_sub(3) {
                format!("{}...", &line[..(area.width as usize).saturating_sub(6)])
            } else {
                line.clone()
            };
            Line::from(truncated)
        })
        .collect();

    let scroll_indicator = if scroll_offset > 0 {
        format!(" [↑{}] ", scroll_offset)
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(visible_lines).block(
        Block::default()
            .title(format!(" Logs{} ", scroll_indicator))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(paragraph, area);
}

/// Create a simple text-based gauge bar
fn gauge_bar(ratio: f64, width: usize) -> String {
    let filled = (ratio * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
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
