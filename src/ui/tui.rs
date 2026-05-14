use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, TableState, List, ListItem},
    layout::{Layout, Constraint, Direction},
    style::{Style, Modifier, Color},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use crate::models::{Dependency, HealthScore};

pub struct App {
    pub items: Vec<(Dependency, HealthScore)>,
    pub state: TableState,
}

pub fn run_tui(items: Vec<(Dependency, HealthScore)>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TableState::default();
    state.select(Some(0));
    let mut app = App { items, state };

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

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => {
                    let i = match app.state.selected() {
                        Some(i) => {
                            if i >= app.items.len() - 1 { 0 } else { i + 1 }
                        }
                        None => 0,
                    };
                    app.state.select(Some(i));
                }
                KeyCode::Up => {
                    let i = match app.state.selected() {
                        Some(i) => {
                            if i == 0 { app.items.len() - 1 } else { i - 1 }
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

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ].as_ref())
        .split(f.size());

    // Top Header
    let title = Paragraph::new("👁️ Vigil - Universal Supply Chain Health Dashboard")
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(title, chunks[0]);

    // Main Body: Table (Left) and Details (Right)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ].as_ref())
        .split(chunks[1]);

    // Table view
    let rows: Vec<Row> = app.items.iter().map(|(dep, score)| {
        let color = if score.composite_score > 80 {
            Color::Green
        } else if score.composite_score > 50 {
            Color::Yellow
        } else {
            Color::Red
        };

        Row::new(vec![
            Cell::from(dep.name.clone()),
            Cell::from(dep.version.clone()),
            Cell::from(score.composite_score.to_string()).style(Style::default().fg(color)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(60),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
    ])
    .header(Row::new(vec!["Dependency", "Version", "Score"])
        .style(Style::default().add_modifier(Modifier::BOLD)))
    .block(Block::default().borders(Borders::ALL).title("Inventory"))
    .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, body_chunks[0], &mut app.state);

    // Details view (Right)
    if let Some(selected_idx) = app.state.selected() {
        if let Some((dep, score)) = app.items.get(selected_idx) {
            let details_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(6), // Health Summary
                    Constraint::Min(0),     // Advisories
                ].as_ref())
                .split(body_chunks[1]);

            // Health Summary Panel
            let mut health_text = vec![
                format!("Ecosystem: {:?}", dep.ecosystem),
                format!("Vitality Score: {}/100", score.composite_score),
                "".to_string(),
                "Maintenance Signals:".to_string(),
            ];
            for signal in &score.maintenance_details {
                health_text.push(format!(" • {}", signal));
            }

            let health_panel = Paragraph::new(health_text.join("\n"))
                .block(Block::default().borders(Borders::ALL).title("Health Vitality"));
            f.render_widget(health_panel, details_chunks[0]);

            // Advisories Panel
            let advisory_items: Vec<ListItem> = if dep.advisories.is_empty() {
                vec![ListItem::new("✅ No known vulnerabilities found.")]
            } else {
                dep.advisories.iter().map(|adv| {
                    ListItem::new(vec![
                        ratatui::text::Line::from(format!("ID: {}", adv.id)).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        ratatui::text::Line::from(format!("Severity: {}", adv.severity)),
                        ratatui::text::Line::from(adv.summary.clone()),
                        ratatui::text::Line::from("─".repeat(20)),
                    ])
                }).collect()
            };

            let advisories_list = List::new(advisory_items)
                .block(Block::default().borders(Borders::ALL).title("Security Advisories"));
            f.render_widget(advisories_list, details_chunks[1]);
        }
    }

    // Footer
    let footer = Paragraph::new("Use ↑/↓ to navigate, 'q' to quit")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
