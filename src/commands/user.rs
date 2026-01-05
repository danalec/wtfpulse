use crate::client::{PulseResponse, WhatpulseClient};
use crate::commands::TuiPage;
use crate::tui::app::{App, SelectionStep, TimePeriod};
use anyhow::Result;
use chrono::{Days, Local, Months, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline, Tabs},
};

inventory::submit! {
    TuiPage {
        title: "Dashboard",
        render: render_tui,
        handle_key,
        handle_mouse: crate::commands::default_handle_mouse,
        priority: 0,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if app.date_picker.open {
        crate::tui::period_utils::handle_date_picker_key(app, key);
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

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    let user = client.get_user().await?;
    // CLI output remains simple
    println!("User: {}", user.username);
    Ok(())
}

fn filter_pulses<'a>(
    pulses: &'a [PulseResponse],
    period: TimePeriod,
    date_picker: &crate::tui::app::DatePickerState,
) -> Vec<&'a PulseResponse> {
    let now = Local::now().date_naive();

    pulses
        .iter()
        .filter(|p| {
            // Try to parse ISO string first, fallback if needed
            // Assuming format like "2023-01-01T12:00:00" or similar
            let date = if let Ok(dt) =
                chrono::NaiveDateTime::parse_from_str(&p.date, "%Y-%m-%d %H:%M:%S")
            {
                dt.date()
            } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&p.date) {
                dt.date_naive()
            } else {
                NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
            };

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
                    if let (Some(start), Some(end)) = (date_picker.start_date, date_picker.end_date)
                    {
                        date >= start && date <= end
                    } else {
                        false
                    }
                }
            }
        })
        .collect()
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let is_local = app.client.is_local();

    let constraints = if is_local {
        vec![Constraint::Min(10)]
    } else {
        vec![
            Constraint::Min(10),   // Main Content
            Constraint::Length(3), // Period Selector
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
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

    if !is_local {
        render_period_selector(f, app, chunks[1]);
    }
}

fn render_period_selector(f: &mut Frame, app: &App, area: Rect) {
    let periods = [
        "Today",
        "Yesterday",
        "Week",
        "Month",
        "Year",
        "All",
        "Custom",
    ];

    let titles: Vec<Line> = periods
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Gray))))
        .collect();

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Time Period (h/l: Cycle | /: Custom Date) "),
        )
        .select(selected_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    f.render_widget(tabs, area);
}

fn render_user_stats(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" User Stats ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(err) = &app.error {
        f.render_widget(
            Paragraph::new(format!("Error: {}", err)).style(Style::default().fg(Color::Red)),
            inner_area,
        );
        return;
    }

    if app.user_loading && app.user_stats.is_none() {
        f.render_widget(Paragraph::new("Loading..."), inner_area);
        return;
    }

    if let Some(user) = &app.user_stats {
        let name = &user.username;
        let country = user
            .country_id
            .map(|c| c.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let joined = user.date_joined.as_deref().unwrap_or("Unknown");
        let period_label = if app.client.is_local() {
            "Total (Local)"
        } else {
            match app.dashboard_period {
                TimePeriod::Today => "Today",
                TimePeriod::Yesterday => "Yesterday",
                TimePeriod::Week => "Last 7 Days",
                TimePeriod::Month => "Last 30 Days",
                TimePeriod::Year => "Last 365 Days",
                TimePeriod::All => "All Time",
                TimePeriod::Custom => "Custom Range",
            }
        };

        let filtered_pulses =
            filter_pulses(&app.recent_pulses, app.dashboard_period, &app.date_picker);

        let mut text = format!(
            "Account: {}\nCountry: {}\nJoined:  {}\nPeriod:  {}\n",
            name, country, joined, period_label
        );

        if app.client.is_local() || app.dashboard_period == TimePeriod::All {
            text.push_str(&format!(
                "\nTotal Keys:    {}\nTotal Clicks:  {}\nTotal Down:    {:.2} MB\nTotal Up:      {:.2} MB",
                user.totals.keys.unwrap_or(0),
                user.totals.clicks.unwrap_or(0),
                user.totals.download_mb.unwrap_or(0.0),
                user.totals.upload_mb.unwrap_or(0.0),
            ));

            if app.client.is_local() {
                text.push_str("\n\n(Local Mode - Pulse History Unavailable)");
            }
        } else {
            let p_keys: u64 = filtered_pulses.iter().map(|p| p.keys.unwrap_or(0)).sum();
            let p_clicks: u64 = filtered_pulses.iter().map(|p| p.clicks.unwrap_or(0)).sum();
            let p_down: f64 = filtered_pulses
                .iter()
                .map(|p| p.download_mb.unwrap_or(0.0))
                .sum();
            let p_up: f64 = filtered_pulses
                .iter()
                .map(|p| p.upload_mb.unwrap_or(0.0))
                .sum();

            text.push_str(&format!(
                "\nTotal Keys:    {}\nTotal Clicks:  {}\nTotal Down:    {:.2} MB\nTotal Up:      {:.2} MB",
                p_keys, p_clicks, p_down, p_up
            ));
        }

        if let Some(ranks) = &user.ranks {
            text.push_str("\n\nRanks:\n");
            text.push_str(&format!("  Keys: {}\n", ranks.keys));
            text.push_str(&format!("  Clicks: {}\n", ranks.clicks));
            text.push_str(&format!("  Download: {}\n", ranks.download));
            text.push_str(&format!("  Upload: {}", ranks.upload));
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
    let title = if app.client.is_local() {
        " Local Mode Statistics "
    } else {
        " Recent Activity "
    };

    let block = Block::default().borders(Borders::ALL).title(title);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if app.client.is_local() {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(inner_area);

        // 1. Total Stats
        if let Some(user) = &app.user_stats {
            let total_text = vec![
                Line::from(Span::styled(
                    "Total Stats",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(format!("Keys:    {}", user.totals.keys.unwrap_or(0))),
                Line::from(format!("Clicks:  {}", user.totals.clicks.unwrap_or(0))),
                Line::from(format!(
                    "Down:    {:.2} MB",
                    user.totals.download_mb.unwrap_or(0.0)
                )),
                Line::from(format!(
                    "Up:      {:.2} MB",
                    user.totals.upload_mb.unwrap_or(0.0)
                )),
                Line::from(format!(
                    "Uptime:  {}",
                    user.totals.uptime_seconds.unwrap_or(0)
                )),
            ];
            f.render_widget(Paragraph::new(total_text), chunks[0]);
        } else {
            f.render_widget(Paragraph::new("Loading Total Stats..."), chunks[0]);
        }

        // 2. Real-time Stats
        let rt_text = vec![
            Line::from(Span::styled(
                "Real-time Stats",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )),
            Line::from(""),
            Line::from(format!(
                "Speed:   {:.2} keys/s",
                app.kinetic_stats.keys_per_second
            )),
            Line::from(format!(
                "Power:   {:.4} W",
                app.kinetic_stats.current_power_watts
            )),
        ];
        f.render_widget(Paragraph::new(rt_text), chunks[1]);

        // 3. Unpulsed Stats
        let up_text = vec![
            Line::from(Span::styled(
                "Unpulsed Stats",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Magenta),
            )),
            Line::from(""),
            Line::from(format!("Keys:    {}", app.kinetic_stats.unpulsed_keys)),
            Line::from(format!("Clicks:  {}", app.kinetic_stats.unpulsed_clicks)),
        ];
        f.render_widget(Paragraph::new(up_text), chunks[2]);

        return;
    }

    if let Some(err) = &app.pulses_error {
        f.render_widget(
            Paragraph::new(format!("Error: {}", err)).style(Style::default().fg(Color::Red)),
            inner_area,
        );
        return;
    }

    let filtered_pulses = filter_pulses(&app.recent_pulses, app.dashboard_period, &app.date_picker);

    if filtered_pulses.is_empty() {
        if app.pulses_loading {
            f.render_widget(Paragraph::new("Loading pulses..."), inner_area);
        } else {
            f.render_widget(
                Paragraph::new("No pulses found for this period."),
                inner_area,
            );
        }
        return;
    }

    let max_bars = inner_area.width as usize;
    let data_len = filtered_pulses.len().min(max_bars);
    let data_iter = filtered_pulses.iter().take(data_len).rev();

    let values: Vec<u64> = data_iter.map(|p| p.keys.unwrap_or(0)).collect();

    let sparkline = Sparkline::default()
        .block(Block::default())
        .style(Style::default().fg(Color::Yellow))
        .data(&values)
        .bar_set(ratatui::symbols::bar::NINE_LEVELS);

    f.render_widget(sparkline, inner_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{UserResponse, UserTotals};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use tokio::sync::mpsc;

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
        terminal
            .draw(|f| {
                render_tui(f, &app, f.area());
            })
            .unwrap();

        // Case 2: Data loaded
        app.user_loading = false;
        app.pulses_loading = false;
        app.user_stats = Some(UserResponse {
            id: 12345,
            username: "TestUser".to_string(),
            country_id: Some(1),
            date_joined: Some("2021-01-01".to_string()),
            first_pulse_date: None,
            last_pulse_date: None,
            pulses: 0,
            team_id: None,
            team_is_manager: false,
            is_premium: false,
            referrals: 0,
            last_referral_date: None,
            avatar: None,
            totals: UserTotals {
                keys: Some(1000),
                clicks: Some(500),
                download_mb: Some(0.0),
                upload_mb: Some(0.0),
                uptime_seconds: Some(0),
                scrolls: 0,
                distance_miles: Some(0.0),
            },
            ranks: None,
            include_in_rankings: false,
            distance_system: "metric".to_string(),
            last_pulse: None,
        });

        terminal
            .draw(|f| {
                render_tui(f, &app, f.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        // Simple check for content
        let content = format!("{:?}", buffer);
        assert!(content.contains("TestUser"));
        assert!(content.contains("Time Period"));
    }
}
