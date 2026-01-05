use crate::commands::TuiPage;
use crate::tui::app::{App, SelectionStep, TimePeriod};
use chrono::{Datelike, Days, Months, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub mod layouts;
use layouts::{KEY_HEIGHT, KeyboardLayout};

inventory::submit! {
    TuiPage {
        title: "Keyboard",
        render: render_tui,
        handle_key,
        handle_mouse: crate::commands::default_handle_mouse,
        priority: 11,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if app.date_picker.open {
        handle_date_picker_key(app, key);
        if !app.date_picker.open {
            // If closed, fetch heatmap with new range if custom
            fetch_heatmap(app);
        }
        return true;
    }

    if app.show_layout_popup {
        match key.code {
            KeyCode::Esc => {
                app.show_layout_popup = false;
                app.layout_search_query.clear();
                return true;
            }
            KeyCode::Enter => {
                if let Some(selected_idx) = app.layout_list_state.get_mut().selected() {
                    let filtered: Vec<KeyboardLayout> = KeyboardLayout::all()
                        .into_iter()
                        .filter(|l| {
                            l.to_string()
                                .to_lowercase()
                                .contains(&app.layout_search_query.to_lowercase())
                        })
                        .collect();

                    if let Some(layout) = filtered.get(selected_idx) {
                        app.keyboard_layout = *layout;
                        app.show_layout_popup = false;
                        app.layout_search_query.clear();
                    }
                }
                return true;
            }
            KeyCode::Up => {
                let filtered_count = KeyboardLayout::all()
                    .into_iter()
                    .filter(|l| {
                        l.to_string()
                            .to_lowercase()
                            .contains(&app.layout_search_query.to_lowercase())
                    })
                    .count();

                if filtered_count > 0 {
                    let i = match app.layout_list_state.get_mut().selected() {
                        Some(i) => {
                            if i == 0 {
                                filtered_count - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    app.layout_list_state.get_mut().select(Some(i));
                }
                return true;
            }
            KeyCode::Down => {
                let filtered_count = KeyboardLayout::all()
                    .into_iter()
                    .filter(|l| {
                        l.to_string()
                            .to_lowercase()
                            .contains(&app.layout_search_query.to_lowercase())
                    })
                    .count();

                if filtered_count > 0 {
                    let i = match app.layout_list_state.get_mut().selected() {
                        Some(i) => {
                            if i >= filtered_count - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    app.layout_list_state.get_mut().select(Some(i));
                }
                return true;
            }
            KeyCode::Home => {
                app.layout_list_state.get_mut().select(Some(0));
                return true;
            }
            KeyCode::End => {
                let filtered_count = KeyboardLayout::all()
                    .into_iter()
                    .filter(|l| {
                        l.to_string()
                            .to_lowercase()
                            .contains(&app.layout_search_query.to_lowercase())
                    })
                    .count();
                if filtered_count > 0 {
                    app.layout_list_state
                        .get_mut()
                        .select(Some(filtered_count - 1));
                }
                return true;
            }
            KeyCode::PageUp => {
                let current = app.layout_list_state.get_mut().selected().unwrap_or(0);
                let next = current.saturating_sub(5);
                app.layout_list_state.get_mut().select(Some(next));
                return true;
            }
            KeyCode::PageDown => {
                let filtered_count = KeyboardLayout::all()
                    .into_iter()
                    .filter(|l| {
                        l.to_string()
                            .to_lowercase()
                            .contains(&app.layout_search_query.to_lowercase())
                    })
                    .count();
                if filtered_count > 0 {
                    let current = app.layout_list_state.get_mut().selected().unwrap_or(0);
                    let next = if current + 5 < filtered_count {
                        current + 5
                    } else {
                        filtered_count - 1
                    };
                    app.layout_list_state.get_mut().select(Some(next));
                }
                return true;
            }
            KeyCode::Char(c) => {
                app.layout_search_query.push(c);
                app.layout_list_state.get_mut().select(Some(0)); // Reset selection on search
                return true;
            }
            KeyCode::Backspace => {
                app.layout_search_query.pop();
                app.layout_list_state.get_mut().select(Some(0)); // Reset selection on search
                return true;
            }
            _ => return true,
        }
    }

    match key.code {
        KeyCode::Char('h') => {
            app.dashboard_period = match app.dashboard_period {
                TimePeriod::Today => TimePeriod::Custom,
                TimePeriod::Yesterday => TimePeriod::Today,
                TimePeriod::Week => TimePeriod::Yesterday,
                TimePeriod::Month => TimePeriod::Week,
                TimePeriod::Year => TimePeriod::Month,
                TimePeriod::All => TimePeriod::Year,
                TimePeriod::Custom => TimePeriod::All,
            };
            if app.dashboard_period == TimePeriod::Custom {
                // Do not fetch immediately, user needs to pick date
            } else {
                fetch_heatmap(app);
            }
            true
        }
        KeyCode::Char('l') => {
            app.dashboard_period = match app.dashboard_period {
                TimePeriod::Today => TimePeriod::Yesterday,
                TimePeriod::Yesterday => TimePeriod::Week,
                TimePeriod::Week => TimePeriod::Month,
                TimePeriod::Month => TimePeriod::Year,
                TimePeriod::Year => TimePeriod::All,
                TimePeriod::All => TimePeriod::Custom,
                TimePeriod::Custom => TimePeriod::Today,
            };
            if app.dashboard_period == TimePeriod::Custom {
                // Do not fetch immediately
            } else {
                fetch_heatmap(app);
            }
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
                // Maybe handle other enters? For now just return false
                false
            }
        }
        KeyCode::Char('k') => {
            app.show_layout_popup = true;
            app.layout_search_query.clear();
            app.layout_list_state.get_mut().select(Some(0));
            true
        }
        _ => false,
    }
}

pub fn handle_date_picker_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.date_picker.open = false;
        }
        KeyCode::Left => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_sub_days(Days::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Right => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_add_days(Days::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Up => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_sub_days(Days::new(7))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Down => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_add_days(Days::new(7))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::PageUp => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_sub_months(Months::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::PageDown => {
            app.date_picker.current_selection = app
                .date_picker
                .current_selection
                .checked_add_months(Months::new(1))
                .unwrap_or(app.date_picker.current_selection);
        }
        KeyCode::Enter => match app.date_picker.selection_step {
            SelectionStep::Start => {
                app.date_picker.start_date = Some(app.date_picker.current_selection);
                app.date_picker.selection_step = SelectionStep::End;

                app.date_picker.current_selection = app
                    .date_picker
                    .current_selection
                    .checked_add_days(Days::new(1))
                    .unwrap_or(app.date_picker.current_selection);
            }
            SelectionStep::End => {
                let end = app.date_picker.current_selection;
                if let Some(start) = app.date_picker.start_date {
                    if end >= start {
                        app.date_picker.end_date = Some(end);
                        app.date_picker.open = false;
                    } else {
                        app.date_picker.start_date = Some(end);
                        app.date_picker.end_date = Some(start);
                        app.date_picker.open = false;
                    }
                } else {
                    app.date_picker.start_date = Some(end);
                    app.date_picker.selection_step = SelectionStep::End;
                }
            }
        },
        _ => {}
    }
}

fn fetch_heatmap(app: &App) {
    let period_str = match app.dashboard_period {
        TimePeriod::Today => "today".to_string(),
        TimePeriod::Yesterday => "yesterday".to_string(),
        TimePeriod::Week => "week".to_string(),
        TimePeriod::Month => "month".to_string(),
        TimePeriod::Year => "year".to_string(),
        TimePeriod::All => "all".to_string(),
        TimePeriod::Custom => {
            if let (Some(start), Some(end)) = (app.date_picker.start_date, app.date_picker.end_date)
            {
                format!("custom:{}:{}", start, end)
            } else {
                // Fallback to all if dates not selected yet
                "all".to_string()
            }
        }
    };
    crate::tui::app::spawn_fetch_keyboard_heatmap(app.client.clone(), app.tx.clone(), &period_str);
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(KEY_HEIGHT + 2),
            Constraint::Length(4),
        ])
        .split(area);

    render_statistics(f, chunks[0]);
    render_keyboard(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);

    if app.show_layout_popup {
        render_layout_popup(f, app, area);
    }

    if app.date_picker.open {
        render_date_picker(f, app, area);
    }
}

fn render_layout_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 50, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Select Layout ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    f.render_widget(block.clone(), popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // List
        ])
        .split(popup_area);

    // Search Bar
    let search_text = format!("Search: {}", app.layout_search_query);
    let search_p = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(search_p, chunks[0]);

    // List
    let filtered: Vec<KeyboardLayout> = KeyboardLayout::all()
        .into_iter()
        .filter(|l| {
            l.to_string()
                .to_lowercase()
                .contains(&app.layout_search_query.to_lowercase())
        })
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|l| ListItem::new(l.to_string()).style(Style::default().fg(Color::White)))
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut list_state = app.layout_list_state.borrow_mut();
    f.render_stateful_widget(list, chunks[1], &mut *list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_statistics(f: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::BOTTOM);
    f.render_widget(block, area);

    let inner = area.inner(ratatui::layout::Margin {
        vertical: 0,
        horizontal: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner);

    let stats = [
        ("Today", "13,831"),
        ("Yesterday", "6,283"),
        ("Unpulsed", "2,284"),
        ("All time", "3,186,900"),
    ];

    for (i, (label, value)) in stats.iter().enumerate() {
        let text = vec![
            Line::from(Span::styled(
                *label,
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                *value,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ];
        let p = Paragraph::new(text).alignment(Alignment::Center);
        f.render_widget(p, chunks[i]);
    }
}

fn render_keyboard(f: &mut Frame, app: &App, area: Rect) {
    // Combine API data with session data
    let mut data = app.heatmap_data.clone();
    for (k, v) in &app.session_heatmap {
        *data.entry(k.clone()).or_insert(0) += v;
    }

    if data.is_empty() {
        let p = Paragraph::new("No data available for this period")
            .block(Block::default().borders(Borders::ALL).title(" Keyboard "))
            .alignment(Alignment::Center);
        f.render_widget(p, area);
        return;
    }

    let max_count = data.values().max().copied().unwrap_or(1);

    // Center the keyboard in the available area
    // Keyboard size is roughly 74x15 (60 main + 2 gap + 12 nav)
    let kbd_width = 74;
    let kbd_height = 15;

    let x_offset = if area.width > kbd_width {
        area.x + (area.width - kbd_width) / 2
    } else {
        area.x
    };

    let y_offset = if area.height > kbd_height {
        area.y + (area.height - kbd_height) / 2
    } else {
        area.y
    };

    for key in app.keyboard_layout.get_keys() {
        // Calculate absolute position
        let x = x_offset + key.x;
        let y = y_offset + key.y;

        // Skip if out of bounds
        if x + key.width > area.x + area.width || y + KEY_HEIGHT > area.y + area.height {
            continue;
        }

        let rect = Rect::new(x, y, key.width, KEY_HEIGHT);

        // Determine color
        let k1 = &key.json_key;
        let k2 = &key.label.to_uppercase();

        // Try exact match, then label match
        let count = data.get(k1).or_else(|| data.get(k2)).copied().unwrap_or(0);

        let bg_color = get_color(count, max_count);

        // Determine text color for contrast
        // Simple heuristic: if background is dark, use white text, else black
        // Since our gradient starts dark blue and ends red, white is mostly safe.
        // But for very bright yellow/green, black might be better.
        // Let's stick to White for now as the requested gradient is mostly darkish/rich colors.
        let fg_color = Color::White;

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(bg_color).fg(fg_color));

        let p = Paragraph::new(key.label)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(p, rect);
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Controls with Top Border
            Constraint::Min(1),    // Status Bar
        ])
        .split(area);

    // 1. Controls
    let period_str = match app.dashboard_period {
        TimePeriod::Today => "Today",
        TimePeriod::Yesterday => "Yesterday",
        TimePeriod::Week => "Week",
        TimePeriod::Month => "Month",
        TimePeriod::Year => "Year",
        TimePeriod::All => "All Time",
        TimePeriod::Custom => "Custom",
    };
    let layout_text = format!(
        " Layout: {} (k) | Period: {} (h/l | /: Custom)",
        app.keyboard_layout, period_str
    );
    let p_controls = Paragraph::new(layout_text)
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Left);
    f.render_widget(p_controls, chunks[0]);

    // 2. Status / Error
    if let Some(err) = &app.error {
        // Show Error in Red at the bottom
        let err_text = format!("ERROR: {}", err);
        let p_err = Paragraph::new(err_text)
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Right);
        f.render_widget(p_err, chunks[1]);
    } else {
        // Show Status
        let map_len = app.heatmap_data.len();
        let max_val = app.heatmap_data.values().max().copied().unwrap_or(0);
        let session_len = app.session_heatmap.len();
        let session_max = app.session_heatmap.values().max().copied().unwrap_or(0);

        // Show full source string (includes error if fallback occurred)
        let source_str = &app.data_source;

        let status_text = if log::max_level() >= log::LevelFilter::Debug {
            format!(
                "Source: {} | Keys: {} (S: {}) | Max: {} (S: {})",
                source_str, map_len, session_len, max_val, session_max
            )
        } else {
            format!(
                "Keys: {} (+{}) | Max: {}",
                map_len,
                session_len,
                max_val.max(session_max)
            )
        };
        let p_status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right);
        f.render_widget(p_status, chunks[1]);
    }
}

fn get_color(count: u64, max: u64) -> Color {
    if max == 0 || count == 0 {
        return Color::Rgb(20, 20, 50); // Base Dark Blue for zero/empty
    }

    // Use Logarithmic scale for better visibility of lower frequency keys
    // Linear scale hides details because Space/E usually dominate by orders of magnitude.
    let log_count = (count as f64).ln();
    let log_max = (max as f64).ln();

    // Avoid division by zero if max is 1 (log(1) = 0)
    let ratio = if log_max <= 0.0 {
        1.0
    } else {
        (log_count / log_max).clamp(0.0, 1.0)
    };

    // Gradient:
    // 0.0 -> Dark Blue (20, 20, 50)
    // 0.5 -> Green/Yellow (50, 200, 50)
    // 1.0 -> Bright Red (255, 50, 50)

    let r: u8;
    let g: u8;
    let b: u8;

    if ratio < 0.5 {
        // Interpolate between Blue and Green
        let t = ratio * 2.0; // 0 to 1
        r = (20.0 + (50.0 - 20.0) * t) as u8;
        g = (20.0 + (200.0 - 20.0) * t) as u8;
        b = 50; // Blue (50) to Green (50) is constant
    } else {
        // Interpolate between Green and Red
        let t = (ratio - 0.5) * 2.0; // 0 to 1
        r = (50.0 + (255.0 - 50.0) * t) as u8;
        g = (200.0 + (50.0 - 200.0) * t) as u8;
        b = 50; // Green (50) to Red (50) is constant
    }

    Color::Rgb(r, g, b)
}

pub fn render_date_picker(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Date Picker ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    // Use fixed size for calendar (approx 40x16 is good for readability)
    let area = centered_fixed_area(40, 16, area);
    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // Header
    let current_month = app
        .date_picker
        .current_selection
        .format("%B %Y")
        .to_string();
    let step_msg = match app.date_picker.selection_step {
        SelectionStep::Start => "Select START Date",
        SelectionStep::End => "Select END Date",
    };

    let header_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(current_month, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled(step_msg, Style::default().fg(Color::Cyan)),
        ]))
        .alignment(Alignment::Center),
        header_layout[0],
    );

    // Calendar Grid
    let days_header = "Sun Mon Tue Wed Thu Fri Sat";
    f.render_widget(
        Paragraph::new(days_header).alignment(Alignment::Center),
        header_layout[1],
    );

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
    let mut current_date = first_day_of_month
        .checked_sub_days(Days::new(start_offset))
        .unwrap();
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
    f.render_widget(
        Paragraph::new("Arrows: Move | PgUp/Dn: Month | Enter: Select | Esc: Cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        footer_area,
    );
}

pub fn centered_fixed_area(width: u16, height: u16, area: Rect) -> Rect {
    let x = if area.width > width {
        (area.width - width) / 2
    } else {
        0
    };
    let y = if area.height > height {
        (area.height - height) / 2
    } else {
        0
    };

    Rect {
        x: area.x + x,
        y: area.y + y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}
