use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, TableState},
    Frame,
};
use crate::tui::app::App;

pub fn render_scrollbar(f: &mut Frame, _app: &App, area: Rect, len: usize, state: &mut TableState) {
    let mut scroll_state = ScrollbarState::default()
        .content_length(len)
        .position(state.selected().unwrap_or(0));
    
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼")),
        area,
        &mut scroll_state,
    );
}

pub fn handle_table_nav(state: &mut TableState, key: KeyCode, len: usize) -> bool {
    if len == 0 {
        return false;
    }

    match key {
        KeyCode::Down | KeyCode::Char('j') => {
            let i = match state.selected() {
                Some(i) => {
                    if i >= len.saturating_sub(1) {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            state.select(Some(i));
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let i = match state.selected() {
                Some(i) => {
                    if i == 0 {
                        len.saturating_sub(1)
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            state.select(Some(i));
            true
        }
        KeyCode::PageDown => {
            let current = state.selected().unwrap_or(0);
            let next = (current + 10).min(len.saturating_sub(1));
            state.select(Some(next));
            true
        }
        KeyCode::PageUp => {
            let current = state.selected().unwrap_or(0);
            let next = current.saturating_sub(10);
            state.select(Some(next));
            true
        }
        KeyCode::Home => {
            state.select(Some(0));
            true
        }
        KeyCode::End => {
            state.select(Some(len.saturating_sub(1)));
            true
        }
        _ => false,
    }
}
