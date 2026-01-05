pub mod widget;

use crate::commands::TuiPage;
use crate::tui::app::{App, SelectionStep, TimePeriod};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};
use widget::AsciiHeatmap;

inventory::submit! {
    TuiPage {
        title: "Mouse",
        render: render_tui,
        handle_key,
        handle_mouse,
        priority: 12,
    }
}

fn handle_mouse(app: &mut App, event: crossterm::event::MouseEvent) -> bool {
    use crossterm::event::MouseEventKind;
    match event.kind {
        MouseEventKind::ScrollDown => {
            app.mouse_period = match app.mouse_period {
                TimePeriod::Today => TimePeriod::Yesterday,
                TimePeriod::Yesterday => TimePeriod::Week,
                TimePeriod::Week => TimePeriod::Month,
                TimePeriod::Month => TimePeriod::Year,
                TimePeriod::Year => TimePeriod::All,
                TimePeriod::All => TimePeriod::Custom,
                TimePeriod::Custom => TimePeriod::Today,
            };
            if app.mouse_period != TimePeriod::Custom {
                fetch_mouse_heatmap(app);
            }
            true
        }
        MouseEventKind::ScrollUp => {
            app.mouse_period = match app.mouse_period {
                TimePeriod::Today => TimePeriod::Custom,
                TimePeriod::Custom => TimePeriod::All,
                TimePeriod::All => TimePeriod::Year,
                TimePeriod::Year => TimePeriod::Month,
                TimePeriod::Month => TimePeriod::Week,
                TimePeriod::Week => TimePeriod::Yesterday,
                TimePeriod::Yesterday => TimePeriod::Today,
            };
            if app.mouse_period != TimePeriod::Custom {
                fetch_mouse_heatmap(app);
            }
            true
        }
        _ => false,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if app.date_picker.open {
        crate::commands::keyboard::handle_date_picker_key(app, key);
        if !app.date_picker.open {
            // If closed, fetch heatmap with new range if custom
            fetch_mouse_heatmap(app);
        }
        return true;
    }

    if app.show_mouse_stats {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('m') || key.code == KeyCode::Enter
        {
            app.show_mouse_stats = false;
        }
        return true;
    }

    match key.code {
        KeyCode::Esc => true,
        KeyCode::Char('m') => {
            app.show_mouse_stats = true;
            true
        }
        KeyCode::Char('h') => {
            app.mouse_period = match app.mouse_period {
                TimePeriod::Today => TimePeriod::Yesterday,
                TimePeriod::Yesterday => TimePeriod::Week,
                TimePeriod::Week => TimePeriod::Month,
                TimePeriod::Month => TimePeriod::Year,
                TimePeriod::Year => TimePeriod::All,
                TimePeriod::All => TimePeriod::Custom,
                TimePeriod::Custom => TimePeriod::Today,
            };
            if app.mouse_period != TimePeriod::Custom {
                fetch_mouse_heatmap(app);
            }
            true
        }
        KeyCode::Char('l') => {
            app.mouse_period = match app.mouse_period {
                TimePeriod::Today => TimePeriod::Custom,
                TimePeriod::Custom => TimePeriod::All,
                TimePeriod::All => TimePeriod::Year,
                TimePeriod::Year => TimePeriod::Month,
                TimePeriod::Month => TimePeriod::Week,
                TimePeriod::Week => TimePeriod::Yesterday,
                TimePeriod::Yesterday => TimePeriod::Today,
            };
            if app.mouse_period != TimePeriod::Custom {
                fetch_mouse_heatmap(app);
            }
            true
        }
        KeyCode::Char('/') | KeyCode::Enter if app.mouse_period == TimePeriod::Custom => {
            app.date_picker.open = true;
            app.date_picker.selection_step = SelectionStep::Start;
            // Initialize selection to today if not set, or keep current
            if app.date_picker.start_date.is_none() {
                app.date_picker.current_selection = chrono::Local::now().naive_local().date();
            }
            true
        }
        _ => false,
    }
}

fn fetch_mouse_heatmap(app: &App) {
    let period_str = match app.mouse_period {
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
                "all".to_string()
            }
        }
    };
    crate::tui::app::spawn_fetch_mouse_heatmap(app.client.clone(), app.tx.clone(), &period_str);
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(3), // Footer
        ])
        .split(area);

    let heatmap_area = chunks[0];
    let footer_area = chunks[1];

    let data = &app.screen_heatmap_data;

    if !data.is_empty() {
        let heatmap = AsciiHeatmap::new(data)
            .block(Block::default().borders(Borders::ALL).title(" Mouse "))
            .use_color(true)
            .show_axes(true);
        f.render_widget(heatmap, heatmap_area);
    } else {
        let p = Paragraph::new("No data available for this period")
            .block(Block::default().borders(Borders::ALL).title(" Mouse "))
            .alignment(Alignment::Center);
        f.render_widget(p, heatmap_area);
    }

    render_footer(f, app, footer_area);

    if app.date_picker.open {
        crate::commands::keyboard::render_date_picker(f, app, area);
    }

    if app.show_mouse_stats {
        render_mouse_stats_popup(f, app, area);
    }
}

fn render_mouse_stats_popup(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Mouse Stats ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    // Fixed size popup
    let popup_area = crate::commands::keyboard::centered_fixed_area(40, 20, area);

    f.render_widget(Clear, popup_area);
    f.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Stats text
            Constraint::Min(10),   // Art
        ])
        .split(inner);

    let stats = &app.mouse_stats;
    let text = [
        format!("Today:     {:>6}", stats.today.clicks),
        format!("Yesterday: {:>6}", stats.yesterday.clicks),
        format!("Unpulsed:  {:>6}", stats.unpulsed.clicks),
        format!("All Time:  {:>6}", stats.all_time.clicks),
        String::new(),
        format!("Scrolls:   {:>6}", stats.all_time.scrolls),
        format!("Dist:      {:.2}m", stats.all_time.distance_meters),
    ]
    .join("\n");

    let p = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(p, chunks[0]);

    // Mouse Art
    let total = stats.all_time.clicks as f64;
    let buttons = &stats.all_time.clicks_by_button;
    let l = *buttons.get(&1).unwrap_or(&0) as f64;
    let m = *buttons.get(&2).unwrap_or(&0) as f64;
    let r = *buttons.get(&3).unwrap_or(&0) as f64;

    let lp = if total > 0.0 { l / total * 100.0 } else { 0.0 };
    let mp = if total > 0.0 { m / total * 100.0 } else { 0.0 };
    let rp = if total > 0.0 { r / total * 100.0 } else { 0.0 };

    let l_str = format!("{:.0}%", lp);
    let m_str = format!("{:.0}%", mp);
    let r_str = format!("{:.0}%", rp);

    let art = format!(
        "_.--\"\"\"\"--._       \n\
        .'     |   |    '.      \n\
       /  {:^4}|{:^4}|{:^4}  \\     \n\
      |      |    |       |    \n\
      |      |____|       |    \n\
      |                  |    \n\
      |                  |    \n\
      |                  |    \n\
       \\                /     \n\
        '.            .'      \n\
          '--......--'        \n",
        l_str, m_str, r_str
    );

    let p_art = Paragraph::new(art).alignment(Alignment::Center);
    f.render_widget(p_art, chunks[1]);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let period_str = match app.mouse_period {
        TimePeriod::Today => "Today",
        TimePeriod::Yesterday => "Yesterday",
        TimePeriod::Week => "Week",
        TimePeriod::Month => "Month",
        TimePeriod::Year => "Year",
        TimePeriod::All => "All Time",
        TimePeriod::Custom => "Custom",
    };
    let controls_text = format!(" Period: {} (h/l | /: Custom | m: Mouse Stats)", period_str);

    let block = Block::default().borders(Borders::TOP);
    let p = Paragraph::new(controls_text)
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}
