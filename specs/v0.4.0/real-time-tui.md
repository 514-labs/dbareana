# Real-Time TUI (Terminal User Interface)

## Feature Overview

An interactive terminal-based dashboard providing real-time visualization of database metrics, resource consumption, and container status. Built with Ratatui, the TUI offers a multi-pane layout with live updates, keyboard navigation, and ASCII-based charts for monitoring multiple database instances simultaneously.

**Status:** Implemented. CLI command name is `dbarena` (legacy examples may still show `simdb`). See `specs/IMPLEMENTATION_TRUTH.md`.

## Problem Statement

Monitoring multiple databases requires:
- Running `docker stats` and `simdb stats` in separate terminals
- Manually switching between windows to see different metrics
- Using external tools to visualize trends
- Parsing raw text output to understand system state

Users need a unified, real-time interface that presents all relevant information in a single view with visual indicators and interactive navigation.

## User Stories

**As a developer**, I want to:
- See all my running database containers in one place
- Monitor CPU, memory, and database metrics simultaneously
- Switch between containers using keyboard shortcuts
- See metric trends over time without external tools

**As a CDC developer**, I want to:
- Watch replication lag in real-time during tests
- Monitor change event rates alongside resource usage
- Stream logs while observing metrics
- Quickly identify which database is causing issues

## Technical Requirements

### Functional Requirements

**FR-1: Multi-Pane Layout**
- Container list (left sidebar, 20% width)
- Resource metrics (top-right, 40% width)
- Database metrics (middle-right, 40% width)
- Logs (bottom, 40% height)
- Status bar (bottom line) showing help and current time

**FR-2: Live Updates**
- Refresh metrics every 1 second
- Smooth transitions (no flickering)
- Highlight changed values
- Show connection status indicators

**FR-3: Interactive Navigation**
- Arrow keys or j/k to navigate container list
- Tab to cycle through panes
- 'q' to quit
- 'r' to refresh immediately
- '?' for help overlay
- 'l' to toggle log streaming
- '+'/'-' to adjust update interval

**FR-4: Visual Elements**
- Color-coded status (green=healthy, yellow=warning, red=error)
- ASCII sparkline charts for time-series metrics
- Progress bars for percentage metrics
- Borders and labels for all panes

**FR-5: Responsive Design**
- Adapt layout to terminal size
- Minimum size: 80x24
- Recommended size: 120x40
- Hide optional panes if terminal too small

### Non-Functional Requirements

**NFR-1: Performance**
- TUI startup time <1 second
- Render frame rate: 30 FPS minimum
- Handle up to 20 containers without lag

**NFR-2: Usability**
- Intuitive keyboard controls
- Clear visual hierarchy
- Helpful error messages
- Graceful degradation on small terminals

## TUI Layout Design

```
┌─────────────────┬──────────────────────────────────────────────────────┐
│ Containers      │ Resource Metrics - simdb-postgres-16-a3f9            │
│                 │                                                      │
│ ► postgres-16   │ CPU:  ████░░░░░░ 42.5% ▲ 2.1%                       │
│   mysql-8       │ Mem:  ███░░░░░░░ 28.3% ▼ 0.5%                       │
│   sqlserver-22  │ Disk: ██░░░░░░░░ 15.2 MB/s                          │
│                 │ Net:  █░░░░░░░░░  1.2 KB/s                           │
│                 │                                                      │
│ [3 containers]  │ CPU (60s):  ▂▃▄▅▄▃▂▃▅▇▆▅▄▃▂                         │
│                 │ Mem (60s):  ▃▃▄▄▄▄▅▅▅▅▅▅▅▄▄                         │
│                 ├──────────────────────────────────────────────────────┤
│                 │ Database Metrics - PostgreSQL 16                     │
│                 │                                                      │
│                 │ Connections:   12 / 100                              │
│                 │ Queries/sec:   245  ▲ 15                             │
│                 │ Transactions:  180/sec (commits: 178, rollbacks: 2) │
│                 │ Cache Hit:     98.5%                                 │
│                 │ Repl Lag:      0 bytes                               │
│                 │                                                      │
├─────────────────┴──────────────────────────────────────────────────────┤
│ Logs - simdb-postgres-16-a3f9                                          │
│                                                                        │
│ 2026-01-22 14:35:21 UTC [1] LOG: database system is ready             │
│ 2026-01-22 14:35:22 UTC [24] LOG: logical replication slot "test"...  │
│ 2026-01-22 14:35:23 UTC [25] LOG: starting logical replication...     │
│                                                                        │
├────────────────────────────────────────────────────────────────────────┤
│ ?: Help  r: Refresh  q: Quit  Tab: Next Pane  ↑↓: Navigate  14:35:24 │
└────────────────────────────────────────────────────────────────────────┘
```

## Implementation Details

### Dependencies

```toml
[dependencies]
ratatui = "0.26"               # Terminal UI framework
crossterm = "0.27"             # Terminal manipulation
tokio = { version = "1.36", features = ["full"] }
```

### TUI Structure

```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
    Terminal,
};

pub struct App {
    containers: Vec<ContainerInfo>,
    selected_container: usize,
    metrics_history: HashMap<String, VecDeque<ResourceMetrics>>,
    show_logs: bool,
    should_quit: bool,
}

impl App {
    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = setup_terminal()?;

        loop {
            terminal.draw(|f| self.ui(f))?;

            if self.should_quit {
                break;
            }

            // Handle input
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    self.handle_key(key);
                }
            }

            // Update metrics
            self.update_metrics().await?;
        }

        restore_terminal()?;
        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(f.size());

        // Render container list
        self.render_container_list(f, chunks[0]);

        // Render metrics and logs
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        self.render_metrics(f, right_chunks[0]);
        self.render_logs(f, right_chunks[1]);
    }
}
```

### Metrics Visualization

```rust
fn render_resource_metrics(&self, f: &mut Frame, area: Rect, metrics: &ResourceMetrics) {
    let block = Block::default()
        .title(" Resource Metrics ")
        .borders(Borders::ALL);

    let cpu_bar = format!(
        "CPU:  {:<10} {:>5.1}%",
        progress_bar(metrics.cpu.usage_percent, 100.0, 10),
        metrics.cpu.usage_percent
    );

    let memory_bar = format!(
        "Mem:  {:<10} {:>5.1}%",
        progress_bar(metrics.memory.usage_percent, 100.0, 10),
        metrics.memory.usage_percent
    );

    let text = vec![
        Line::from(cpu_bar),
        Line::from(memory_bar),
        Line::from(""),
        Line::from("CPU (60s):"),
        Line::from(self.render_sparkline(&metrics_history, |m| m.cpu.usage_percent)),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_sparkline(&self, history: &VecDeque<ResourceMetrics>, extractor: fn(&ResourceMetrics) -> f64) -> String {
    let data: Vec<u64> = history
        .iter()
        .map(|m| extractor(m) as u64)
        .collect();

    let sparkline = Sparkline::default()
        .data(&data)
        .max(100);

    // Render sparkline to string (simplified)
    format!("{:?}", sparkline) // In real implementation, render properly
}

fn progress_bar(value: f64, max: f64, width: usize) -> String {
    let filled = ((value / max) * width as f64) as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
```

## CLI Interface Design

```bash
# Start TUI mode
simdb tui

# Start TUI with specific containers
simdb tui <container-name>...

# Start TUI with custom refresh interval
simdb tui --interval <seconds>
```

## Testing Strategy

### Integration Tests
- `test_tui_startup()`: Verify TUI starts and renders
- `test_keyboard_navigation()`: Test all keyboard shortcuts
- `test_terminal_resize()`: Verify layout adapts to size changes
- `test_metrics_updates()`: Verify live metrics refresh

### Manual Testing
1. **Multi-Container**: Run 5 containers, verify all visible
2. **Navigation**: Test arrow keys, Tab, shortcuts
3. **Resize**: Shrink/expand terminal, verify layout
4. **Long Running**: Run for 1 hour, verify no memory leaks
5. **High Load**: Generate high database load, verify TUI responsive

## Documentation Requirements
- **TUI Guide**: Complete keyboard shortcuts and navigation
- **Layout Reference**: Pane descriptions and metrics displayed
- **Terminal Requirements**: Supported terminals and minimum size

## Future Enhancements
- Mouse support for clickable navigation
- Customizable layouts (user-defined pane arrangements)
- Metric history export from TUI
- Side-by-side container comparison view
- Dark/light theme toggle
