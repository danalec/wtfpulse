use crate::client::{PulseResponse, WhatpulseClient};
use crate::commands::TuiPage;
use crate::tui::app::App;
use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use std::collections::HashMap;

inventory::submit! {
    TuiPage {
        title: "Pulses",
        render: render_tui,
        handle_key,
        priority: 15,
    }
}

fn handle_key(_app: &mut App, _key: KeyEvent) -> bool {
    false
}

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    let pulses_map = client
        .get_resource::<HashMap<String, PulseResponse>>("pulses")
        .await?;
    println!("Found {} pulses:", pulses_map.len());

    // Convert to vector and sort by key (Pulse ID) descending to show newest first
    let mut pulses: Vec<_> = pulses_map.into_iter().collect();
    // Pulse IDs are strings like "Pulse-123", so string sort works reasonably well for ordering
    pulses.sort_by(|a, b| b.0.cmp(&a.0));

    for (id, pulse) in pulses.iter().take(5) {
        println!(
            "{}: {} keys on {}",
            id,
            pulse.keys.as_deref().unwrap_or("0"),
            pulse.date.as_deref().unwrap_or("unknown date")
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

    let rows: Vec<Row> = app
        .recent_pulses
        .iter()
        .map(|pulse| {
            Row::new(vec![
                pulse.date.as_deref().unwrap_or("Unknown").to_string(),
                pulse.keys.as_deref().unwrap_or("0").to_string(),
                pulse.clicks.as_deref().unwrap_or("0").to_string(),
                pulse.download_mb.as_deref().unwrap_or("0").to_string(),
                pulse.upload_mb.as_deref().unwrap_or("0").to_string(),
            ])
        })
        .collect();

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
    .block(Block::default());

    f.render_widget(table, inner_area);
}
