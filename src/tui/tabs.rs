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
    let titles: Vec<Line> = pages
        .iter()
        .map(|page| Line::from(Span::styled(page.title, Style::default().fg(Color::Cyan))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" WhatPulse TUI "),
        )
        .select(app.current_tab)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED));

    f.render_widget(tabs, area);
}
