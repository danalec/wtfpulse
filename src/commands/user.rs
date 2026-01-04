use anyhow::Result;
use crate::client::{WhatpulseClient, UserResponse, PulseResponse};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline, Tabs, Clear},
    Frame,
};
use crate::tui::app::{App, TimePeriod, SelectionStep};
use crate::commands::TuiPage;
use crossterm::event::{KeyEvent, KeyCode};
use chrono::{Datelike, NaiveDate, Days, Months, Utc, Local, TimeZone};

inventory::submit! {
    TuiPage {
        title: "Dashboard",
        render: render_tui,
        handle_key,
        priority: 0,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if app.date_picker.open {
        handle_date_picker_key(app, key);
        return true;
    }

    match key.code {
        KeyCode::Char('h') | KeyCode::Char('[') => {
            app.dashboard_period = match app.dashboard_period {
                TimePeriod::Today => TimePeriod::Custom,
                TimePeriod::Yesterday => TimePeriod::Today,
                TimePeriod::Week => TimePeriod::Yesterday,
                TimePeriod::Month => TimePeriod::Week,
                TimePeriod::Year => TimePeriod::Month,
                TimePeriod::All => TimePeriod::Year,
                TimePeriod::Custom => TimePeriod::All,
            };
            true
        }
        KeyCode::Char('l') | KeyCode::Char(']') => {
            app.dashboard_period = match app.dashboard_period {
                TimePeriod::Today => TimePeriod::Yesterday,
                TimePeriod::Yesterday => TimePeriod::Week,
                TimePeriod::Week => TimePeriod::Month,
                TimePeriod::Month => TimePeriod::Year,
                TimePeriod::Year => TimePeriod::All,
                TimePeriod::All => TimePeriod::Custom,
                TimePeriod::Custom => TimePeriod::Today,
            };
            true
        }
        KeyCode::Char('/') => {
            app.dashboard_period = TimePeriod::Custom;
            app.date_picker.open = true;
            app.date_picker.selection_step = SelectionStep::Start;
            if app.date_picker.start_date.is_none() {
                app.date_picker.current_selection = chrono::Local::now().date_naive();
            } else {
                app.date_picker.current_selection = app.date_picker.start_date.unwrap();
            }
            true
        }
        KeyCode::Enter => {
            if app.dashboard_period == TimePeriod::Custom {
                app.date_picker.open = true;
                // Reset picker state
                app.date_picker.selection_step = SelectionStep::Start;
                if app.date_picker.start_date.is_none() {
                    app.date_picker.current_selection = chrono::Local::now().date_naive();
                } else {
                    app.date_picker.current_selection = app.date_picker.start_date.unwrap();
                }
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn handle_date_picker_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.date_picker.open = false;
        }
        KeyCode::Left => {
            app.date_picker.current_selection = app.date_picker.current_selection.checked_sub_days(Days::new(1)).unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Right => {
            app.date_picker.current_selection = app.date_picker.current_selection.checked_add_days(Days::new(1)).unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Up => {
            app.date_picker.current_selection = app.date_picker.current_selection.checked_sub_days(Days::new(7)).unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Down => {
            app.date_picker.current_selection = app.date_picker.current_selection.checked_add_days(Days::new(7)).unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::PageUp => {
            app.date_picker.current_selection = app.date_picker.current_selection.checked_sub_months(Months::new(1)).unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::PageDown => {
            app.date_picker.current_selection = app.date_picker.current_selection.checked_add_months(Months::new(1)).unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Enter => {
            match app.date_picker.selection_step {
                SelectionStep::Start => {
                    app.date_picker.start_date = Some(app.date_picker.current_selection);
                    app.date_picker.selection_step = SelectionStep::End;
                    // Auto move cursor to next day for convenience
                    app.date_picker.current_selection = app.date_picker.current_selection.checked_add_days(Days::new(1)).unwrap_or(app.date_picker.current_selection);
                }
                SelectionStep::End => {
                    let end = app.date_picker.current_selection;
                    if let Some(start) = app.date_picker.start_date {
                        if end >= start {
                            app.date_picker.end_date = Some(end);
                            app.date_picker.open = false;
                        } else {
                            // Invalid range, maybe reset or swap? Let's just swap for UX
                            app.date_picker.start_date = Some(end);
                            app.date_picker.end_date = Some(start);
                            app.date_picker.open = false;
                        }
                    } else {
                        // Should not happen if step is End
                        app.date_picker.start_date = Some(end);
                        app.date_picker.selection_step = SelectionStep::End;
                    }
                }
            }
        }
        _ => {}
    }
}

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    let user = client.get_resource::<UserResponse>("user").await?;
    // CLI output remains simple
    println!("User: {}", user.account_name.as_deref().unwrap_or("Unknown"));
    Ok(())
}

fn filter_pulses<'a>(pulses: &'a [PulseResponse], period: TimePeriod, date_picker: &crate::tui::app::DatePickerState) -> Vec<&'a PulseResponse> {
    let now = Local::now().date_naive();
    
    pulses.iter().filter(|p| {
        let ts_str = p.timestamp.as_deref().unwrap_or("0");
        let ts = ts_str.parse::<i64>().unwrap_or(0);
        
        let dt_utc = Utc.timestamp_opt(ts, 0).single().unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());
        let date = dt_utc.with_timezone(&Local).date_naive();
        
        match period {
            TimePeriod::Today => date == now,
            TimePeriod::Yesterday => date == now.pred_opt().unwrap(),
            TimePeriod::Week => {
                let week_ago = now.checked_sub_days(Days::new(7)).unwrap();
                date >= week_ago && date <= now
            }
            TimePeriod::Month => {
                let month_ago = now.checked_sub_months(Months::new(1)).unwrap();
                date >= month_ago && date <= now
            }
            TimePeriod::Year => {
                let year_ago = now.checked_sub_months(Months::new(12)).unwrap();
                date >= year_ago && date <= now
            }
            TimePeriod::All => true,
            TimePeriod::Custom => {
                if let (Some(start), Some(end)) = (date_picker.start_date, date_picker.end_date) {
                    date >= start && date <= end
                } else {
                    false
                }
            }
        }
    }).collect()
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Main Content
            Constraint::Length(3),  // Period Selector
        ])
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(15), // User Stats
            Constraint::Min(10),    // Pulse Graph
        ])
        .split(chunks[0]);

    render_user_stats(f, app, content_chunks[0]);
    render_pulse_graph(f, app, content_chunks[1]);
    render_period_selector(f, app, chunks[1]);

    if app.date_picker.open {
        render_date_picker(f, app, area);
    }
}

fn render_period_selector(f: &mut Frame, app: &App, area: Rect) {
    let periods = vec![
        "Today", "Yesterday", "Week", "Month", "Year", "All", "Custom"
    ];
    
    let titles: Vec<Line> = periods.iter().map(|t| {
        Line::from(Span::styled(*t, Style::default().fg(Color::Gray)))
    }).collect();

    let selected_index = match app.dashboard_period {
        TimePeriod::Today => 0,
        TimePeriod::Yesterday => 1,
        TimePeriod::Week => 2,
        TimePeriod::Month => 3,
        TimePeriod::Year => 4,
        TimePeriod::All => 5,
        TimePeriod::Custom => 6,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Time Period (h/l: Cycle | /: Custom Date) "))
        .select(selected_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    f.render_widget(tabs, area);
}

fn render_user_stats(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" User Stats ");
    
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
        let name = user.account_name.as_deref().unwrap_or("Unknown");
        let country = user.country.as_deref().unwrap_or("Unknown");
        let joined = user.date_joined.as_deref().unwrap_or("Unknown");
        let period_label = format!("{:?}", app.dashboard_period);

        let filtered_pulses = filter_pulses(&app.recent_pulses, app.dashboard_period, &app.date_picker);
        
        let mut text = format!(
            "Account: {}\nCountry: {}\nJoined:  {}\nPeriod:  {}\n",
            name, country, joined, period_label
        );

        if app.dashboard_period == TimePeriod::All {
            text.push_str(&format!(
                "\nTotal Keys:    {}\nTotal Clicks:  {}\nTotal Down:    {} MB\nTotal Up:      {} MB",
                user.keys.as_deref().unwrap_or("0"),
                user.clicks.as_deref().unwrap_or("0"),
                user.download_mb.as_deref().unwrap_or("0"),
                user.upload_mb.as_deref().unwrap_or("0"),
            ));
        } else {
            let p_keys: u64 = filtered_pulses.iter().map(|p| p.keys.as_deref().unwrap_or("0").replace(',', "").parse::<u64>().unwrap_or(0)).sum();
            let p_clicks: u64 = filtered_pulses.iter().map(|p| p.clicks.as_deref().unwrap_or("0").replace(',', "").parse::<u64>().unwrap_or(0)).sum();
            let p_down: u64 = filtered_pulses.iter().map(|p| p.download_mb.as_deref().unwrap_or("0").replace(',', "").parse::<u64>().unwrap_or(0)).sum();
            let p_up: u64 = filtered_pulses.iter().map(|p| p.upload_mb.as_deref().unwrap_or("0").replace(',', "").parse::<u64>().unwrap_or(0)).sum();

            text.push_str(&format!(
                "\nPeriod Keys:   {}\nPeriod Clicks: {}\nPeriod Down:   {} MB\nPeriod Up:     {} MB",
                p_keys, p_clicks, p_down, p_up
            ));
            text.push_str("\n(Based on available pulses)");
        }

        if let Some(ranks) = &user.ranks {
            text.push_str("\n\nRanks:\n");
            text.push_str(&format!("  Keys: {}\n", ranks.get("Keys").and_then(|v| v.as_str()).unwrap_or("-")));
            text.push_str(&format!("  Clicks: {}\n", ranks.get("Clicks").and_then(|v| v.as_str()).unwrap_or("-")));
            text.push_str(&format!("  Download: {}\n", ranks.get("Download").and_then(|v| v.as_str()).unwrap_or("-")));
            text.push_str(&format!("  Upload: {}", ranks.get("Upload").and_then(|v| v.as_str()).unwrap_or("-")));
        }
        
        if app.dashboard_period == TimePeriod::Custom {
             if let (Some(s), Some(e)) = (app.date_picker.start_date, app.date_picker.end_date) {
                 text.push_str(&format!("\n\nCustom Range: {} to {}", s, e));
             } else {
                 text.push_str("\n\nCustom Range: (Press / to select dates)");
             }
        }

        f.render_widget(Paragraph::new(text), inner_area);
    } else {
        f.render_widget(Paragraph::new("No user data available."), inner_area);
    }
}

fn render_pulse_graph(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Recent Activity (Keys in Pulses) ");
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(err) = &app.pulses_error {
        f.render_widget(Paragraph::new(format!("Error: {}", err)).style(Style::default().fg(Color::Red)), inner_area);
        return;
    }

    let filtered_pulses = filter_pulses(&app.recent_pulses, app.dashboard_period, &app.date_picker);

    if filtered_pulses.is_empty() {
        if app.pulses_loading {
            f.render_widget(Paragraph::new("Loading pulses..."), inner_area);
        } else {
            f.render_widget(Paragraph::new("No pulses found for this period."), inner_area);
        }
        return;
    }

    let max_bars = inner_area.width as usize;
    let data_len = filtered_pulses.len().min(max_bars);
    let data_iter = filtered_pulses.iter().take(data_len).rev();
    
    let values: Vec<u64> = data_iter
        .map(|p| {
            p.keys
                .as_deref()
                .unwrap_or("0")
                .replace(',', "")
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect();

    let sparkline = Sparkline::default()
        .block(Block::default())
        .style(Style::default().fg(Color::Yellow))
        .data(&values)
        .bar_set(ratatui::symbols::bar::NINE_LEVELS);
    
    f.render_widget(sparkline, inner_area);
}

fn render_date_picker(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title(" Date Picker ").borders(Borders::ALL).style(Style::default().bg(Color::DarkGray));
    // Use fixed size for calendar (approx 40x16 is good for readability)
    let area = centered_fixed_area(40, 16, area);
    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    
    // Header
    let current_month = app.date_picker.current_selection.format("%B %Y").to_string();
    let step_msg = match app.date_picker.selection_step {
        SelectionStep::Start => "Select START Date",
        SelectionStep::End => "Select END Date",
    };
    
    let header_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
        
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(current_month, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled(step_msg, Style::default().fg(Color::Cyan)),
    ])).alignment(Alignment::Center), header_layout[0]);

    // Calendar Grid
    let days_header = "Sun Mon Tue Wed Thu Fri Sat";
    f.render_widget(Paragraph::new(days_header).alignment(Alignment::Center), header_layout[1]);

    let grid_area = header_layout[2];
    
    // Calculate calendar days
    let sel = app.date_picker.current_selection;
    let first_day_of_month = NaiveDate::from_ymd_opt(sel.year(), sel.month(), 1).unwrap();
    // Weekday: Mon=0..Sun=6 in chrono (Datelike::weekday().num_days_from_monday())
    // We want Sun=0..Sat=6. 
    // Chrono weekday: Mon(0), Tue(1)..Sun(6).
    // Shift: Sun(6)->0, Mon(0)->1 ...
    let start_offset = (first_day_of_month.weekday().num_days_from_sunday()) as u64; // 0 for Sunday
    
    // Render weeks
    let mut current_date = first_day_of_month.checked_sub_days(Days::new(start_offset)).unwrap();
    let mut rows = Vec::new();
    
    // 6 weeks usually enough
    for _week in 0..6 {
        let mut row_spans = Vec::new();
        for _day in 0..7 {
            let day_str = format!("{:>3}", current_date.day());
            let mut style = Style::default();

            // Check if in range
            let mut in_range = false;
            if let (Some(s), Some(e)) = (app.date_picker.start_date, app.date_picker.end_date) {
                if current_date >= s && current_date <= e {
                    in_range = true;
                }
            } else if let Some(s) = app.date_picker.start_date {
                 // During selection
                 if app.date_picker.selection_step == SelectionStep::End {
                     if current_date >= s && current_date <= app.date_picker.current_selection {
                         in_range = true;
                     }
                 } else if current_date == s {
                     in_range = true;
                 }
            }

            // Colors
            if current_date == app.date_picker.current_selection {
                style = style.bg(Color::Yellow).fg(Color::Black);
            } else if in_range {
                style = style.bg(Color::Blue);
            } else if current_date.month() != sel.month() {
                style = style.fg(Color::Gray);
            }

            if Some(current_date) == app.date_picker.start_date {
                 style = style.bg(Color::Green).fg(Color::Black);
            }
            if Some(current_date) == app.date_picker.end_date {
                 style = style.bg(Color::Red).fg(Color::Black);
            }

            row_spans.push(Span::styled(day_str, style));
            row_spans.push(Span::raw(" ")); // spacing
            
            current_date = current_date.checked_add_days(Days::new(1)).unwrap();
        }
        rows.push(Line::from(row_spans));
    }
    
    let calendar_paragraph = Paragraph::new(rows).alignment(Alignment::Center);
    f.render_widget(calendar_paragraph, grid_area);
    
    // Instructions footer
    let footer_area = Rect::new(area.x, area.y + area.height - 2, area.width, 1);
    f.render_widget(Paragraph::new("Arrows: Move | PgUp/Dn: Month | Enter: Select | Esc: Cancel").alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray)), footer_area);
}

fn centered_fixed_area(width: u16, height: u16, area: Rect) -> Rect {
    let x = if area.width > width { (area.width - width) / 2 } else { 0 };
    let y = if area.height > height { (area.height - height) / 2 } else { 0 };
    
    Rect {
        x: area.x + x,
        y: area.y + y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tokio::sync::mpsc;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_render_tui() {
        let fake_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NSJ9.signature";
        let client = WhatpulseClient::new(fake_token).await.unwrap();
        let (tx, _rx) = mpsc::channel(10);
        let mut app = App::new(client, tx);

        let backend = TestBackend::new(40, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        // Case 1: Loading
        app.user_loading = true;
        app.pulses_loading = true;
        terminal.draw(|f| {
            render_tui(f, &app, f.area());
        }).unwrap();

        // Case 2: Data loaded
        app.user_loading = false;
        app.pulses_loading = false;
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
            computers: None,
            ranks: None,
            extra: HashMap::new(),
        });

        terminal.draw(|f| {
            render_tui(f, &app, f.area());
        }).unwrap();
        
        let buffer = terminal.backend().buffer();
        // Simple check for content
        let content = format!("{:?}", buffer);
        assert!(content.contains("TestUser"));
        assert!(content.contains("Time Period"));
    }
}
