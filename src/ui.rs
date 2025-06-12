use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame, Terminal,
};
use std::collections::VecDeque;
use std::io;
use std::sync::mpsc;
use std::time::Duration;

pub struct App {
    pub bandwidth_data: VecDeque<(f64, f64)>,
    pub current_bandwidth: f64,
    pub max_bandwidth: f64,
    pub interface: String,
    pub filter: String,
    pub should_quit: bool,
    pub tick_count: usize,
}

impl App {
    pub fn new(interface: String, filter: String) -> Self {
        Self {
            bandwidth_data: VecDeque::new(),
            current_bandwidth: 0.0,
            max_bandwidth: 0.0,
            interface,
            filter,
            should_quit: false,
            tick_count: 0,
        }
    }

    pub fn update(&mut self, bandwidth: f64) {
        self.current_bandwidth = bandwidth;
        self.max_bandwidth = self.max_bandwidth.max(bandwidth);
        
        let x = self.tick_count as f64;
        // Convert bytes/s to Mbps: bytes/s * 8 bits/byte / 1,000,000 bits/Mbps
        let mbps = bandwidth * 8.0 / 1_000_000.0;
        self.bandwidth_data.push_back((x, mbps));
        
        if self.bandwidth_data.len() > 100 {
            self.bandwidth_data.pop_front();
        }
        
        self.tick_count += 1;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

pub fn run_ui(
    mut app: App,
    bandwidth_rx: mpsc::Receiver<f64>,
    update_interval: Duration,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_tick = std::time::Instant::now();
    let tick_rate = update_interval;

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.quit();
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if let Ok(bandwidth) = bandwidth_rx.try_recv() {
                app.update(bandwidth);
            }
            last_tick = std::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("TCPGraph", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled(&app.interface, Style::default().fg(Color::Green)),
            Span::raw(" | Filter: "),
            Span::styled(&app.filter, Style::default().fg(Color::Yellow)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Network Monitor"));
    
    f.render_widget(title, chunks[0]);

    let chart_data: Vec<(f64, f64)> = app.bandwidth_data.iter().cloned().collect();
    
    let datasets = vec![Dataset::default()
        .name("Bandwidth (Mbps)")
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Cyan))
        .graph_type(GraphType::Line)
        .data(&chart_data)];

    let x_max = if app.tick_count > 100 {
        app.tick_count as f64
    } else {
        100.0
    };
    let x_min = if app.tick_count > 100 {
        (app.tick_count - 100) as f64
    } else {
        0.0
    };

    // Calculate appropriate y-axis scale with speed buckets
    let current_mbps = app.current_bandwidth * 8.0 / 1_000_000.0;
    let max_mbps = app.max_bandwidth * 8.0 / 1_000_000.0;
    
    // Determine appropriate scale based on current speeds
    let y_max = if max_mbps < 10.0 {
        10.0
    } else if max_mbps < 50.0 {
        50.0
    } else if max_mbps < 100.0 {
        100.0
    } else if max_mbps < 250.0 {
        250.0
    } else if max_mbps < 500.0 {
        500.0
    } else if max_mbps < 1000.0 {
        1000.0
    } else {
        (max_mbps * 1.2).ceil()
    };

    // Create speed bucket labels
    let y_labels = if y_max <= 10.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("2.5", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("5", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("7.5", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("10", Style::default().add_modifier(Modifier::BOLD)),
        ]
    } else if y_max <= 50.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("10", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("25", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("40", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("50", Style::default().add_modifier(Modifier::BOLD)),
        ]
    } else if y_max <= 100.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("25", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("50", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("75", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
        ]
    } else if y_max <= 250.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("50", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("200", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("250", Style::default().add_modifier(Modifier::BOLD)),
        ]
    } else if y_max <= 500.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("250", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("400", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("500", Style::default().add_modifier(Modifier::BOLD)),
        ]
    } else if y_max <= 1000.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("200", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("500", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("750", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled("1000", Style::default().add_modifier(Modifier::BOLD)),
        ]
    } else {
        let step = y_max / 4.0;
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.0}", step), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.0}", step * 2.0), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.0}", step * 3.0), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.0}", y_max), Style::default().add_modifier(Modifier::BOLD)),
        ]
    };

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("Bandwidth Over Time")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([x_min, x_max])
                .labels(vec![
                    Span::styled(format!("{:.0}", x_min), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:.0}", (x_min + x_max) / 2.0), Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:.0}", x_max), Style::default().add_modifier(Modifier::BOLD)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Mbps")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, y_max])
                .labels(y_labels),
        );

    f.render_widget(chart, chunks[1]);

    // Determine speed bucket category
    let speed_category = if current_mbps < 1.0 {
        ("Very Slow", Color::Red)
    } else if current_mbps < 10.0 {
        ("Slow", Color::Yellow)
    } else if current_mbps < 25.0 {
        ("Basic", Color::Blue)
    } else if current_mbps < 50.0 {
        ("Fast", Color::Green)
    } else if current_mbps < 100.0 {
        ("Very Fast", Color::Cyan)
    } else if current_mbps < 250.0 {
        ("Superfast", Color::Magenta)
    } else {
        ("Ultra Fast", Color::White)
    };

    let current_info = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Current: "),
            Span::styled(
                format!("{:.2} Mbps", current_mbps),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ("),
            Span::styled(
                speed_category.0,
                Style::default().fg(speed_category.1).add_modifier(Modifier::BOLD),
            ),
            Span::raw(") | Max: "),
            Span::styled(
                format!("{:.2} Mbps", max_mbps),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | Press 'q' to quit"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Statistics"));
    
    f.render_widget(current_info, chunks[2]);
}