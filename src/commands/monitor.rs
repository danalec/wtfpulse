use crate::client::WhatpulseClient;
use crate::commands::TuiPage;
use crate::tui::app::{Action, App, MonitorCommand, RealtimeData, UnitSystem};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use futures_util::{SinkExt, StreamExt};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

// Constants
// Gauge Scale Physics:
// - Modern Burst (305 WPM Raw): ~25.4 KPS * 0.0024 J ≈ 0.061 W.
// - Historical Sustained (Stella Pajunas, 1946): 216 WPM (~11.8 KPS effective) ≈ 0.028 W.
// - Barbara Blackburn (Dvorak): 212 WPM (~11.6 KPS) ≈ 0.027 W.
// We set the scale to 0.065 W (65mW) to perfectly frame the absolute human peak (305 WPM Raw).
const MAX_GAUGE_POWER_WATTS: f64 = 0.065;
const HEALTH_LIMIT_JOULES_PER_HOUR: f64 = 50.0; // Approx 70 WPM sustained

inventory::submit! {
    TuiPage {
        title: "Kinetic",
        render: render_tui,
        handle_key,
        priority: 30,
    }
}

#[derive(Serialize, Debug)]
struct WpWebSocketRequest {
    source: String,
    action: String,
}

#[derive(Deserialize, Debug)]
struct WpWebSocketMsg {
    action: String,
    data: Option<WpDataResponse>,
}

use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct WpDataResponse {
    #[serde(rename = "account-totals")]
    _account_totals: Option<serde_json::Value>,
    realtime: Option<WpRealtime>,
    unpulsed: Option<WpUnpulsed>,
    heatmap: Option<HashMap<String, u64>>,
}

#[derive(Deserialize, Debug)]
struct WpRealtime {
    keys: String, // "0.00" or "0,00"
    #[serde(rename = "clicks")]
    _clicks: String,
}

#[derive(Deserialize, Debug)]
struct WpUnpulsed {
    keys: i64,
    clicks: i64,
    #[serde(default)]
    scrolls: i64,
}

// Helper to parse localized float strings (e.g. "2,17" or "2.17")
fn parse_localized_float(s: &str) -> f64 {
    let normalized = s.replace(',', ".");
    normalized.parse::<f64>().unwrap_or(0.0)
}

// CLI Execution (Streaming Mode)
pub async fn execute(_client: &WhatpulseClient) -> Result<()> {
    let url = url::Url::parse("ws://127.0.0.1:3489")?;
    println!("Connecting to {}...", url);

    let (mut ws_stream, _) = connect_async(url.to_string()).await?;
    println!("Connected! Sending subscription...");

    // Subscribe to realtime stats
    ws_stream
        .send(Message::Text("/v1/realtime".to_string().into()))
        .await?;

    let (_, mut read) = ws_stream.split();

    println!("Listening for pulses...");
    println!("Press Ctrl+C to exit.");

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        println!("{}", text);
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("Connection closed.");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\nExiting...");
                break;
            }
        }
    }
    Ok(())
}

// Background Task for TUI
pub async fn spawn_monitor_task(
    tx: tokio::sync::mpsc::Sender<Action>,
    mut rx_cmd: tokio::sync::mpsc::Receiver<MonitorCommand>,
) {
    let url = url::Url::parse("ws://127.0.0.1:3489").unwrap();
    loop {
        // let _ = tx.send(Action::DebugInfo(format!("Connecting to {}...", url))).await;
        match connect_async(url.to_string()).await {
            Ok((ws_stream, _)) => {
                let _ = tx.send(Action::WebSocketStatus(true, None)).await;
                // let _ = tx.send(Action::DebugInfo("Connected! Sending Identify...".to_string())).await;

                let (mut write, mut read) = ws_stream.split();

                // Handshake: Identify as plugin
                let identify_req = WpWebSocketRequest {
                    source: "plugin".to_string(),
                    action: "identify".to_string(),
                };
                let identify_json = serde_json::to_string(&identify_req).unwrap();

                if let Err(e) = write.send(Message::Text(identify_json.into())).await {
                    let _ = tx
                        .send(Action::WebSocketStatus(
                            true,
                            Some(format!("Handshake failed: {}", e)),
                        ))
                        .await;
                }

                loop {
                    tokio::select! {
                        // Handle incoming WebSocket messages
                        msg = read.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    // Try to parse as JSON Value first for debugging
                                    if let Ok(_val) = serde_json::from_str::<serde_json::Value>(&text) {
                                         // let _ = tx.send(Action::DebugInfo(format!("RX: {}", val))).await;
                                    }

                                    match serde_json::from_str::<WpWebSocketMsg>(&text) {
                                        Ok(msg) => {
                                            if msg.action == "update-status" {
                                                if let Some(data) = msg.data {
                                                    // Parse Realtime KPS
                                                    let kps = if let Some(rt) = data.realtime {
                                                        parse_localized_float(&rt.keys)
                                                    } else {
                                                        0.0
                                                    };

                                                    // Parse Unpulsed Stats
                                                    let (keys, clicks, scrolls) = if let Some(up) = data.unpulsed {
                                                        (up.keys, up.clicks, up.scrolls)
                                                    } else {
                                                        (0, 0, 0)
                                                    };

                                                    // We can update last_time/last_keys if we want to verify KPS,
                                                    // but let's trust the API for now or use unpulsed for accumulated work.

                                                    // Parse Heatmap
                                                    let heatmap = data.heatmap.unwrap_or_default();

                                                    // Update TUI
                                                    let _ = tx.send(Action::RealtimeUpdate(RealtimeData {
                                                        unpulsed_keys: keys,
                                                        unpulsed_clicks: clicks,
                                                        unpulsed_scrolls: scrolls,
                                                        keys_per_second: kps,
                                                        heatmap,
                                                    })).await;
                                                }
                                            } else {
                                                let _ = tx.send(Action::DebugInfo(format!("Unknown Action: {}", msg.action))).await;
                                            }
                                        }
                                        Err(e) => {
                                            // It might be a simple response message like { "msg": "Pulse executed." }
                                            // or { "source": "plugin", "action": "identify" } echo?
                                            // Let's log it but not fail hard.
                                            let _ = tx.send(Action::DebugInfo(format!("JSON Parse Error: {} | Raw: {}", e, text))).await;
                                        }
                                    }
                                }
                                Some(Ok(Message::Close(_))) => break,
                                Some(Err(_)) => break,
                                None => break,
                                _ => {}
                            }
                        }
                        // Handle outgoing commands from TUI
                        cmd = rx_cmd.recv() => {
                            if let Some(command) = cmd {
                                let action_str = match command {
                                    MonitorCommand::Pulse => "pulse",
                                    MonitorCommand::OpenWindow => "open-window",
                                };

                                let req = WpWebSocketRequest {
                                    source: "plugin".to_string(),
                                    action: action_str.to_string(),
                                };
                                let req_json = serde_json::to_string(&req).unwrap();

                                // let _ = tx.send(Action::DebugInfo(format!("Sending command: {}", action_str))).await;
                                if let Err(e) = write.send(Message::Text(req_json.into())).await {
                                     let _ = tx.send(Action::DebugInfo(format!("Send failed: {}", e))).await;
                                }
                            } else {
                                // Channel closed
                                break;
                            }
                        }
                    }
                }
                let _ = tx
                    .send(Action::WebSocketStatus(
                        false,
                        Some("Connection closed".to_string()),
                    ))
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(Action::WebSocketStatus(false, Some(e.to_string())))
                    .await;
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

// TUI Rendering
fn render_tui(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Status Header
            Constraint::Length(3), // Power Gauge
            Constraint::Min(10),   // Graphs & Stats
        ])
        .split(area);

    // 1. Status Header
    let status_color = if app.kinetic_stats.is_connected {
        Color::Green
    } else {
        Color::Red
    };
    let status_text = if app.kinetic_stats.is_connected {
        if let Some(err) = &app.kinetic_stats.connection_error {
            format!("ERROR: {}", err)
        } else if let Some(last) = app.kinetic_stats.last_update {
            format!("{}", last.format("%H:%M:%S"))
        } else {
            "WAITING...".to_string()
        }
    } else {
        let error_msg = app
            .kinetic_stats
            .connection_error
            .as_deref()
            .unwrap_or("Retrying...");
        let clean_error = if error_msg.contains("No connection could be made") {
            "Connection Refused (Check WhatPulse Settings)"
        } else {
            error_msg
        };
        format!("DISCONNECTED: {}", clean_error)
    };

    let header = Paragraph::new(Line::from(vec![
        Span::raw("Kinetic Dashboard | "),
        Span::styled(status_text, Style::default().fg(status_color)),
        Span::raw(format!(
            " | Profile: {} ({:.1}cN, {:.1}mm)",
            app.current_profile().name,
            app.current_profile().force_newtons * 101.97, // N to gf roughly
            app.current_profile().distance_meters * 1000.0
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("System Status"),
    );
    f.render_widget(header, chunks[0]);

    // 2. Gauges Row (Power & Health)
    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // 2a. Power Gauge
    let power = app.kinetic_stats.current_power_watts;
    let ratio = (power / MAX_GAUGE_POWER_WATTS).min(1.0);

    let power_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Instantaneous Power Output"),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .ratio(ratio)
        .label(format!("{:.4} W", power));
    f.render_widget(power_gauge, gauge_chunks[0]);

    // 2b. Health/Intensity Gauge
    let hourly_joules = power * 3600.0;
    let health_ratio = (hourly_joules / HEALTH_LIMIT_JOULES_PER_HOUR).min(1.0);

    let health_color = if health_ratio > 0.9 {
        Color::Red
    } else if health_ratio > 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };

    let health_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Hourly Intensity (Fatigue Risk)"),
        )
        .gauge_style(Style::default().fg(health_color).bg(Color::Black))
        .ratio(health_ratio)
        .label(format!(
            "{:.1} / {:.1} J/h",
            hourly_joules, HEALTH_LIMIT_JOULES_PER_HOUR
        ));
    f.render_widget(health_gauge, gauge_chunks[1]);

    // 3. Stats & Graph
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Stats Column
    let (velocity_str, accel_str, unit) = match app.unit_system {
        UnitSystem::Metric => (
            format!("{:.4}", app.kinetic_stats.peak_velocity_mps),
            format!("{:.4}", app.kinetic_stats.burst_acceleration),
            "m/s",
        ),
        UnitSystem::Centimeters => (
            format!("{:.2}", app.kinetic_stats.peak_velocity_mps * 100.0),
            format!("{:.2}", app.kinetic_stats.burst_acceleration * 100.0),
            "cm/s",
        ),
    };

    let stats_text = vec![
        Line::from(vec![
            Span::styled(
                "Finger Velocity (Peak): ",
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(format!("{} {}", velocity_str, unit)),
        ]),
        Line::from(vec![
            Span::styled("Burst Accel: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} {}²", accel_str, unit)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Health Monitor",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Magenta),
        )]),
        Line::from(vec![
            Span::raw("Hourly Rate: "),
            Span::styled(
                format!("{:.2} J/h", app.kinetic_stats.current_power_watts * 3600.0),
                if app.kinetic_stats.current_power_watts * 3600.0 > HEALTH_LIMIT_JOULES_PER_HOUR {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::raw(format!(" (Limit: {:.1} J/h)", HEALTH_LIMIT_JOULES_PER_HOUR)),
        ]),
        Line::from(vec![Span::raw(format!(
            "Current Session: {:.4} J",
            app.kinetic_stats.accumulated_work_joules
        ))]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Shortcuts: ", Style::default().fg(Color::DarkGray)),
            Span::raw("'u' to toggle units"),
        ]),
    ];

    let stats = Paragraph::new(stats_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Kinetic Telemetry"),
    );
    f.render_widget(stats, bottom_chunks[0]);

    // Sparkline
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Power History (Watts)"),
        )
        .data(&app.kinetic_stats.history_power)
        .style(Style::default().fg(Color::LightBlue));
    f.render_widget(sparkline, bottom_chunks[1]);
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('p') => {
            app.profile_index = (app.profile_index + 1) % app.profiles.len();
            true
        }
        KeyCode::Char('u') => {
            app.unit_system = match app.unit_system {
                UnitSystem::Metric => UnitSystem::Centimeters,
                UnitSystem::Centimeters => UnitSystem::Metric,
            };
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_localized_float() {
        assert_eq!(parse_localized_float("2.17"), 2.17);
        assert_eq!(parse_localized_float("2,17"), 2.17);
        assert_eq!(parse_localized_float("0"), 0.0);
        assert_eq!(parse_localized_float("invalid"), 0.0);
    }

    #[test]
    fn test_deserialize_update_status() {
        let json = r#"{
            "action": "update-status",
            "data": {
                "account-totals": null,
                "realtime": {
                    "keys": "1,23",
                    "clicks": "0.45"
                },
                "unpulsed": {
                    "keys": 100,
                    "clicks": 50
                }
            }
        }"#;

        let msg: WpWebSocketMsg = serde_json::from_str(json).unwrap();
        assert_eq!(msg.action, "update-status");

        let data = msg.data.unwrap();
        let realtime = data.realtime.unwrap();
        assert_eq!(realtime.keys, "1,23");
        assert_eq!(realtime._clicks, "0.45");

        let unpulsed = data.unpulsed.unwrap();
        assert_eq!(unpulsed.keys, 100);
        assert_eq!(unpulsed.clicks, 50);
    }

    #[test]
    fn test_serialize_request() {
        let req = WpWebSocketRequest {
            source: "plugin".to_string(),
            action: "pulse".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"source":"plugin","action":"pulse"}"#);
    }
}
