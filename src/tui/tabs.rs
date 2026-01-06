use crate::commands::get_pages;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let pages = get_pages();

    // Group pages by category
    // Order: General, Inputs, Stats, System
    let categories = [
        "Overview", "Input", "Network", "Uptime", "Settings", "Account", "Toys",
    ];
    let mut category_map: std::collections::HashMap<&str, Vec<usize>> =
        std::collections::HashMap::new();

    for (i, page) in pages.iter().enumerate() {
        category_map.entry(page.category).or_default().push(i);
    }

    // Determine active category based on current_tab
    let active_category_idx = categories
        .iter()
        .position(|cat| {
            if let Some(indices) = category_map.get(cat) {
                indices.contains(&app.nav.current_tab)
            } else {
                false
            }
        })
        .unwrap_or(0);

    // Dynamic Title
    let block_title = if let Some(user) = &app.user_stats {
        format!(" WhatPulse TUI | User: {} ", user.username)
    } else {
        " WhatPulse TUI ".to_string()
    };

    // Render Categories as Tabs
    let titles: Vec<Line> = categories
        .iter()
        .map(|cat| {
            let count = category_map.get(cat).map(|v| v.len()).unwrap_or(0);
            let label = if count > 1 {
                format!("{} ▼", cat)
            } else {
                cat.to_string()
            };
            Line::from(Span::styled(label, Style::default().fg(Color::Cyan)))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(block_title))
        .select(active_category_idx)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED));

    f.render_widget(tabs, area);
}

pub fn render_nav_popup(f: &mut Frame, app: &App, area: Rect) {
    use crate::commands::get_pages;
    use ratatui::widgets::{Clear, List, ListItem};

    let pages = get_pages();
    // Group pages by category
    let categories = [
        "Overview", "Input", "Network", "Uptime", "Settings", "Account", "Toys",
    ];
    let mut category_map: std::collections::HashMap<&str, Vec<usize>> =
        std::collections::HashMap::new();

    for (i, page) in pages.iter().enumerate() {
        category_map.entry(page.category).or_default().push(i);
    }

    // Determine active category based on current_tab
    let active_category_idx = categories
        .iter()
        .position(|cat| {
            if let Some(indices) = category_map.get(cat) {
                indices.contains(&app.nav.current_tab)
            } else {
                false
            }
        })
        .unwrap_or(0);

    let cat = categories[active_category_idx];
    if let Some(indices) = category_map.get(cat) {
        // Find screen position: exactly under the tab label
        // We sum the lengths of previous titles + 1 char for separator
        // + 1 char for the left border of the Tabs block.
        let mut x_offset = 1; // Left border
        for c in categories.iter().take(active_category_idx) {
            let count = category_map.get(*c).map(|v| v.len()).unwrap_or(0);
            let len = c.len() + if count > 1 { 2 } else { 0 }; // + " ▼"
            x_offset += (len as u16) + 1; // Title + Separator (|)
        }
        let x = area.x + x_offset;
        let y = area.y + 2; // Below tabs

        let width = 20;
        let height = (indices.len() as u16) + 2;

        // Ensure popup stays on screen
        let x = x.min(area.width.saturating_sub(width));

        let popup_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = indices
            .iter()
            .map(|&i| {
                let page = &pages[i];
                let style = if i == app.nav.current_tab {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(page.title).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(cat)
                    .style(Style::default().bg(Color::Black)), // Force opaque background
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_widget(list, popup_area);
    }
}
