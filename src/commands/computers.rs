use crate::client::WhatpulseClient;
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

inventory::submit! {
    TuiPage {
        title: "Computers",
        render: render_tui,
        handle_key,
        priority: 10,
    }
}

fn handle_key(_app: &mut App, _key: KeyEvent) -> bool {
    false
}

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    // Computer stats are nested inside the User response
    let computers = client.get_computers().await?;
    if !computers.is_empty() {
        println!("Found {} computers:", computers.len());
        for comp in computers {
            println!(
                "{} ({}): {} keys, {} clicks",
                comp.name, comp.id, comp.totals.keys, comp.totals.clicks
            );
        }
    } else {
        println!("No computers found in user profile.");
    }
    Ok(())
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Computers ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(err) = &app.error {
        f.render_widget(
            Paragraph::new(format!("Error: {}", err)).style(Style::default().fg(Color::Red)),
            inner_area,
        );
        return;
    }

    if app.computers_loading && app.computers.is_empty() {
        f.render_widget(Paragraph::new("Loading..."), inner_area);
        return;
    }

    if !app.computers.is_empty() {
        let mut rows = Vec::new();

        // Sort computers by keys (descending)
        let mut comps: Vec<_> = app.computers.iter().collect();
        comps.sort_by(|a, b| b.totals.keys.cmp(&a.totals.keys));

        for comp in comps {
            rows.push(Row::new(vec![
                comp.name.clone(),
                comp.os.clone(),
                comp.totals.keys.to_string(),
                comp.totals.clicks.to_string(),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(vec!["Name", "OS", "Keys", "Clicks"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(Block::default());

        f.render_widget(table, inner_area);
    } else if app.client.is_local() {
        let text = vec![
            Line::from(Span::styled(
                "No computers available in Local Mode.",
                Style::default().fg(Color::Yellow),
            )),
            Line::from("Reason: No WHATPULSE_API_KEY detected."),
            Line::from("To see per-computer stats, please provide a valid API key."),
        ];
        f.render_widget(Paragraph::new(text), inner_area);
    } else {
        f.render_widget(Paragraph::new("No computers found."), inner_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{ComputerResponse, ComputerTotals};
    use ratatui::Terminal;
    use ratatui::backend::{Backend, TestBackend};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_render_tui() {
        let fake_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NSJ9.signature";
        let client = WhatpulseClient::new(fake_token).await.unwrap();
        let (tx, _rx) = mpsc::channel(10);
        let mut app = App::new(client, tx);

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Case 1: Loading
        app.computers_loading = true;
        terminal
            .draw(|f| {
                render_tui(f, &app, f.area());
            })
            .unwrap();
        // Assertions would go here

        // Case 2: Data loaded
        let computer = ComputerResponse {
            id: 1,
            name: "Test PC".to_string(),
            client_version: "1.0.0".to_string(),
            os: "Windows".to_string(),
            is_archived: false,
            totals: ComputerTotals {
                keys: 1000,
                clicks: 500,
                download_mb: None,
                upload_mb: None,
                uptime_seconds: None,
                scrolls: None,
                distance_miles: None,
            },
            pulses: None,
            last_pulse_date: None,
            hardware: None,
        };

        app.computers_loading = false;
        app.computers = vec![computer];

        terminal
            .draw(|f| {
                render_tui(f, &app, f.area());
            })
            .unwrap();

        assert_eq!(
            terminal.backend().size().unwrap(),
            ratatui::layout::Size::new(40, 10)
        );
    }
}
