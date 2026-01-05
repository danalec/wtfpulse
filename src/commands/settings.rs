use crate::commands::TuiPage;
use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct SettingsPage;

impl SettingsPage {
    pub fn render(f: &mut Frame, app: &App, area: Rect) {
        render_settings(f, app, area);
    }

    pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
        handle_settings_key(app, key)
    }

    pub fn handle_mouse(_app: &mut App, _mouse: crossterm::event::MouseEvent) -> bool {
        false
    }
}

inventory::submit! {
    TuiPage {
        title: "Settings",
        render: SettingsPage::render,
        handle_key: SettingsPage::handle_key,
        handle_mouse: SettingsPage::handle_mouse,
        priority: 90,
    }
}

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Refresh Rate
            Constraint::Length(3), // API Key
            Constraint::Min(0),    // Instructions
        ])
        .split(area);

    let refresh_rate = app.config.refresh_rate_seconds.unwrap_or(60);
    let rr_text = format!("Refresh Rate: {} seconds", refresh_rate);

    let rr_block = Block::default()
        .borders(Borders::ALL)
        .title(" Configuration ")
        .style(Style::default().fg(if app.is_editing_api_key {
            Color::Gray
        } else {
            Color::White
        }));

    let rr_para = Paragraph::new(rr_text).block(rr_block);

    f.render_widget(rr_para, chunks[0]);

    // API Key
    let (key_text, key_style, border_style) = if app.is_editing_api_key {
        (
            format!("API Key: {}_", app.api_key_input), // Show cursor
            Style::default().fg(Color::Yellow),
            Style::default().fg(Color::Yellow),
        )
    } else {
        let api_key = app.config.api_key.as_deref().unwrap_or("Not Set");
        let masked_key = if api_key == "Not Set" {
            "Not Set"
        } else {
            "****************"
        };
        (
            format!("API Key: {}", masked_key),
            Style::default().fg(Color::Gray),
            Style::default().fg(Color::Gray),
        )
    };

    let key_block = Block::default()
        .borders(Borders::ALL)
        .title(" API Key ")
        .border_style(border_style)
        .style(key_style);

    f.render_widget(Paragraph::new(key_text).block(key_block), chunks[1]);

    // Instructions
    let mut instructions = vec![Line::from(Span::styled(
        "Controls:",
        Style::default().add_modifier(Modifier::BOLD),
    ))];

    if app.is_editing_api_key {
        instructions.push(Line::from("  Enter: Save API Key"));
        instructions.push(Line::from("  Ctrl+V: Paste from Clipboard"));
        instructions.push(Line::from("  Esc: Cancel Editing"));
    } else {
        instructions.push(Line::from(
            "  r: Cycle Refresh Rate (1s, 5s, 10s, 30s, 60s)",
        ));
        instructions.push(Line::from("  e: Edit API Key"));
        instructions.push(Line::from("  S: Save Configuration"));
    }

    let instr_block = Block::default().borders(Borders::ALL).title(" Help ");

    f.render_widget(Paragraph::new(instructions).block(instr_block), chunks[2]);
}

pub fn handle_settings_key(app: &mut App, key: KeyEvent) -> bool {
    if app.is_editing_api_key {
        match key.code {
            KeyCode::Enter => {
                // Save changes
                let new_key = app.api_key_input.trim().to_string();
                if new_key.is_empty() {
                    app.config.api_key = None;
                } else {
                    app.config.api_key = Some(new_key);
                }
                app.is_editing_api_key = false;

                // Auto-save config when confirming API key
                if let Err(e) = app.config.save() {
                    app.error = Some(format!("Failed to save config: {}", e));
                } else {
                    app.set_notification("Configuration Saved!".to_string());
                }
                true
            }
            KeyCode::Esc => {
                // Cancel changes
                app.is_editing_api_key = false;
                true
            }
            KeyCode::Backspace => {
                app.api_key_input.pop();
                true
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new()
                    && let Ok(text) = clipboard.get_text()
                {
                    app.api_key_input.push_str(&text);
                }
                true
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                {
                    app.api_key_input.push(c);
                }
                true
            }
            _ => false,
        }
    } else {
        match key.code {
            KeyCode::Char('r') => {
                let current = app.config.refresh_rate_seconds.unwrap_or(60);
                let next = match current {
                    1 => 5,
                    5 => 10,
                    10 => 30,
                    30 => 60,
                    _ => 1,
                };
                app.config.refresh_rate_seconds = Some(next);
                app.refresh_rate = std::time::Duration::from_secs(next);
                true
            }
            KeyCode::Char('e') => {
                app.is_editing_api_key = true;
                app.api_key_input = app.config.api_key.clone().unwrap_or_default();
                true
            }
            KeyCode::Char('S') => {
                if let Err(e) = app.config.save() {
                    app.error = Some(format!("Failed to save config: {}", e));
                } else {
                    app.set_notification("Configuration Saved!".to_string());
                }
                true
            }
            _ => false,
        }
    }
}
