use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use crate::tui::app::App;
use crate::tui::tabs;
use crate::commands::get_pages;

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
}
