use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use serde_json::json;

/// A result to display in the TUI
#[derive(Clone)]
pub struct TuiResult {
    pub url: String,
    pub status_code: u16,
    pub content_length: u64,
    pub redirect_location: Option<String>,
    pub content_type: Option<String>,
    pub server: Option<String>,
    pub duration_ms: u64,
}

pub struct TuiState {
    pub results: Vec<TuiResult>,
    pub total: usize,
    pub scanned: usize,
    pub found: usize,
    pub errors: usize,
    pub start_time: Instant,
    pub mode: String,
    pub target: String,
    pub wordlist: String,
    pub threads: usize,
    pub scan_complete: bool,
    pub scroll_offset: usize,
}

impl TuiState {
    pub fn new(mode: String, target: String, wordlist: String, threads: usize, total: usize) -> Self {
        Self {
            results: Vec::new(),
            total,
            scanned: 0,
            found: 0,
            errors: 0,
            start_time: Instant::now(),
            mode,
            target,
            wordlist,
            threads,
            scan_complete: false,
            scroll_offset: 0,
        }
    }

    pub fn add_result(&mut self, result: TuiResult) {
        self.found += 1;
        self.results.push(result);
    }

    pub fn increment_scanned(&mut self) {
        self.scanned += 1;
    }

    pub fn increment_errors(&mut self) {
        self.errors += 1;
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn speed(&self) -> f64 {
        let elapsed_secs = self.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 {
            self.scanned as f64 / elapsed_secs
        } else {
            0.0
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self, max_visible: usize) {
        if self.scroll_offset + max_visible < self.results.len() {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self, max_visible: usize) {
        if self.results.len() > max_visible {
            self.scroll_offset = self.results.len() - max_visible;
        } else {
            self.scroll_offset = 0;
        }
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: Arc<Mutex<TuiState>>,
}

impl Tui {
    pub fn new(state: Arc<Mutex<TuiState>>) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal, state })
    }

    pub fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let state = self.state.lock().unwrap();
            render_ui(f, &state);
        })?;
        Ok(())
    }

    pub async fn run(&mut self, mut rx: mpsc::Receiver<TuiMessage>) -> Result<()> {
        let mut scan_finished = false;
        let mut last_draw = Instant::now();
        
        loop {
            if let Err(e) = self.draw() {
                eprintln!("[TUI Error] Failed to draw: {}", e);
                continue;
            }

            match event::poll(Duration::from_millis(50)) {
                Ok(true) => {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => break,
                                KeyCode::Up | KeyCode::Char('k') => {
                                    let mut state = self.state.lock().unwrap();
                                    state.scroll_up();
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let mut state = self.state.lock().unwrap();
                                    let max_visible = 20; // Approximate visible items
                                    state.scroll_down(max_visible);
                                }
                                KeyCode::Home | KeyCode::Char('g') => {
                                    let mut state = self.state.lock().unwrap();
                                    state.scroll_to_top();
                                }
                                KeyCode::End | KeyCode::Char('G') => {
                                    let mut state = self.state.lock().unwrap();
                                    let max_visible = 20;
                                    state.scroll_to_bottom(max_visible);
                                }
                                KeyCode::PageUp => {
                                    let mut state = self.state.lock().unwrap();
                                    for _ in 0..10 {
                                        state.scroll_up();
                                    }
                                }
                                KeyCode::PageDown => {
                                    let mut state = self.state.lock().unwrap();
                                    let max_visible = 20;
                                    for _ in 0..10 {
                                        state.scroll_down(max_visible);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(_) => continue,
                        _ => {}
                    }
                }
                Ok(false) => {}
                Err(_) => continue,
            }

            let mut messages_processed = 0;
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        let mut state = self.state.lock().unwrap();
                        match msg {
                            TuiMessage::Result(result) => state.add_result(result),
                            TuiMessage::Scanned => state.increment_scanned(),
                            TuiMessage::Error => state.increment_errors(),
                            TuiMessage::Done => {
                                state.scan_complete = true;
                                scan_finished = true;
                            }
                        }
                        drop(state);
                        messages_processed += 1;
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        break;
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        if !scan_finished {
                            let mut state = self.state.lock().unwrap();
                            state.scan_complete = true;
                            scan_finished = true;
                        }
                        break;
                    }
                }
            }

            if messages_processed > 0 || last_draw.elapsed() > Duration::from_millis(100) {
                let _ = self.draw();
                last_draw = Instant::now();
            }
        }

        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

pub enum TuiMessage {
    Result(TuiResult),
    Scanned,
    Error,
    Done,
}

fn render_ui(f: &mut Frame, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(10),    // Results
            Constraint::Length(5),  // Progress & Stats
        ])
        .split(f.area());

    render_header(f, chunks[0], state);
    render_results(f, chunks[1], state);
    render_footer(f, chunks[2], state);
}

fn render_header(f: &mut Frame, area: Rect, state: &TuiState) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("Rustbuster ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("v0.1.0", Style::default().fg(Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("Mode: ", Style::default().fg(Color::Yellow)),
            Span::raw(&state.mode),
            Span::raw("  |  "),
            Span::styled("Target: ", Style::default().fg(Color::Yellow)),
            Span::raw(&state.target),
        ]),
        Line::from(vec![
            Span::styled("Wordlist: ", Style::default().fg(Color::Yellow)),
            Span::raw(&state.wordlist),
            Span::raw("  |  "),
            Span::styled("Threads: ", Style::default().fg(Color::Yellow)),
            Span::raw(state.threads.to_string()),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Info").style(Style::default().fg(Color::Cyan)));

    f.render_widget(header, area);
}

fn render_results(f: &mut Frame, area: Rect, state: &TuiState) {
    let max_visible = area.height.saturating_sub(2) as usize;
    let total_results = state.results.len();
    
    let start_idx = state.scroll_offset;
    let end_idx = (start_idx + max_visible).min(total_results);
    
    let results: Vec<ListItem> = state
        .results
        .iter()
        .skip(start_idx)
        .take(max_visible)
        .map(|result| {
            let status_color = match result.status_code {
                200..=299 => Color::Green,
                300..=399 => Color::Yellow,
                400..=499 => Color::Red,
                500..=599 => Color::Magenta,
                _ => Color::White,
            };
            
            let status_text = match result.status_code {
                200 => "OK", 201 => "Created", 204 => "No Content",
                301 => "Moved", 302 => "Found", 307 => "Redirect",
                401 => "Unauthorized", 403 => "Forbidden", 404 => "Not Found",
                500 => "Error", 502 => "Bad Gateway", 503 => "Unavailable",
                _ => "",
            };

            let mut line_spans = vec![
                Span::styled(
                    format!("[{} {}] ", result.status_code, status_text),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{} ", result.url)),
                Span::styled(
                    format!("[{}B]", result.content_length),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!(" [{}ms]", result.duration_ms),
                    Style::default().fg(Color::Magenta),
                ),
            ];
            
            if let Some(content_type) = &result.content_type {
                line_spans.push(Span::styled(
                    format!(" [{}]", content_type),
                    Style::default().fg(Color::Cyan),
                ));
            }

            if let Some(location) = &result.redirect_location {
                line_spans.push(Span::styled(
                    format!(" -> {}", location),
                    Style::default().fg(Color::Blue),
                ));
            }

            ListItem::new(Line::from(line_spans))
        })
        .collect();

    let title = if total_results > max_visible {
        format!(
            "Results (Found: {}) - Showing {}-{} of {} [↑↓ to scroll, g/G for top/bottom]",
            state.found,
            start_idx + 1,
            end_idx,
            total_results
        )
    } else {
        format!("Results (Found: {})", state.found)
    };

    let results_list = List::new(results)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(results_list, area);
}

fn render_footer(f: &mut Frame, area: Rect, state: &TuiState) {
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(area);

    let progress = if state.total > 0 {
        ((state.scanned as f64 / state.total as f64) * 100.0).min(100.0)
    } else {
        0.0
    };

    let progress_title = if state.scan_complete {
        "Progress - COMPLETE ✓"
    } else {
        "Progress - Scanning..."
    };

    let progress_label = if state.total > 0 {
        format!("{:.1}% ({}/{})", progress, state.scanned, state.total)
    } else {
        "Calculating...".to_string()
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(progress_title))
        .gauge_style(
            if state.scan_complete {
                Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).bg(Color::Black).add_modifier(Modifier::BOLD)
            }
        )
        .percent(progress as u16)
        .label(progress_label);

    f.render_widget(gauge, footer_chunks[0]);

    let elapsed = state.elapsed();
    let elapsed_str = format!("{:02}:{:02}", elapsed.as_secs() / 60, elapsed.as_secs() % 60);
    
    let stats_text = if state.scan_complete {
        vec![
            Line::from(vec![
                Span::styled("Scan Complete! ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Found: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}", state.found)),
                Span::raw("  |  "),
                Span::styled("Elapsed: ", Style::default().fg(Color::Yellow)),
                Span::raw(elapsed_str),
                Span::raw("  |  "),
                Span::styled("Errors: ", Style::default().fg(Color::Yellow)),
                Span::raw(state.errors.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("'q'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" or ", Style::default().fg(Color::Gray)),
                Span::styled("'ESC'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to quit", Style::default().fg(Color::Gray)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Speed: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:.1} req/s", state.speed())),
                Span::raw("  |  "),
                Span::styled("Elapsed: ", Style::default().fg(Color::Yellow)),
                Span::raw(elapsed_str),
                Span::raw("  |  "),
                Span::styled("Errors: ", Style::default().fg(Color::Yellow)),
                Span::raw(state.errors.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("'q'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" or ", Style::default().fg(Color::Gray)),
                Span::styled("'ESC'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to quit", Style::default().fg(Color::Gray)),
            ]),
        ]
    };

    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Stats").style(Style::default().fg(Color::Cyan)));

    f.render_widget(stats, footer_chunks[1]);
}

pub async fn run_tui_mode<F, Fut>(
    mode: String,
    target: String,
    wordlist: String,
    threads: usize,
    total: usize,
    output_file: Option<String>,
    output_format: String,
    scan_fn: F,
) -> Result<()>
where
    F: FnOnce(mpsc::Sender<TuiMessage>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send + 'static,
{
    let (tx, rx) = mpsc::channel(100);
    
    let state = Arc::new(Mutex::new(TuiState::new(
        mode,
        target,
        wordlist,
        threads,
        total,
    )));
    
    let mut tui = Tui::new(Arc::clone(&state))?;
    
    let scan_handle = tokio::spawn(async move {
        scan_fn(tx).await
    });
    
    let tui_result = tui.run(rx).await;
    
    let _ = scan_handle.await;
    
    if let Some(output_path) = output_file {
        let state = state.lock().unwrap();
        write_results_to_file(&state.results, &output_path, &output_format)?;
        drop(state);
        
        println!("\nResults saved to: {}", output_path);
    }
    
    tui_result
}

fn write_results_to_file(results: &[TuiResult], file_path: &str, format: &str) -> Result<()> {
    match format {
        "json" => write_json_results(results, file_path),
        "csv" => write_csv_results(results, file_path),
        _ => write_plain_results(results, file_path),
    }
}

fn write_plain_results(results: &[TuiResult], file_path: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)?;

    for result in results {
        let line = if let Some(location) = &result.redirect_location {
            format!(
                "{} [{}] [{}B] [{}ms] -> {}\n",
                result.url, result.status_code, result.content_length, result.duration_ms, location
            )
        } else {
            format!(
                "{} [{}] [{}B] [{}ms]\n",
                result.url, result.status_code, result.content_length, result.duration_ms
            )
        };
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

fn write_json_results(results: &[TuiResult], file_path: &str) -> Result<()> {
    let json_results: Vec<_> = results
        .iter()
        .map(|r| {
            json!({
                "url": r.url,
                "status_code": r.status_code,
                "content_length": r.content_length,
                "duration_ms": r.duration_ms,
                "redirect_location": r.redirect_location,
                "content_type": r.content_type,
                "server": r.server,
            })
        })
        .collect();

    let json_output = serde_json::to_string_pretty(&json_results)?;
    std::fs::write(file_path, json_output)?;
    Ok(())
}

fn write_csv_results(results: &[TuiResult], file_path: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)?;

    writeln!(file, "URL,Status Code,Content Length,Duration (ms),Redirect Location,Content Type,Server")?;

    for result in results {
        writeln!(
            file,
            "{},{},{},{},{},{},{}",
            result.url,
            result.status_code,
            result.content_length,
            result.duration_ms,
            result.redirect_location.as_deref().unwrap_or(""),
            result.content_type.as_deref().unwrap_or(""),
            result.server.as_deref().unwrap_or(""),
        )?;
    }

    Ok(())
}
