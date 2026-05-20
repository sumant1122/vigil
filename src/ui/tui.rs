use crate::models::{Dependency, HealthScore};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState},
    Terminal,
};
use std::io;

pub struct App {
    pub items: Vec<(Dependency, HealthScore)>,
    pub state: TableState,
    pub search_query: String,
    pub in_search_mode: bool,
}

pub fn run_tui(items: Vec<(Dependency, HealthScore)>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TableState::default();
    state.select(Some(0));
    let mut app = App {
        items,
        state,
        search_query: String::new(),
        in_search_mode: false,
    };

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            let filtered_count = app
                .items
                .iter()
                .filter(|(dep, _)| {
                    app.search_query.is_empty()
                        || dep
                            .name
                            .to_lowercase()
                            .contains(&app.search_query.to_lowercase())
                })
                .count();

            if app.in_search_mode {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        app.in_search_mode = false;
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.state.select(Some(0));
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.state.select(Some(0));
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('/') => {
                        app.in_search_mode = true;
                    }
                    KeyCode::Down if filtered_count > 0 => {
                        let i = match app.state.selected() {
                            Some(i) => {
                                if i >= filtered_count - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.state.select(Some(i));
                    }
                    KeyCode::Up if filtered_count > 0 => {
                        let i = match app.state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    filtered_count - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.state.select(Some(i));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Length(3), // Stats
                Constraint::Min(0),    // Body
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(f.size());

    // Top Header
    let title = Paragraph::new("👁️ Vigil - Universal Supply Chain Health Dashboard")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title("Vigil v0.1.0"));
    f.render_widget(title, chunks[0]);

    // Stats Bar
    let total_deps = app.items.len();
    let vulnerable_deps = app
        .items
        .iter()
        .filter(|(d, _)| !d.advisories.is_empty())
        .count();
    let avg_score = if total_deps > 0 {
        app.items
            .iter()
            .map(|(_, s)| s.composite_score as u32)
            .sum::<u32>()
            / total_deps as u32
    } else {
        0
    };

    let stats_text = format!(
        " 📦 Total: {} | 🛡️ Vulnerable: {} | 📈 Avg Vitality: {}/100 ",
        total_deps, vulnerable_deps, avg_score
    );
    let stats =
        Paragraph::new(stats_text).block(Block::default().borders(Borders::ALL).title("Summary"));
    f.render_widget(stats, chunks[1]);

    // Main Body: Table (Left) and Details (Right)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
        .split(chunks[2]);

    // Filter and clamp state
    let filtered_items: Vec<&(Dependency, HealthScore)> = app
        .items
        .iter()
        .filter(|(dep, _)| {
            app.search_query.is_empty()
                || dep
                    .name
                    .to_lowercase()
                    .contains(&app.search_query.to_lowercase())
        })
        .collect();

    if !filtered_items.is_empty() {
        if let Some(selected) = app.state.selected() {
            if selected >= filtered_items.len() {
                app.state.select(Some(filtered_items.len() - 1));
            }
        } else {
            app.state.select(Some(0));
        }
    } else {
        app.state.select(None);
    }

    // Table view
    let rows: Vec<Row> = filtered_items
        .iter()
        .map(|(dep, score)| {
            let color = if score.composite_score > 80 {
                Color::Green
            } else if score.composite_score > 50 {
                Color::Yellow
            } else {
                Color::Red
            };

            let ecosystem_indicator = match dep.ecosystem {
                crate::models::Ecosystem::Cargo => "🦀 Cargo",
                crate::models::Ecosystem::Npm => " NPM",
                crate::models::Ecosystem::Pip => "🐍 PyPI",
                crate::models::Ecosystem::Go => "🐹 Go",
            };

            Row::new(vec![
                Cell::from(ecosystem_indicator),
                Cell::from(dep.name.clone()),
                Cell::from(dep.version.clone()),
                Cell::from(score.composite_score.to_string()).style(Style::default().fg(color)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
        ],
    )
    .header(
        Row::new(vec!["Type", "Dependency", "Version", "Score"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title("Inventory"))
    .highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray),
    )
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, body_chunks[0], &mut app.state);

    // Details view (Right)
    if let Some(selected_idx) = app.state.selected() {
        if let Some(item) = filtered_items.get(selected_idx) {
            let (dep, score) = *item;
            let details_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(10), // Health Summary
                        Constraint::Min(0),     // Advisories
                    ]
                    .as_ref(),
                )
                .split(body_chunks[1]);

            // Health Summary Panel
            let license = dep.license.as_deref().unwrap_or("Unknown");
            let security_status = if dep.advisories.is_empty() {
                "🛡️ Secure".to_string()
            } else {
                format!("⚠️ Vulnerable ({} advisories)", dep.advisories.len())
            };

            let mut health_text = vec![
                format!("Ecosystem:   {:?}", dep.ecosystem),
                format!("License:     {}", license),
                format!("Security:    {}", security_status),
                format!(
                    "Dependencies: {} direct / {} transitive",
                    dep.direct_dependencies.len(),
                    score.bloat_index
                ),
                format!("Vitality Score: {}/100", score.composite_score),
                "".to_string(),
                "Maintenance Signals:".to_string(),
            ];
            for signal in &score.maintenance_details {
                health_text.push(format!(" • {}", signal));
            }

            let health_panel = Paragraph::new(health_text.join("\n")).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Health Vitality"),
            );
            f.render_widget(health_panel, details_chunks[0]);

            // Advisories Panel
            let advisory_items: Vec<ListItem> = if dep.advisories.is_empty() {
                vec![ListItem::new("✅ No known vulnerabilities found.")]
            } else {
                dep.advisories
                    .iter()
                    .map(|adv| {
                        ListItem::new(vec![
                            ratatui::text::Line::from(format!("ID: {}", adv.id)).style(
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                            ),
                            ratatui::text::Line::from(format!("Severity: {}", adv.severity)),
                            ratatui::text::Line::from(adv.summary.clone()),
                            ratatui::text::Line::from("─".repeat(20)),
                        ])
                    })
                    .collect()
            };

            let advisories_list = List::new(advisory_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Security Advisories"),
            );
            f.render_widget(advisories_list, details_chunks[1]);
        }
    }

    // Footer
    let footer = if app.in_search_mode {
        let footer_text = format!(" 🔍 Search Dependency: {}▋ ", app.search_query);
        Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search Mode (Esc/Enter to exit)"),
            )
    } else {
        Paragraph::new(" Use ↑/↓ to navigate, '/' to search, 'q' to quit ")
            .block(Block::default().borders(Borders::ALL).title("Controls"))
    };
    f.render_widget(footer, chunks[3]);
}
