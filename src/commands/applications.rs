use crate::commands::TuiPage;
use crate::tui::app::{App, AppSortMode, SortOrder};
use crate::tui::table_utils::{handle_table_nav, render_scrollbar};
use crate::tui::period_utils::{handle_period_nav, get_display_period, StatsTarget};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Cell},
    Frame,
};

inventory::submit! {
    TuiPage {
        title: "Applications",
        render: render_apps,
        handle_key: handle_apps_key,
        handle_mouse: handle_mouse,
        priority: 40,
    }
}

fn render_apps(f: &mut Frame, app: &App, area: Rect) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let header_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let row_highlight_style = Style::default().add_modifier(Modifier::REVERSED);

    let rows: Vec<Row> = app
        .app_stats
        .iter()
        .map(|stat| {
            Row::new(vec![
                stat.name.clone(),
                stat.keys.to_string(),
                stat.clicks.to_string(),
                stat.scrolls.to_string(),
                format!("{:.2} MB", stat.download_mb),
                format!("{:.2} MB", stat.upload_mb),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let period_str = get_display_period(app.app_stats_period);
    
    // Sort Indicator
    let sort_indicator = match app.app_sort_order {
        SortOrder::Ascending => "▲",
        SortOrder::Descending => "▼",
    };
    let sort_col = match app.app_sort_mode {
        AppSortMode::Keys => "Keys",
        AppSortMode::Clicks => "Clicks",
        AppSortMode::Scrolls => "Scrolls",
        AppSortMode::Download => "Download",
        AppSortMode::Upload => "Upload",
        AppSortMode::Name => "Name",
    };

    let title = format!(
        " Application Usage - {} (h/l: Period, s: Sort [{} {}], /: Date) ", 
        period_str, sort_col, sort_indicator
    );

    // Dynamic Header with Indicator
    let headers = vec![
        "Application", "Keys", "Clicks", "Scrolls", "Download", "Upload"
    ];
    let header_cells = headers.iter().map(|h| {
        let mut content = h.to_string();
        let is_sorted = match (app.app_sort_mode, h) {
            (AppSortMode::Name, &"Application") => true,
            (AppSortMode::Keys, &"Keys") => true,
            (AppSortMode::Clicks, &"Clicks") => true,
            (AppSortMode::Scrolls, &"Scrolls") => true,
            (AppSortMode::Download, &"Download") => true,
            (AppSortMode::Upload, &"Upload") => true,
            _ => false,
        };
        if is_sorted {
            content = format!("{} {}", h, sort_indicator);
        }
        Cell::from(content).style(header_style)
    });

    let table = Table::new(rows, widths)
        .header(
            Row::new(header_cells)
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .row_highlight_style(row_highlight_style)
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, chunks[0], &mut app.apps_table_state.borrow_mut());

    render_scrollbar(f, app, chunks[0], app.app_stats.len(), &mut app.apps_table_state.borrow_mut());

    if app.date_picker.open {
        crate::tui::ui::render_date_picker(f, app, area);
    }
}

fn handle_apps_key(app: &mut App, key: KeyEvent) -> bool {
    // Handle period navigation (h, l, /)
    if handle_period_nav(app, key, StatsTarget::Applications) {
        return true;
    }

    match key.code {
        KeyCode::Char('s') => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+s: Toggle Order
                app.app_sort_order = match app.app_sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
            } else {
                // s: Cycle Mode
                app.app_sort_mode = match app.app_sort_mode {
                    AppSortMode::Keys => AppSortMode::Clicks,
                    AppSortMode::Clicks => AppSortMode::Scrolls,
                    AppSortMode::Scrolls => AppSortMode::Download,
                    AppSortMode::Download => AppSortMode::Upload,
                    AppSortMode::Upload => AppSortMode::Name,
                    AppSortMode::Name => AppSortMode::Keys,
                };
                if app.app_sort_mode == AppSortMode::Name {
                    app.app_sort_order = SortOrder::Ascending;
                } else {
                    app.app_sort_order = SortOrder::Descending;
                }
            }
            app.sort_app_stats();
            true
        }
        KeyCode::Char('o') => {
             app.app_sort_order = match app.app_sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
            app.sort_app_stats();
            true
        }
        _ => {
            let len = app.app_stats.len();
            handle_table_nav(&mut app.apps_table_state.borrow_mut(), key.code, len)
        }
    }
}

fn handle_mouse(app: &mut App, event: crossterm::event::MouseEvent) -> bool {
    use crossterm::event::MouseEventKind;
    let len = app.app_stats.len();
    if len == 0 {
        return false;
    }

    match event.kind {
        MouseEventKind::ScrollDown => {
            handle_table_nav(&mut app.apps_table_state.borrow_mut(), KeyCode::Down, len)
        }
        MouseEventKind::ScrollUp => {
            handle_table_nav(&mut app.apps_table_state.borrow_mut(), KeyCode::Up, len)
        }
        _ => false,
    }
}
