use crate::client::PulseResponse;
use crate::commands::TuiPage;
use crate::tui::app::{App, SelectionStep, TimePeriod};
use crate::tui::period_utils::{cycle_period_next, cycle_period_prev, handle_date_picker_key};
use chrono::{Datelike, Days, Local, Months, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Paragraph, Tabs},
};
use std::collections::HashMap;

inventory::submit! {
    TuiPage {
        title: "Uptime",
        category: "Uptime",
        render: render_uptime,
        handle_key: handle_uptime_key,
        handle_mouse: crate::commands::default_handle_mouse,
        priority: 60,
    }
}

fn handle_uptime_key(app: &mut App, key: KeyEvent) -> bool {
    if app.date_picker.open {
        handle_date_picker_key(app, key);
        return true;
    }

    match key.code {
        KeyCode::Char('h') | KeyCode::Char('[') => {
            app.uptime_period = cycle_period_prev(app.uptime_period);
            true
        }
        KeyCode::Char('l') | KeyCode::Char(']') => {
            app.uptime_period = cycle_period_next(app.uptime_period);
            true
        }
        KeyCode::Char('/') => {
            app.uptime_period = TimePeriod::Custom;
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
            if app.uptime_period == TimePeriod::Custom {
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

fn parse_pulse_date(date_str: &str) -> NaiveDate {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        dt.date()
    } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.date_naive()
    } else {
        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
    }
}

fn is_in_period(
    date: NaiveDate,
    period: TimePeriod,
    date_picker: &crate::tui::app::DatePickerState,
) -> bool {
    let now = Local::now().date_naive();
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
}

fn filter_pulses<'a>(
    pulses: &'a [PulseResponse],
    period: TimePeriod,
    date_picker: &crate::tui::app::DatePickerState,
) -> Vec<&'a PulseResponse> {
    pulses
        .iter()
        .filter(|p| {
            let date = parse_pulse_date(&p.date);
            is_in_period(date, period, date_picker)
        })
        .collect()
}

fn render_uptime(f: &mut Frame, app: &App, area: Rect) {
    // Root Layout: Top Filter, Main Content, Bottom Tabs
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Filter
            Constraint::Min(10),   // Main Content
            Constraint::Length(3), // Tab Bar
        ])
        .split(area);

    // 1. Top Filter
    let filter_text = Line::from(vec![
        Span::raw("Profile filter: "),
        Span::styled("[ All stats ]", Style::default().fg(Color::Cyan)),
    ]);
    f.render_widget(
        Paragraph::new(filter_text).alignment(Alignment::Right),
        chunks[0],
    );

    // 2. Main Content Split: Chart (Left) vs Side Info (Right)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),     // Total Active Hours Chart (takes remaining width)
            Constraint::Length(40), // Details + Favorite Reboot Days (fixed narrow width)
        ])
        .split(chunks[1]);

    // Data Processing
    // We want to calculate "Active Hours" by intersecting pulse uptime intervals with days.
    // 1. Generate Intervals from ALL pulses (to catch overnight sessions correctly)
    let mut intervals: Vec<(i64, i64)> = Vec::new();
    for pulse in &app.recent_pulses {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&pulse.date, "%Y-%m-%d %H:%M:%S") {
            // Assume pulse date is in Local time since API usually returns that or UTC?
            // The API returns "2023-01-01 12:00:00". Let's assume Local for now as per dashboard logic.
            // Actually, best to use NaiveDateTime and convert to timestamp if we assume Local.
            let end_dt = dt
                .and_local_timezone(Local)
                .latest()
                .unwrap_or(Local::now());
            let end_ts = end_dt.timestamp();
            let uptime = pulse.uptime_seconds.unwrap_or(0) as i64;
            let start_ts = end_ts - uptime;
            intervals.push((start_ts, end_ts));
        } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&pulse.date) {
            let end_ts = dt.timestamp();
            let uptime = pulse.uptime_seconds.unwrap_or(0) as i64;
            let start_ts = end_ts - uptime;
            intervals.push((start_ts, end_ts));
        }
    }

    // 2. Merge Overlapping Intervals
    intervals.sort_by_key(|k| k.0);
    let mut merged: Vec<(i64, i64)> = Vec::new();
    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            if interval.0 < last.1 {
                // Overlap or adjacent
                last.1 = last.1.max(interval.1);
            } else {
                merged.push(interval);
            }
        } else {
            merged.push(interval);
        }
    }

    // 3. Bucket into Days
    let mut daily_seconds: HashMap<String, i64> = HashMap::new();
    for (start, end) in merged {
        let mut curr = start;
        while curr < end {
            // Safe unwrap for timestamp
            let curr_dt = chrono::DateTime::from_timestamp(curr, 0)
                .unwrap_or_default()
                .with_timezone(&Local);

            // Calculate next midnight
            let next_day = curr_dt.date_naive().checked_add_days(Days::new(1)).unwrap();
            let next_midnight = next_day
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp();

            let segment_end = end.min(next_midnight);
            let duration = segment_end - curr;

            if duration > 0 {
                let day_key = curr_dt.format("%Y-%m-%d").to_string();
                *daily_seconds.entry(day_key).or_insert(0) += duration;
            }

            curr = segment_end;
        }
    }

    // 4. Aggregation and Filtering
    let agg_mode = match app.uptime_period {
        TimePeriod::Year | TimePeriod::All => "Monthly",
        TimePeriod::Custom => {
            if let (Some(start), Some(end)) = (app.date_picker.start_date, app.date_picker.end_date)
            {
                if (end - start).num_days() > 60 {
                    "Monthly"
                } else {
                    "Daily"
                }
            } else {
                "Daily"
            }
        }
        _ => "Daily",
    };

    let mut final_data: HashMap<String, i64> = HashMap::new();
    let now = Local::now().date_naive();

    for (day_key, secs) in daily_seconds {
        let date = NaiveDate::parse_from_str(&day_key, "%Y-%m-%d").unwrap_or_default();

        let in_period = match app.uptime_period {
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
                if let (Some(start), Some(end)) =
                    (app.date_picker.start_date, app.date_picker.end_date)
                {
                    date >= start && date <= end
                } else {
                    false
                }
            }
        };

        if in_period {
            let key = if agg_mode == "Monthly" {
                day_key[0..7].to_string() // YYYY-MM
            } else {
                day_key // YYYY-MM-DD
            };
            *final_data.entry(key).or_insert(0) += secs;
        }
    }

    // Sort keys first to ensure chronological order
    let mut sorted_keys: Vec<String> = final_data.keys().cloned().collect();
    sorted_keys.sort();

    let mut chart_data: Vec<(String, u64)> = sorted_keys
        .into_iter()
        .map(|k| {
            let secs = *final_data.get(&k).unwrap_or(&0);
            let label = if agg_mode == "Monthly" {
                // Parse YYYY-MM
                if let Ok(d) = NaiveDate::parse_from_str(&format!("{}-01", k), "%Y-%m-%d") {
                    d.format("%b '%y").to_string()
                } else {
                    k
                }
            } else {
                // Parse YYYY-MM-DD
                if let Ok(d) = NaiveDate::parse_from_str(&k, "%Y-%m-%d") {
                    d.format("%m/%d").to_string()
                } else {
                    k
                }
            };
            (label, secs as u64)
        })
        .collect();

    // Dynamic Slicing based on Width
    // Border takes 2 chars, Bar takes 5 chars, Gap takes 1 char = 6 chars per item
    // Actually BarChart implementation might behave slightly differently but 6 is a safe bet.
    let max_bars = (main_chunks[0].width.saturating_sub(2) / 6) as usize;
    if max_bars > 0 && chart_data.len() > max_bars {
        chart_data = chart_data.split_off(chart_data.len() - max_bars);
    }

    // Determine unit based on max value
    let max_val = chart_data.iter().map(|(_, v)| *v).max().unwrap_or(0);
    let (unit, divisor) = if max_val >= 3600 {
        ("Hours", 3600)
    } else if max_val >= 60 {
        ("Minutes", 60)
    } else {
        ("Seconds", 1)
    };

    let bar_data_refs: Vec<(&str, u64)> = chart_data
        .iter()
        .map(|(s, v)| (s.as_str(), *v / divisor))
        .collect();

    // 2a. Chart
    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .title(format!(" Total Active {} ", unit))
                .borders(Borders::ALL),
        )
        .data(&bar_data_refs)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Blue))
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(bar_chart, main_chunks[0]);

    // 3. Side Info Split: Details (Top) vs Reboot Days (Bottom)
    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Fixed height for details
            Constraint::Length(9), // Fixed height for 7 days + 2 borders
            Constraint::Min(0),    // Remaining spacer
        ])
        .split(main_chunks[1]);

    // -- Calculate Reboot Days --
    // Use filtered pulses for stats to respect the time period
    let filtered_pulses_refs =
        filter_pulses(&app.recent_pulses, app.uptime_period, &app.date_picker);
    // Sort pulses by date for reboot detection
    let mut sorted_pulses: Vec<&PulseResponse> = filtered_pulses_refs.clone();
    sorted_pulses.sort_by(|a, b| a.date.cmp(&b.date));

    let mut reboot_counts: HashMap<String, u64> = HashMap::new();
    let days_of_week = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    for day in &days_of_week {
        reboot_counts.insert(day.to_string(), 0);
    }

    // Scan for reboots in sorted pulses
    // We scan ALL pulses to detect reboots accurately (uptime drops),
    // then filter the reboot events by the selected time period.
    let mut all_pulses_refs: Vec<&PulseResponse> = app.recent_pulses.iter().collect();
    all_pulses_refs.sort_by(|a, b| a.date.cmp(&b.date));

    let mut prev_uptime = 0;
    for pulse in &all_pulses_refs {
        let uptime = pulse.uptime_seconds.unwrap_or(0);
        if uptime < prev_uptime {
            // Reboot detected
            let date = parse_pulse_date(&pulse.date);

            if is_in_period(date, app.uptime_period, &app.date_picker) {
                // Find weekday of this pulse
                let weekday = date.weekday(); // Mon=0, Sun=6
                let day_str = match weekday {
                    chrono::Weekday::Mon => "Mon",
                    chrono::Weekday::Tue => "Tue",
                    chrono::Weekday::Wed => "Wed",
                    chrono::Weekday::Thu => "Thu",
                    chrono::Weekday::Fri => "Fri",
                    chrono::Weekday::Sat => "Sat",
                    chrono::Weekday::Sun => "Sun",
                };
                *reboot_counts.get_mut(day_str).unwrap() += 1;
            }
        }
        prev_uptime = uptime;
    }

    let mut reboot_data: Vec<(&str, u64)> = Vec::new();
    for day in &days_of_week {
        reboot_data.push((day, *reboot_counts.get(*day).unwrap_or(&0)));
    }

    // -- Calculate Details --
    // Total Uptime (from user stats, not just filtered)
    let total_uptime_seconds = app
        .user_stats
        .as_ref()
        .and_then(|u| u.totals.uptime_seconds)
        .unwrap_or(0);

    // Longest Uptime (scan all filtered pulses)
    let longest_uptime_seconds = sorted_pulses
        .iter()
        .map(|p| p.uptime_seconds.unwrap_or(0))
        .max()
        .unwrap_or(0);

    // Format durations
    fn format_duration(secs: u64) -> String {
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        if days > 0 {
            format!("{}d, {}h, {}m", days, hours, mins)
        } else {
            format!("{}h, {}m", hours, mins)
        }
    }

    let details_text = vec![
        Line::from(vec![
            Span::raw("Unpulsed uptime: "),
            Span::styled("N/A", Style::default().fg(Color::DarkGray)), // Not available
        ]),
        Line::from(vec![
            Span::raw("Current uptime: "),
            Span::styled("N/A", Style::default().fg(Color::DarkGray)), // Need sys info
        ]),
        Line::from(vec![
            Span::raw("Total uptime: "),
            Span::styled(
                format_duration(total_uptime_seconds),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::raw("Longest uptime: "),
            Span::styled(
                format_duration(longest_uptime_seconds),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let details = Paragraph::new(details_text)
        .block(
            Block::default()
                .borders(Borders::ALL) // Added Borders
                .title(" Details ")
                .title_alignment(Alignment::Center)
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .alignment(Alignment::Left);

    f.render_widget(details, side_chunks[0]);

    // Favorite Reboot Days Chart (DOS-style Text Histogram)
    let max_reboots = reboot_data.iter().map(|(_, v)| *v).max().unwrap_or(0);
    let chart_width = side_chunks[1].width.saturating_sub(10); // Reserve space for label + count
    let max_bar_len = chart_width.min(20) as usize; // Cap bar length

    let mut histogram_lines = Vec::new();
    for (day, count) in reboot_data {
        let bar_len = if max_reboots > 0 {
            ((count as f64 / max_reboots as f64) * max_bar_len as f64).round() as usize
        } else {
            0
        };
        let bar = "#".repeat(bar_len);
        let padding = " ".repeat(max_bar_len.saturating_sub(bar_len));

        // Format: "Mon [#####     ] 5"
        let line = Line::from(vec![
            Span::styled(format!("{:<3} ", day), Style::default().fg(Color::Cyan)),
            Span::raw("["),
            Span::styled(bar, Style::default().fg(Color::Yellow)),
            Span::raw(padding),
            Span::raw("] "),
            Span::styled(
                count.to_string(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]);
        histogram_lines.push(line);
    }

    let histogram = Paragraph::new(histogram_lines).block(
        Block::default()
            .title(" Favorite reboot days ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL),
    );

    f.render_widget(histogram, side_chunks[1]);

    // 4. Time Period Tabs (Bottom)
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

    let selected_index = match app.uptime_period {
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

    f.render_widget(tabs, chunks[2]);
}
