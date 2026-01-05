use crate::commands::TuiPage;
use crate::tui::app::App;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Gauge, Paragraph},
};

pub mod landmarks;
pub use landmarks::LANDMARKS;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let current_height = app.scroll_meters;

    // -------------------------------------------------------------------------
    // Helper: Dynamic Atmosphere Color
    // -------------------------------------------------------------------------
    let atmosphere_style = if current_height < 2_000.0 {
        // ... (truncated for brevity in diff match, but need to be careful)
        // I will just match the start and end of this block to simplify
        // Troposphere: Cyan/Light Blue
        Style::default().fg(Color::Cyan)
    } else if current_height < 12_000.0 {
        // Troposphere/Stratosphere: Blue
        Style::default().fg(Color::Blue)
    } else if current_height < 50_000.0 {
        // Stratosphere: Dark Blue / Magenta transition
        Style::default().fg(Color::Magenta)
    } else if current_height < 80_000.0 {
        // Mesosphere: Dark Gray / Black
        Style::default().fg(Color::DarkGray)
    } else {
        // Space: White on Black (or just white text)
        Style::default().fg(Color::White)
    };

    // Find next landmark for "Next: ..."
    // The previously found 'target_landmark' is the "current active" one we are climbing towards OR just conquered.
    // If we conquered it (current >= height), we are technically looking for the NEXT one.

    let active_landmark_idx = LANDMARKS
        .iter()
        .position(|l| l.height_meters > current_height)
        .unwrap_or(LANDMARKS.len() - 1);
    let active_landmark = &LANDMARKS[active_landmark_idx];

    // Re-calc progress based on PREVIOUS landmark (to show 0-100% between distinct steps)
    let previous_landmark_height = if active_landmark_idx > 0 {
        LANDMARKS[active_landmark_idx - 1].height_meters
    } else {
        0.0
    };

    let leg_height = active_landmark.height_meters - previous_landmark_height;
    let current_leg_progress = current_height - previous_landmark_height;
    let progress_ratio = (current_leg_progress / leg_height).clamp(0.0, 1.0);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Art (Flexible)
            Constraint::Length(1), // Description
            Constraint::Length(3), // Progress Bar
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // 1. Header
    // Title: "Scroll Tower" + Altitude
    // Right: Frenzy Status

    let title_text = format!("The Scroll Tower | Altitude: {:.2} m", current_height);

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(atmosphere_style)
        .title_style(atmosphere_style.add_modifier(Modifier::BOLD));

    let header_paragraph = Paragraph::new(title_text)
        .block(header_block)
        .alignment(Alignment::Center);

    f.render_widget(header_paragraph, chunks[0]);

    // 2. ASCII Art
    // Show the ACTIVE landmark art (the one we are approaching)
    // If we are VERY close (e.g. > 90% of the way there), maybe show it?
    // Or just always show what we are climbing towards.
    let art_lines: Vec<Line> = active_landmark.ascii_art.lines().map(Line::from).collect();

    let art_widget = Paragraph::new(art_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(art_widget, chunks[1]);

    // 3. Description
    let desc_text = format!(
        "Target: {} ({:.0}m)",
        active_landmark.name, active_landmark.height_meters
    );
    let desc = Paragraph::new(desc_text)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    f.render_widget(desc, chunks[2]);

    // 4. Progress Bar
    let landmark_label = format!("{:.1}% to {}", progress_ratio * 100.0, active_landmark.name);
    let landmark_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(atmosphere_style)
        .ratio(progress_ratio)
        .label(landmark_label);

    f.render_widget(landmark_gauge, chunks[3]);

    // 5. Footer
    let stats_desc = active_landmark.description;
    let mode_text = match app.scroll_mode {
        crate::tui::app::ScrollMode::Lifetime => "Lifetime",
        crate::tui::app::ScrollMode::Session => "Session",
    };
    let footer_text = format!("Mode: {} | \"{}\"", mode_text, stats_desc);
    let footer = Paragraph::new(footer_text)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[4]);
}

pub fn handle_key(_app: &mut App, _key: KeyEvent) -> bool {
    false
}

inventory::submit! {
    TuiPage {
        title: "Scroll Tower",
        render,
        handle_key,
        handle_mouse: crate::commands::default_handle_mouse,
        priority: 16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_height_calculation() {
        // 1 tick = 0.016m (1.6cm)
        let base_meters_per_tick = 0.016;
        assert_eq!(base_meters_per_tick, 0.016);

        let scrolls = 100;
        let height = scrolls as f64 * base_meters_per_tick;
        assert_eq!(height, 1.6);
    }

    #[test]
    fn test_landmark_registry() {
        assert!(LANDMARKS.len() >= 26);
        assert_eq!(LANDMARKS[0].name, "Rubber Duck 🦆");
        assert_eq!(LANDMARKS[0].height_meters, 0.1);
        assert!(LANDMARKS[0].description.contains("best listener"));

        // Ensure sorted by height implicitly or check logic
        let mut prev_height = 0.0;
        for landmark in LANDMARKS {
            assert!(
                landmark.height_meters > prev_height,
                "Landmark {} is not sorted correctly",
                landmark.name
            );
            prev_height = landmark.height_meters;
        }
    }
}
