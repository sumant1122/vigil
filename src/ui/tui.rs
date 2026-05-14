use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, TableState},
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

    let title = Paragraph::new("👁️ Vigil - Universal Supply Chain Health Dashboard")
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(title, chunks[0]);

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
            Cell::from(format!("{:?}", dep.ecosystem)),
            Cell::from(score.composite_score.to_string()).style(Style::default().fg(color)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(Row::new(vec!["Dependency", "Version", "Ecosystem", "Score"])
        .style(Style::default().add_modifier(Modifier::BOLD)))
    .block(Block::default().borders(Borders::ALL).title("Dependencies"))
    .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, chunks[1], &mut app.state);

    let footer = Paragraph::new("Use ↑/↓ to navigate, 'q' to quit")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
