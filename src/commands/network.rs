use crate::commands::TuiPage;
use crate::tui::app::{App, NetworkSortMode, SortOrder};
use crate::tui::table_utils::{handle_table_nav, render_scrollbar};
use crate::tui::period_utils::{handle_period_nav, get_display_period, StatsTarget};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, Cell},
};

inventory::submit! {
    TuiPage {
        title: "Network",
        render: render_network,
        handle_key: handle_network_key,
        handle_mouse: handle_mouse,
        priority: 50,
    }
}

fn render_network(f: &mut Frame, app: &App, area: Rect) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let header_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let row_highlight_style = Style::default().add_modifier(Modifier::REVERSED);

    let rows: Vec<Row> = app
        .network_stats
        .iter()
        .map(|stat| {
            Row::new(vec![
                stat.interface.clone(),
                format!("{:.2} MB", stat.download_mb),
                format!("{:.2} MB", stat.upload_mb),
                format!("{:.2} MB", stat.download_mb + stat.upload_mb),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let period_str = get_display_period(app.network_stats_period);
    
    // Sort Indicator
    let sort_indicator = match app.network_sort_order {
        SortOrder::Ascending => "▲",
        SortOrder::Descending => "▼",
    };
    let sort_col = match app.network_sort_mode {
        NetworkSortMode::Download => "Download",
        NetworkSortMode::Upload => "Upload",
        NetworkSortMode::Total => "Total",
        NetworkSortMode::Interface => "Interface",
    };

    let title = format!(
        " Network Activity - {} (h/l: Period, s: Sort [{} {}], /: Date) ", 
        period_str, sort_col, sort_indicator
    );

    // Dynamic Header with Indicator
    let headers = vec![
        "Interface", "Download", "Upload", "Total"
    ];
    let header_cells = headers.iter().map(|h| {
        let mut content = h.to_string();
        let is_sorted = match (app.network_sort_mode, h) {
            (NetworkSortMode::Interface, &"Interface") => true,
            (NetworkSortMode::Download, &"Download") => true,
            (NetworkSortMode::Upload, &"Upload") => true,
            (NetworkSortMode::Total, &"Total") => true,
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

    f.render_stateful_widget(table, chunks[0], &mut app.network_table_state.borrow_mut());

    render_scrollbar(f, app, chunks[0], app.network_stats.len(), &mut app.network_table_state.borrow_mut());

    if app.date_picker.open {
        crate::tui::ui::render_date_picker(f, app, area);
    }
}

fn handle_network_key(app: &mut App, key: KeyEvent) -> bool {
    // Handle period navigation (h, l, /)
    if handle_period_nav(app, key, StatsTarget::Network) {
        return true;
    }

    match key.code {
        KeyCode::Char('s') => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+s: Toggle Order
                app.network_sort_order = match app.network_sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
            } else {
                // s: Cycle Mode
                app.network_sort_mode = match app.network_sort_mode {
                    NetworkSortMode::Download => NetworkSortMode::Upload,
                    NetworkSortMode::Upload => NetworkSortMode::Total,
                    NetworkSortMode::Total => NetworkSortMode::Interface,
                    NetworkSortMode::Interface => NetworkSortMode::Download,
                };
                if app.network_sort_mode == NetworkSortMode::Interface {
                    app.network_sort_order = SortOrder::Ascending;
                } else {
                    app.network_sort_order = SortOrder::Descending;
                }
            }
            app.sort_network_stats();
            true
        }
        KeyCode::Char('o') => {
             app.network_sort_order = match app.network_sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
            app.sort_network_stats();
            true
        }
        _ => {
            let len = app.network_stats.len();
            handle_table_nav(&mut app.network_table_state.borrow_mut(), key.code, len)
        }
    }
}

fn handle_mouse(app: &mut App, event: crossterm::event::MouseEvent) -> bool {
    use crossterm::event::MouseEventKind;
    let len = app.network_stats.len();
    if len == 0 {
        return false;
    }

    match event.kind {
        MouseEventKind::ScrollDown => handle_table_nav(
            &mut app.network_table_state.borrow_mut(),
            KeyCode::Down,
            len,
        ),
        MouseEventKind::ScrollUp => {
            handle_table_nav(&mut app.network_table_state.borrow_mut(), KeyCode::Up, len)
        }
        _ => false,
    }
}
