use crate::{audit_file, AuditOptions, AuditResult, Severity};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::{fs, io, path::Path};

pub struct App {
    files: Vec<String>,
    results: Vec<Option<AuditResult>>,
    selected: usize,
    show_details: bool,
}

impl App {
    pub fn new(options: AuditOptions) -> io::Result<Self> {
        let mut files = Vec::new();
        
        if let Ok(entries) = fs::read_dir(".github/workflows") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yml") 
                    || path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
        
        if Path::new(".gitlab-ci.yml").exists() {
            files.push(".gitlab-ci.yml".to_string());
        }
        
        if Path::new(".circleci/config.yml").exists() {
            files.push(".circleci/config.yml".to_string());
        }
        
        let mut results = Vec::new();
        for file in &files {
            let opts = AuditOptions {
                check_docker_images: options.check_docker_images,
                strict_mode: options.strict_mode,
            };
            let result = audit_file(file, opts).ok();
            results.push(result);
        }
        
        Ok(App {
            files,
            results,
            selected: 0,
            show_details: false,
        })
    }
    
    fn next(&mut self) {
        if self.selected < self.files.len().saturating_sub(1) {
            self.selected += 1;
        }
    }
    
    fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
    
    fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
}

pub fn run_tui(options: AuditOptions) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new(options)?;
    let res = run_app(&mut terminal, &mut app);
    
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    res
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => app.toggle_details(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());
    
    // Header
    let header = Paragraph::new("🔍 Pipecheck - Interactive Mode")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);
    
    // Main content
    if app.show_details && app.selected < app.results.len() {
        render_details(f, chunks[1], app);
    } else {
        render_list(f, chunks[1], app);
    }
    
    // Footer
    let footer_text = if app.show_details {
        "[↑/↓] Navigate  [Enter/Space] Back  [Q/Esc] Quit"
    } else {
        "[↑/↓] Navigate  [Enter/Space] Details  [Q/Esc] Quit"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn render_list(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.files.iter().enumerate().map(|(i, file)| {
        let result = &app.results[i];
        let (status, errors, warnings) = if let Some(r) = result {
            let e = r.issues.iter().filter(|i| i.severity == Severity::Error).count();
            let w = r.issues.iter().filter(|i| i.severity == Severity::Warning).count();
            let status = if e > 0 { "❌" } else if w > 0 { "⚠️ " } else { "✅" };
            (status, e, w)
        } else {
            ("❓", 0, 0)
        };
        
        let filename = Path::new(file).file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file);
        
        let line = Line::from(vec![
            Span::raw(status),
            Span::raw(" "),
            Span::styled(filename, Style::default().fg(Color::White)),
            Span::raw(" │ "),
            Span::styled(format!("{} errors", errors), 
                if errors > 0 { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Green) }),
            Span::raw(" │ "),
            Span::styled(format!("{} warnings", warnings),
                if warnings > 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Green) }),
        ]);
        
        ListItem::new(line)
    }).collect();
    
    let mut state = ListState::default();
    state.select(Some(app.selected));
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Workflows"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    
    f.render_stateful_widget(list, area, &mut state);
}

fn render_details(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let result = &app.results[app.selected];
    
    let text = if let Some(r) = result {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&app.files[app.selected]),
            ]),
            Line::from(vec![
                Span::styled("Provider: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:?}", r.provider)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(&r.summary, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
        ];
        
        if r.issues.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("✅ No issues found!", Style::default().fg(Color::Green)),
            ]));
        } else {
            for issue in &r.issues {
                let (color, prefix) = match issue.severity {
                    Severity::Error => (Color::Red, "❌ ERROR: "),
                    Severity::Warning => (Color::Yellow, "⚠️  WARNING: "),
                    Severity::Info => (Color::Blue, "ℹ️  INFO: "),
                };
                
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw(&issue.message),
                ]));
                
                if let Some(suggestion) = &issue.suggestion {
                    lines.push(Line::from(vec![
                        Span::raw("   💡 "),
                        Span::styled(suggestion, Style::default().fg(Color::Cyan)),
                    ]));
                }
                lines.push(Line::from(""));
            }
        }
        
        lines
    } else {
        vec![Line::from(vec![
            Span::styled("❌ Failed to audit file", Style::default().fg(Color::Red)),
        ])]
    };
    
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}
