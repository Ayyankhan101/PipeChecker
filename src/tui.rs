//! Interactive Terminal UI for pipechecker
//!
//! Provides a ratatui-based interface for browsing audit results
//! across multiple workflow files with keyboard navigation.
//!
//! # Keyboard shortcuts
//! - `↑/↓` or `j/k`: Navigate between files
//! - `Enter/Space`: Toggle detail view
//! - `q/Esc`: Quit

use crate::{
    audit_file, discover_workflows, AuditOptions, AuditResult, DiscoveryOptions, Severity,
};
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
use std::{io, path::Path};

/// Application state for the TUI
pub struct App {
    files: Vec<String>,
    results: Vec<Option<AuditResult>>,
    selected: usize,
    show_details: bool,
}

impl App {
    /// Create a new TUI application instance
    ///
    /// Discovers workflow files and runs initial audits
    pub fn new(options: AuditOptions) -> io::Result<Self> {
        let files = discover_workflows(Path::new("."), &DiscoveryOptions::default());

        let mut results = Vec::new();
        for file in &files {
            let opts = AuditOptions {
                check_docker_images: options.check_docker_images,
                strict_mode: options.strict_mode,
                rules: options.rules,
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

/// Run the interactive TUI mode
///
/// Sets up terminal in raw mode with alternate screen,
/// handles keyboard events, and renders the UI.
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
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
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
    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let result = &app.results[i];
            let (status, errors, warnings) = if let Some(r) = result {
                let e = r
                    .issues
                    .iter()
                    .filter(|i| i.severity == Severity::Error)
                    .count();
                let w = r
                    .issues
                    .iter()
                    .filter(|i| i.severity == Severity::Warning)
                    .count();
                let status = if e > 0 {
                    "❌"
                } else if w > 0 {
                    "⚠️ "
                } else {
                    "✅"
                };
                (status, e, w)
            } else {
                ("❓", 0, 0)
            };

            let filename = Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file);

            let line = Line::from(vec![
                Span::raw(status),
                Span::raw(" "),
                Span::styled(filename, Style::default().fg(Color::White)),
                Span::raw(" │ "),
                Span::styled(
                    format!("{} errors", errors),
                    if errors > 0 {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ),
                Span::raw(" │ "),
                Span::styled(
                    format!("{} warnings", warnings),
                    if warnings > 0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Workflows"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
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
            Line::from(vec![Span::styled(
                &r.summary,
                Style::default().fg(Color::Cyan),
            )]),
            Line::from(""),
        ];

        if r.issues.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "✅ No issues found!",
                Style::default().fg(Color::Green),
            )]));
        } else {
            for issue in &r.issues {
                let (color, prefix) = match issue.severity {
                    Severity::Error => (Color::Red, "❌ ERROR: "),
                    Severity::Warning => (Color::Yellow, "⚠️  WARNING: "),
                    Severity::Info => (Color::Blue, "ℹ️  INFO: "),
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        prefix,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
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
        vec![Line::from(vec![Span::styled(
            "❌ Failed to audit file",
            Style::default().fg(Color::Red),
        )])]
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AuditResult, Issue, Location, Provider, Severity};

    fn make_app(files: Vec<&str>, results: Vec<Option<AuditResult>>) -> App {
        App {
            files: files.into_iter().map(String::from).collect(),
            results,
            selected: 0,
            show_details: false,
        }
    }

    fn make_result(issues: Vec<Issue>) -> AuditResult {
        AuditResult {
            provider: Provider::GitHubActions,
            issues,
            summary: format!("0 errors, 0 warnings"),
            elapsed: std::time::Duration::from_millis(0),
        }
    }

    #[test]
    fn test_app_navigation_next() {
        let mut app = make_app(vec!["a.yml", "b.yml", "c.yml"], vec![None, None, None]);

        assert_eq!(app.selected, 0);
        app.next();
        assert_eq!(app.selected, 1);
        app.next();
        assert_eq!(app.selected, 2);
        // Should not go past the last item
        app.next();
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn test_app_navigation_previous() {
        let mut app = make_app(vec!["a.yml", "b.yml", "c.yml"], vec![None, None, None]);
        app.selected = 2;

        app.previous();
        assert_eq!(app.selected, 1);
        app.previous();
        assert_eq!(app.selected, 0);
        // Should not go below 0
        app.previous();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_app_navigation_single_item() {
        let mut app = make_app(vec!["single.yml"], vec![None]);

        app.next();
        assert_eq!(app.selected, 0);
        app.previous();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_app_toggle_details() {
        let mut app = make_app(vec!["a.yml"], vec![None]);

        assert!(!app.show_details);
        app.toggle_details();
        assert!(app.show_details);
        app.toggle_details();
        assert!(!app.show_details);
    }

    #[test]
    fn test_app_navigation_with_results() {
        let issues = vec![Issue::new(
            Severity::Error,
            "test error",
            Some("fix it".to_string()),
        )];
        let result = make_result(issues);
        let mut app = make_app(vec!["ci.yml"], vec![Some(result)]);

        app.next();
        assert_eq!(app.selected, 0); // only one item
        app.toggle_details();
        assert!(app.show_details);
    }

    #[test]
    fn test_app_navigation_empty_files() {
        let mut app = make_app(vec![], vec![]);

        // Navigation on empty list should not crash
        app.next();
        assert_eq!(app.selected, 0);
        app.previous();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_app_navigation_many_files() {
        let file_names: Vec<String> = (1..=10).map(|i| format!("job-{}.yml", i)).collect();
        let files: Vec<&str> = file_names.iter().map(|s| s.as_str()).collect();
        let results: Vec<Option<AuditResult>> = (1..=10)
            .map(|i| {
                Some(make_result(vec![Issue::new(
                    Severity::Warning,
                    &format!("warning {}", i),
                    None,
                )]))
            })
            .collect();

        let mut app = make_app(files, results);

        // Navigate all the way down
        for _ in 0..20 {
            app.next();
        }
        assert_eq!(app.selected, 9);

        // Navigate all the way up
        for _ in 0..20 {
            app.previous();
        }
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_app_mixed_severities_display() {
        let issues = vec![
            Issue::new(Severity::Error, "error msg", Some("fix error".to_string())),
            Issue::new(Severity::Warning, "warning msg", None),
            Issue::new(Severity::Info, "info msg", Some("hint".to_string())),
        ];
        let result = make_result(issues);
        let app = make_app(vec!["mixed.yml"], vec![Some(result)]);

        assert_eq!(app.files.len(), 1);
        assert_eq!(app.results.len(), 1);
        if let Some(r) = &app.results[0] {
            assert_eq!(r.issues.len(), 3);
        }
    }

    #[test]
    fn test_app_with_failed_audit_result() {
        // App can hold None in results when audit fails
        let app = make_app(vec!["bad.yml"], vec![None]);
        assert_eq!(app.files.len(), 1);
        assert!(app.results[0].is_none());
    }

    #[test]
    fn test_app_with_location_info() {
        let issue = Issue {
            severity: Severity::Error,
            message: "syntax error".to_string(),
            location: Some(Location {
                line: 10,
                column: 5,
                job: Some("build".to_string()),
            }),
            suggestion: Some("fix the syntax".to_string()),
        };
        let result = make_result(vec![issue]);
        let app = make_app(vec!["loc.yml"], vec![Some(result)]);

        if let Some(Some(r)) = app.results.get(0) {
            assert_eq!(r.issues[0].location.as_ref().unwrap().line, 10);
        }
    }
}
