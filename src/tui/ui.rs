use crate::commands::get_pages;
use crate::tui::app::{App, SelectionStep};
use crate::tui::tabs;
use chrono::{Datelike, Days, NaiveDate};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
        ])
        .split(f.area());

    tabs::render(f, app, chunks[0]);

    if let Some(page) = get_pages().get(app.current_tab) {
        (page.render)(f, app, chunks[1]);
    }

    if app.date_picker.open {
        render_date_picker(f, app, f.area());
    }

    if let Some(err) = &app.error {
        let area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, area); // Clear background

        let block = Block::default()
            .title(" Error ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));

        let p = Paragraph::new(err.clone())
            .block(block)
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });

        f.render_widget(p, area);
    }

    // Render Notification Popup
    if let Some((msg, time)) = &app.notification
        && time.elapsed() < std::time::Duration::from_secs(3)
    {
        let area = centered_fixed_area(60, 5, f.area());
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(" Notification ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let p = Paragraph::new(msg.clone())
            .block(block)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(p, area);
    }
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
