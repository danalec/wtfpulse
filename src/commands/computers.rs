use anyhow::Result;
use crate::client::{WhatpulseClient, UserResponse};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};
use crate::tui::app::App;
use crate::commands::TuiPage;
use crossterm::event::KeyEvent;

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
    let user = client.get_resource::<UserResponse>("user").await?;
    if let Some(computers) = user.computers {
        println!("Found {} computers:", computers.len());
        for (_, comp) in computers {
            println!("{} ({}): {} keys, {} clicks", 
                comp.name.as_deref().unwrap_or("unknown"),
                comp.id.as_deref().unwrap_or("unknown"),
                comp.keys.as_deref().unwrap_or("0"),
                comp.clicks.as_deref().unwrap_or("0")
            );
        }
    } else {
        println!("No computers found in user profile.");
    }
    Ok(())
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Computers ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(err) = &app.error {
        f.render_widget(Paragraph::new(format!("Error: {}", err)).style(Style::default().fg(Color::Red)), inner_area);
        return;
    }

    if app.user_loading && app.user_stats.is_none() {
        f.render_widget(Paragraph::new("Loading..."), inner_area);
        return;
    }

    if let Some(user) = &app.user_stats {
        if let Some(computers) = &user.computers {
            let mut rows = Vec::new();
            
            // Sort computers by keys (descending)
            let mut comps: Vec<_> = computers.values().collect();
            comps.sort_by(|a, b| {
                let keys_a = a.keys.as_deref().unwrap_or("0").replace(',', "").parse::<u64>().unwrap_or(0);
                let keys_b = b.keys.as_deref().unwrap_or("0").replace(',', "").parse::<u64>().unwrap_or(0);
                keys_b.cmp(&keys_a)
            });

            for comp in comps {
                rows.push(Row::new(vec![
                    comp.name.as_deref().unwrap_or("Unknown").to_string(),
                    comp.os.as_deref().unwrap_or("-").to_string(),
                    comp.keys.as_deref().unwrap_or("0").to_string(),
                    comp.clicks.as_deref().unwrap_or("0").to_string(),
                ]));
            }

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ]
            )
            .header(Row::new(vec!["Name", "OS", "Keys", "Clicks"]).style(Style::default().fg(Color::Yellow)))
            .block(Block::default());

            f.render_widget(table, inner_area);
        } else {
            f.render_widget(Paragraph::new("No computers found."), inner_area);
        }
    } else {
        f.render_widget(Paragraph::new("No data available."), inner_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::{Backend, TestBackend};
    use ratatui::Terminal;
    use tokio::sync::mpsc;
    use crate::client::ComputerResponse;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_render_tui() {
        let fake_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NSJ9.signature";
        let client = WhatpulseClient::new(fake_token).await.unwrap();
        let (tx, _rx) = mpsc::channel(10);
        let mut app = App::new(client, tx);

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Case 1: Loading
        app.user_loading = true;
        terminal.draw(|f| {
            render_tui(f, &app, f.area());
        }).unwrap();
        // Assertions would go here

        // Case 2: Data loaded
        let mut computers = HashMap::new();
        computers.insert("1".to_string(), ComputerResponse {
            id: Some("1".to_string()),
            name: Some("Test PC".to_string()),
            os: Some("Windows".to_string()),
            keys: Some("1000".to_string()),
            clicks: Some("500".to_string()),
            download_mb: None,
            upload_mb: None,
            extra: HashMap::new(),
        });

        app.user_loading = false;
        app.user_stats = Some(UserResponse {
            id: Some("12345".to_string()),
            account_name: Some("TestUser".to_string()),
            country: Some("TestLand".to_string()),
            date_joined: Some("2021-01-01".to_string()),
            keys: Some("1000".to_string()),
            clicks: Some("500".to_string()),
            download_mb: None,
            upload_mb: None,
            uptime_seconds: None,
            computers: Some(computers),
            ranks: None,
            extra: HashMap::new(),
        });

        terminal.draw(|f| {
            render_tui(f, &app, f.area());
        }).unwrap();
        
        assert_eq!(terminal.backend().size().unwrap(), ratatui::layout::Size::new(40, 10));
    }
}
