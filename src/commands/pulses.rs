use crate::client::WhatpulseClient;
use crate::commands::TuiPage;
use crate::tui::app::App;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    },
};

inventory::submit! {
    TuiPage {
        title: "Pulses",
        render: render_tui,
        handle_key,
        handle_mouse,
        priority: 15,
    }
}

fn handle_mouse(app: &mut App, event: crossterm::event::MouseEvent) -> bool {
    use crossterm::event::MouseEventKind;
    let len = app.recent_pulses.len();
    if len == 0 {
        return false;
    }

    match event.kind {
        MouseEventKind::ScrollDown => {
            let i = match app.pulses_table_state.borrow().selected() {
                Some(i) => {
                    if i >= len.saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app.pulses_table_state.borrow_mut().select(Some(i));
            true
        }
        MouseEventKind::ScrollUp => {
            let i = match app.pulses_table_state.borrow().selected() {
                Some(i) => {
                    if i == 0 {
                        len.saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app.pulses_table_state.borrow_mut().select(Some(i));
            true
        }
        _ => false,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    let len = app.recent_pulses.len();
    if len == 0 {
        return false;
    }

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            let i = match app.pulses_table_state.borrow().selected() {
                Some(i) => {
                    if i >= len.saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app.pulses_table_state.borrow_mut().select(Some(i));
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let i = match app.pulses_table_state.borrow().selected() {
                Some(i) => {
                    if i == 0 {
                        len.saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app.pulses_table_state.borrow_mut().select(Some(i));
            true
        }
        KeyCode::PageDown => {
            let current = app.pulses_table_state.borrow().selected().unwrap_or(0);
            let next = (current + 10).min(len.saturating_sub(1));
            app.pulses_table_state.borrow_mut().select(Some(next));
            true
        }
        KeyCode::PageUp => {
            let current = app.pulses_table_state.borrow().selected().unwrap_or(0);
            let next = current.saturating_sub(10);
            app.pulses_table_state.borrow_mut().select(Some(next));
            true
        }
        KeyCode::Home => {
            app.pulses_table_state.borrow_mut().select(Some(0));
            true
        }
        KeyCode::End => {
            app.pulses_table_state
                .borrow_mut()
                .select(Some(len.saturating_sub(1)));
            true
        }
        _ => false,
    }
}

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    let pulses = client.get_pulses().await?;
    println!("Found {} pulses:", pulses.len());

    // Already sorted by API usually, but let's ensure it if needed or just take top 5
    // The API returns history, usually newest first.
    for pulse in pulses.iter().take(5) {
        println!(
            "Pulse #{}: {} keys on {}",
            pulse.id,
            pulse.keys.unwrap_or(0),
            pulse.date
        );
    }
    Ok(())
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Recent Pulses ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(err) = &app.pulses_error {
        f.render_widget(
            Paragraph::new(format!("Error: {}", err)).style(Style::default().fg(Color::Red)),
            inner_area,
        );
        return;
    }

    if app.pulses_loading && app.recent_pulses.is_empty() {
        f.render_widget(Paragraph::new("Loading..."), inner_area);
        return;
    }

    if app.recent_pulses.is_empty() {
        if app.client.is_local() {
            let text = vec![
                Line::from(Span::styled(
                    "Pulses history not available in Local Mode.",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from("Reason: No WHATPULSE_API_KEY detected."),
                Line::from("To see pulse history, please provide a valid API key."),
            ];
            f.render_widget(Paragraph::new(text), inner_area);
        } else {
            f.render_widget(Paragraph::new("No recent pulses found."), inner_area);
        }
        return;
    }

    let rows = app.recent_pulses.iter().enumerate().map(|(i, pulse)| {
        let row = Row::new(vec![
            pulse.date.clone(),
            pulse.keys.unwrap_or(0).to_string(),
            pulse.clicks.unwrap_or(0).to_string(),
            format!("{:.2}", pulse.download_mb.unwrap_or(0.0)),
            format!("{:.2}", pulse.upload_mb.unwrap_or(0.0)),
        ]);

        if i % 2 == 1 {
            row.style(Style::default().bg(Color::Rgb(30, 30, 30)))
        } else {
            row
        }
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25), // Date
            Constraint::Percentage(20), // Keys
            Constraint::Percentage(20), // Clicks
            Constraint::Percentage(15), // Download
            Constraint::Percentage(15), // Upload
        ],
    )
    .header(
        Row::new(vec!["Date", "Keys", "Clicks", "DL (MB)", "UL (MB)"])
            .style(Style::default().fg(Color::Yellow)),
    )
    .block(Block::default())
    .row_highlight_style(Style::default().fg(Color::Black).bg(Color::White))
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, inner_area, &mut *app.pulses_table_state.borrow_mut());

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));

    let mut scrollbar_state = ScrollbarState::new(app.recent_pulses.len())
        .position(app.pulses_table_state.borrow().selected().unwrap_or(0));

    f.render_stateful_widget(scrollbar, inner_area, &mut scrollbar_state);
}
