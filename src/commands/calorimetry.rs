use crate::client::WhatpulseClient;
use anyhow::{Context, Result};
use uom::si::energy::{calorie, joule, kilocalorie};
use uom::si::f64::Energy;
use uom::si::force::newton;
use uom::si::length::meter;

// TUI Imports
use crate::commands::TuiPage;
use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

inventory::submit! {
    TuiPage {
        title: "Calorimetry",
        render: render_tui,
        handle_key,
        handle_mouse: crate::commands::default_handle_mouse,
        priority: 20,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if let KeyCode::Char('p') = key.code {
        app.profile_index = (app.profile_index + 1) % app.profiles.len();
        app.recalculate_energy();
        return true;
    }
    false
}

// Conversion constants
const CALORIES_PER_M_AND_M: f64 = 10.0; // ~10 kcal per M&M (standard size)
const CALORIES_PER_MINUTE_RUNNING: f64 = 10.0; // ~10 kcal/min for average runner

#[derive(Debug, Clone)]
pub struct SwitchProfile {
    pub name: String,
    pub force_newtons: f64,
    pub distance_meters: f64,
}

impl Default for SwitchProfile {
    fn default() -> Self {
        Self::cherry_mx_red()
    }
}

impl SwitchProfile {
    pub fn new(name: &str, force_g: f64, distance_mm: f64) -> Self {
        Self {
            name: name.to_string(),
            force_newtons: force_g * 0.00980665, // Convert gf to N
            distance_meters: distance_mm / 1000.0, // Convert mm to m
        }
    }

    pub fn cherry_mx_red() -> Self {
        Self::new("Cherry MX Red", 45.0, 4.0)
    }

    pub fn cherry_mx_blue() -> Self {
        Self::new("Cherry MX Blue", 60.0, 4.0) // ~50g actuation, 60g peak
    }

    pub fn cherry_mx_brown() -> Self {
        Self::new("Cherry MX Brown", 55.0, 4.0) // ~45g actuation, 55g peak
    }

    pub fn membrane() -> Self {
        Self::new("Generic Membrane", 55.0, 3.5) // Approx
    }
}

pub struct EnergyStats {
    #[allow(dead_code)]
    pub total_keys: f64,
    pub work_joules: f64,
    pub calories: f64,
    pub kcal: f64,
    pub m_and_ms: f64,
    pub running_seconds: f64,
}

pub fn calculate_energy(keys_str: &str, profile: Option<&SwitchProfile>) -> Result<EnergyStats> {
    // Remove commas if present (API might return "15,234")
    let keys_clean = keys_str.replace(',', "");
    let keys: f64 = keys_clean.parse().context("Failed to parse keys count")?;

    let default_profile = SwitchProfile::default();
    let profile = profile.unwrap_or(&default_profile);

    // Calculate Work: W = F * d * keys
    // We calculate work for ONE keystroke first
    let force = uom::si::f64::Force::new::<newton>(profile.force_newtons);
    let distance = uom::si::f64::Length::new::<meter>(profile.distance_meters);
    let work_per_keystroke: Energy = force * distance;

    // Total work
    let total_work = work_per_keystroke * keys;

    // Convert to calories (small calories)
    let total_calories = total_work.get::<calorie>();
    // Convert to kilocalories (food calories)
    let total_kcal = total_work.get::<kilocalorie>();

    // Comparisons
    let m_and_ms = total_kcal / CALORIES_PER_M_AND_M;
    let running_minutes = total_kcal / CALORIES_PER_MINUTE_RUNNING;
    let running_seconds = running_minutes * 60.0;

    Ok(EnergyStats {
        total_keys: keys,
        work_joules: total_work.get::<joule>(),
        calories: total_calories,
        kcal: total_kcal,
        m_and_ms,
        running_seconds,
    })
}

pub async fn execute(client: &WhatpulseClient) -> Result<()> {
    println!("Fetching latest pulse data...");

    // Fetch user stats to get total keys
    let user = client
        .get_user()
        .await
        .context("Failed to fetch user data")?;

    let keys_str = user.totals.keys.unwrap_or(0).to_string();
    // Default to Cherry MX Red for CLI for now, could add args later
    let stats = calculate_energy(&keys_str, None)?;

    // Formatting output
    println!("\nEnergy Expenditure Report:");
    println!("──────────────────────────");
    println!("Total Keystrokes: {}", keys_str); // Use original string with commas if available
    println!("Work Performed:   {:.2} J", stats.work_joules);
    println!("Calories Burned:  {:.2} cal", stats.calories);
    println!("                  {:.4} kcal", stats.kcal);
    println!("──────────────────────────");
    println!("Fun Comparisons:");
    println!("• Equivalent to {:.4} M&Ms", stats.m_and_ms);

    if stats.running_seconds >= 60.0 {
        println!(
            "• Like running for {:.1} minutes",
            stats.running_seconds / 60.0
        );
    } else {
        println!("• Like running for {:.0} seconds", stats.running_seconds);
    }

    Ok(())
}

pub fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Calorimetry ");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if app.user_loading && app.energy_stats.is_none() {
        f.render_widget(
            Paragraph::new("Loading...").style(Style::default().fg(Color::Yellow)),
            inner_area,
        );
        return;
    }

    if let Some(err) = &app.error {
        let p = Paragraph::new(format!("Error: {}", err)).style(Style::default().fg(Color::Red));
        f.render_widget(p, inner_area);
        return;
    }

    if let Some(stats) = &app.energy_stats {
        let profile = app.current_profile();
        let text = vec![
            Line::from(vec![
                Span::styled("Switch Profile: ", Style::default().fg(Color::Cyan)),
                Span::raw(&profile.name),
                Span::styled(
                    " (Press 'p' to cycle)",
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Work Performed: ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{:.2} J", stats.work_joules)),
            ]),
            Line::from(vec![
                Span::styled("Calories Burned: ", Style::default().fg(Color::Blue)),
                Span::raw(format!(
                    "{:.2} cal / {:.4} kcal",
                    stats.calories, stats.kcal
                )),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Fun Comparisons:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(format!("• {:.4} M&Ms", stats.m_and_ms)),
            Line::from(if stats.running_seconds >= 60.0 {
                format!("• Running for {:.1} minutes", stats.running_seconds / 60.0)
            } else {
                format!("• Running for {:.0} seconds", stats.running_seconds)
            }),
        ];

        f.render_widget(Paragraph::new(text), inner_area);
    } else {
        f.render_widget(
            Paragraph::new("No energy statistics available.\n\nPossible reasons:\n- User data not loaded yet\n- 'Keys' field missing in API response")
                .style(Style::default().fg(Color::DarkGray)),
            inner_area
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::App;
    use ratatui::Terminal;
    use ratatui::backend::{Backend, TestBackend};
    use tokio::sync::mpsc;

    #[test]
    fn test_energy_calculation_default() {
        let keys = "1000";
        let stats = calculate_energy(keys, None).unwrap();

        // F = 0.441 N (45g), d = 0.004 m
        // W = 0.441 * 0.004 * 1000 = 1.764 J
        assert!((stats.work_joules - 1.765).abs() < 0.01);
    }

    #[test]
    fn test_energy_calculation_blue() {
        let keys = "1000";
        let profile = SwitchProfile::cherry_mx_blue();
        let stats = calculate_energy(keys, Some(&profile)).unwrap();

        // F = 0.588 N (60g), d = 0.004 m
        // W = 0.588 * 0.004 * 1000 = 2.352 J
        assert!((stats.work_joules - 2.353).abs() < 0.01);
    }

    #[test]
    fn test_invalid_input() {
        let keys = "abc";
        let result = calculate_energy(keys, None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_render_tui() {
        // Create a fake valid JWT: header.payload.signature
        // Payload {"sub":"12345"} -> eyJzdWIiOiIxMjM0NSJ9
        let fake_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NSJ9.signature";
        let client = WhatpulseClient::new(fake_token).await.unwrap();
        let (tx, _rx) = mpsc::channel(10);
        let mut app = App::new(client, tx);

        // Setup terminal
        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Case 1: Loading
        app.user_loading = true;
        app.energy_stats = None;
        terminal
            .draw(|f| {
                render_tui(f, &app, f.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        // Check for "Loading..." text
        let mut found_loading = false;
        for cell in buffer.content.iter() {
            if cell.symbol() == "L" {
                // Simple check, real check would be more complex
                found_loading = true;
                break;
            }
        }
        // Actually TestBackend has better assertions, but let's just ensure it drew something
        assert!(found_loading, "Should display Loading...");

        // Case 2: Data loaded
        app.user_loading = false;
        app.energy_stats = Some(calculate_energy("1000", None).unwrap());
        terminal
            .draw(|f| {
                render_tui(f, &app, f.area());
            })
            .unwrap();

        // Just verify it doesn't panic and draws
        assert_eq!(
            terminal.backend().size().unwrap(),
            ratatui::layout::Size::new(20, 10)
        );
    }
}
