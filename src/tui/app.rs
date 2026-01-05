use crate::client::{ComputerResponse, PulseResponse, UserResponse, WhatpulseClient};
use crate::commands::calorimetry::{EnergyStats, SwitchProfile, calculate_energy};
use crate::commands::get_pages;
use crate::commands::heatmap::layouts::KeyboardLayout;
use crate::commands::heatmap::layouts::get_api_key_from_char;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

use log::info;
use std::collections::HashMap;

use std::cell::RefCell;

#[derive(Debug, Clone, Default)]
pub struct KineticStats {
    pub current_power_watts: f64,
    pub peak_velocity_mps: f64,
    pub accumulated_work_joules: f64,
    pub burst_acceleration: f64,
    pub history_power: Vec<u64>, // For Sparkline
    pub is_connected: bool,
    pub connection_error: Option<String>,
    pub last_keys: i64,
    pub last_scrolls: i64,
    pub last_velocity_mps: f64,
    pub last_update: Option<chrono::DateTime<chrono::Local>>,
    pub debug_info: Option<String>,
    // Raw stats for Dashboard
    pub unpulsed_keys: i64,
    pub unpulsed_clicks: i64,
    pub unpulsed_scrolls: i64,
    pub keys_per_second: f64,
}

impl KineticStats {
    pub fn update(&mut self, data: &RealtimeData, profile: &SwitchProfile) -> u32 {
        // Store raw data
        self.unpulsed_keys = data.unpulsed_keys;
        self.unpulsed_clicks = data.unpulsed_clicks;
        self.unpulsed_scrolls = data.unpulsed_scrolls;
        self.keys_per_second = data.keys_per_second;

        let now = chrono::Local::now();
        let dt = if let Some(last) = self.last_update {
            (now - last).num_milliseconds() as f64 / 1000.0
        } else {
            0.0
        };
        self.last_update = Some(now);

        // Calculate Delta Keys
        let delta = if self.last_keys == 0 {
            0 // First update, don't jump
        } else {
            data.unpulsed_keys - self.last_keys
        };

        // Handle reset (if unpulsed keys drops to 0)
        let delta = if delta < 0 { 0 } else { delta };

        // Update last keys
        self.last_keys = data.unpulsed_keys;

        // Calculate Delta Scrolls
        let delta_scrolls = if self.last_scrolls == 0 {
            0 // First update
        } else {
            data.unpulsed_scrolls - self.last_scrolls
        };
        let delta_scrolls = if delta_scrolls < 0 { 0 } else { delta_scrolls };
        self.last_scrolls = data.unpulsed_scrolls;

        // Force (N) * Distance (m) * Keys/s = Power (W)
        let power = profile.force_newtons * profile.distance_meters * data.keys_per_second;

        // Velocity (m/s) = Keys/s * Distance (m)
        let velocity = data.keys_per_second * profile.distance_meters;

        // Acceleration (m/s^2) = dV / dt
        if dt > 0.0 {
            let acceleration = (velocity - self.last_velocity_mps).abs() / dt;
            // "Burst" implies peak acceleration
            self.burst_acceleration = self.burst_acceleration.max(acceleration);
        }
        self.last_velocity_mps = velocity;

        // Accumulate Work: Work = Force * Distance * DeltaKeys
        let work_joules = profile.force_newtons * profile.distance_meters * (delta as f64);
        self.accumulated_work_joules += work_joules;

        // Update stats
        self.current_power_watts = power;
        self.peak_velocity_mps = self.peak_velocity_mps.max(velocity);

        // History for sparkline (scale up for visibility)
        self.history_power.push((power * 1000.0) as u64); // mW for better resolution
        if self.history_power.len() > 100 {
            self.history_power.remove(0);
        }

        delta_scrolls as u32
    }
}

#[derive(Debug, Clone)]
pub struct RealtimeData {
    pub unpulsed_keys: i64,
    pub unpulsed_clicks: i64,
    pub unpulsed_scrolls: i64,
    pub keys_per_second: f64,
    pub heatmap: HashMap<String, u64>,
}

pub enum Action {
    Tick,
    Quit,
    Refresh,
    Key(KeyEvent),
    UserLoaded(Box<Result<UserResponse>>),
    PulsesLoaded(Result<Vec<PulseResponse>>),
    ComputersLoaded(Result<Vec<ComputerResponse>>),
    HeatmapLoaded(HashMap<String, u64>, String),
    HeatmapError(String),
    WebSocketStatus(bool, Option<String>),
    RealtimeUpdate(RealtimeData),
    DebugInfo(String),
    TogglePopup,
    SelectLayout,
    NextLayoutItem,
    PrevLayoutItem,
    PopupSearch(String),
    PopupSelect,
}

use chrono::{Local, NaiveDate};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum UnitSystem {
    #[default]
    Metric, // Meters (m/s)
    Centimeters, // Centimeters (cm/s)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TimePeriod {
    Today,
    Yesterday,
    Week,
    Month,
    Year,
    #[default]
    All,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatePickerState {
    pub open: bool,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub current_selection: NaiveDate,
    pub selection_step: SelectionStep,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionStep {
    Start,
    End,
}

impl Default for DatePickerState {
    fn default() -> Self {
        Self {
            open: false,
            start_date: None,
            end_date: None,
            current_selection: Local::now().date_naive(),
            selection_step: SelectionStep::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ScrollMode {
    #[default]
    Lifetime,
    Session,
}

#[derive(Debug, Clone)]
pub enum MonitorCommand {
    Pulse,
    OpenWindow,
}

pub struct App {
    pub user_stats: Option<UserResponse>,
    pub recent_pulses: Vec<PulseResponse>,
    pub computers: Vec<ComputerResponse>,
    pub energy_stats: Option<EnergyStats>,
    pub user_loading: bool,
    pub pulses_loading: bool,
    pub computers_loading: bool,
    pub error: Option<String>,
    pub pulses_error: Option<String>,
    pub current_tab: usize,
    pub client: WhatpulseClient,
    pub tx: mpsc::Sender<Action>,
    pub monitor_tx: Option<mpsc::Sender<MonitorCommand>>,
    pub profiles: Vec<SwitchProfile>,
    pub profile_index: usize,
    pub dashboard_period: TimePeriod,
    pub date_picker: DatePickerState,
    pub kinetic_stats: KineticStats,
    pub unit_system: UnitSystem,
    pub heatmap_data: HashMap<String, u64>,
    pub keyboard_layout: KeyboardLayout,
    // Heatmap Layout Popup State
    pub show_layout_popup: bool,
    pub layout_search_query: String,
    pub layout_list_state: RefCell<ratatui::widgets::ListState>,
    pub data_source: String,
    pub heatmap_error: Option<String>,
    pub should_quit: bool,
    pub scroll_meters: f64,
    pub scroll_mode: ScrollMode,
    pub session_start_scrolls: Option<u64>,
    pub current_total_scrolls: u64,
    pub session_heatmap: HashMap<String, u64>,
}

impl App {
    pub fn new(client: WhatpulseClient, tx: mpsc::Sender<Action>) -> Self {
        Self {
            user_stats: None,
            recent_pulses: Vec::new(),
            computers: Vec::new(),
            energy_stats: None,
            user_loading: true,
            pulses_loading: true,
            computers_loading: true,
            error: None,
            pulses_error: None,
            current_tab: 0,
            client,
            tx,
            monitor_tx: None,
            profiles: vec![
                SwitchProfile::cherry_mx_red(),
                SwitchProfile::cherry_mx_blue(),
                SwitchProfile::cherry_mx_brown(),
                SwitchProfile::membrane(),
            ],
            profile_index: 0,
            dashboard_period: TimePeriod::default(),
            date_picker: DatePickerState::default(),
            kinetic_stats: KineticStats::default(),
            unit_system: UnitSystem::default(),
            heatmap_data: HashMap::new(),
            keyboard_layout: KeyboardLayout::Qwerty,
            show_layout_popup: false,
            layout_search_query: String::new(),
            layout_list_state: RefCell::new(ratatui::widgets::ListState::default()),
            data_source: String::from("API"),
            heatmap_error: None,
            should_quit: false,
            scroll_meters: 0.0,
            scroll_mode: ScrollMode::default(),
            session_start_scrolls: None,
            current_total_scrolls: 0,
            session_heatmap: HashMap::new(),
        }
    }

    pub fn set_monitor_tx(&mut self, tx: mpsc::Sender<MonitorCommand>) {
        self.monitor_tx = Some(tx);
    }

    pub async fn trigger_pulse(&self) {
        if let Some(tx) = &self.monitor_tx {
            let _ = tx.send(MonitorCommand::Pulse).await;
        }
    }

    pub async fn trigger_open_window(&self) {
        if let Some(tx) = &self.monitor_tx {
            let _ = tx.send(MonitorCommand::OpenWindow).await;
        }
    }

    pub fn current_profile(&self) -> &SwitchProfile {
        &self.profiles[self.profile_index]
    }

    pub fn recalculate_energy(&mut self) {
        if let Some(keys) = self.user_stats.as_ref().and_then(|u| u.totals.keys) {
            let profile = self.current_profile();
            self.energy_stats = calculate_energy(&keys.to_string(), Some(profile)).ok();
        }
    }

    pub async fn update(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => {
                return true;
            }
            Action::Tick => {
                // No time-based physics updates needed anymore
            }
            Action::Refresh => {
                self.user_loading = true;
                self.pulses_loading = true;
                spawn_fetch(self.client.clone(), self.tx.clone());
            }
            Action::Key(key) => {
                // Update Session Heatmap from TUI inputs
                let key_str = match key.code {
                    KeyCode::Char(c) => Some(get_api_key_from_char(c)),
                    KeyCode::Enter => Some("RETURN".to_string()),
                    KeyCode::Backspace => Some("BACKSPACE".to_string()),
                    KeyCode::Tab => Some("TAB".to_string()),
                    KeyCode::Esc => Some("ESCAPE".to_string()),
                    KeyCode::Delete => Some("DELETE".to_string()),
                    _ => None,
                };
                if let Some(k) = key_str {
                    *self.session_heatmap.entry(k).or_insert(0) += 1;
                }

                let pages = get_pages();

                // Let the current page handle the key first
                let mut handled = false;
                if let Some(page) = pages.get(self.current_tab) {
                    handled = (page.handle_key)(self, key);
                }

                // Handle Scroll Tower Tab Shortcuts (Index 4)
                if self.current_tab == 4 {
                    match key.code {
                        KeyCode::Char('p') => {
                            self.profile_index = (self.profile_index + 1) % self.profiles.len();
                            handled = true;
                        }
                        KeyCode::Char('w') => {
                            self.trigger_open_window().await;
                            handled = true;
                        }
                        KeyCode::Char('m') => {
                            // Toggle Mode
                            self.scroll_mode = match self.scroll_mode {
                                ScrollMode::Lifetime => ScrollMode::Session,
                                ScrollMode::Session => ScrollMode::Lifetime,
                            };

                            // Immediately recalc meters for UI responsiveness
                            // Need logic similar to RealtimeUpdate but utilizing current stored totals
                            let total = self.current_total_scrolls;
                            let display_scrolls = match self.scroll_mode {
                                ScrollMode::Lifetime => total,
                                ScrollMode::Session => total
                                    .saturating_sub(self.session_start_scrolls.unwrap_or(total)),
                            };
                            self.scroll_meters = display_scrolls as f64 * 0.016;

                            handled = true;
                        }
                        _ => {}
                    }
                }

                if !handled {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => return true,
                        KeyCode::Char('r') => {
                            self.user_loading = true;
                            self.pulses_loading = true;
                            spawn_fetch(self.client.clone(), self.tx.clone());
                        }
                        KeyCode::Tab | KeyCode::Right => {
                            self.current_tab = (self.current_tab + 1) % pages.len();
                        }
                        KeyCode::Left => {
                            if self.current_tab == 0 {
                                self.current_tab = pages.len() - 1;
                            } else {
                                self.current_tab -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Action::UserLoaded(res) => {
                self.user_loading = false;
                match *res {
                    Ok(user) => {
                        self.user_stats = Some(user);
                        self.error = None;
                        self.recalculate_energy();
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                    }
                }
            }
            Action::PulsesLoaded(res) => {
                self.pulses_loading = false;
                match res {
                    Ok(pulses) => {
                        self.recent_pulses = pulses;
                        self.pulses_error = None;
                    }
                    Err(e) => {
                        self.pulses_error = Some(e.to_string());
                    }
                }
            }
            Action::ComputersLoaded(res) => {
                self.computers_loading = false;
                match res {
                    Ok(comps) => {
                        self.computers = comps;
                    }
                    Err(_e) => {
                        // Maybe store computer error specifically?
                        // For now just log or ignore? Or use global error?
                        // Let's rely on individual tab error rendering if we add it.
                    }
                }
            }
            Action::HeatmapLoaded(map, source) => {
                info!("Heatmap loaded with {} keys from {}", map.len(), source);
                self.heatmap_data = map;
                self.error = None;
                self.heatmap_error = None;
                self.data_source = source;
            }
            Action::HeatmapError(e) => {
                self.error = Some(e.clone());
                self.heatmap_error = Some(e);
                self.data_source = "Error".to_string();
            }
            Action::WebSocketStatus(connected, error) => {
                self.kinetic_stats.is_connected = connected;
                self.kinetic_stats.connection_error = error;
            }
            Action::RealtimeUpdate(data) => {
                let profile = self.profiles[self.profile_index].clone();
                let _ = self.kinetic_stats.update(&data, &profile);

                // Update Session Heatmap
                if !data.heatmap.is_empty() {
                    self.session_heatmap = data.heatmap.clone();
                }

                // Update Scroll Meters with absolute total (User Baseline + Unpulsed)
                // 1 tick = 0.016 meters (1.6 cm)
                if let Some(user) = &self.user_stats {
                    let baseline = user.totals.scrolls;
                    let unpulsed = data.unpulsed_scrolls.max(0) as u64;
                    let total = baseline + unpulsed;

                    self.current_total_scrolls = total;

                    if self.session_start_scrolls.is_none() {
                        self.session_start_scrolls = Some(total);
                    }

                    let display_scrolls = match self.scroll_mode {
                        ScrollMode::Lifetime => total,
                        ScrollMode::Session => {
                            total.saturating_sub(self.session_start_scrolls.unwrap_or(total))
                        }
                    };

                    self.scroll_meters = display_scrolls as f64 * 0.016;
                }
            }
            Action::DebugInfo(msg) => {
                self.kinetic_stats.debug_info = Some(msg);
            }
            Action::PopupSelect => {
                if let Some(selected_index) = self.layout_list_state.borrow().selected() {
                    let layouts = KeyboardLayout::all();
                    // Need to filter again to find the correct item if searching
                    let filtered: Vec<_> = layouts
                        .into_iter()
                        .filter(|l| {
                            l.to_string()
                                .to_lowercase()
                                .contains(&self.layout_search_query.to_lowercase())
                        })
                        .collect();

                    if let Some(layout) = filtered.get(selected_index) {
                        self.keyboard_layout = *layout;
                        self.show_layout_popup = false;
                    }
                }
            }
            Action::TogglePopup => {
                self.show_layout_popup = !self.show_layout_popup;
                // Reset search when opening
                if self.show_layout_popup {
                    self.layout_search_query.clear();
                    self.layout_list_state.borrow_mut().select(Some(0));
                }
            }
            Action::SelectLayout => {
                // Already handled in PopupSelect, but kept for compatibility if needed
            }
            Action::NextLayoutItem => {
                let mut state = self.layout_list_state.borrow_mut();
                let selected = state.selected().unwrap_or(0);
                // We don't know the filtered count here easily without recalculating
                // For simplicity, just increment (the UI rendering handles bounds usually, but logic is better)
                state.select(Some(selected + 1));
            }
            Action::PrevLayoutItem => {
                let mut state = self.layout_list_state.borrow_mut();
                let selected = state.selected().unwrap_or(0);
                if selected > 0 {
                    state.select(Some(selected - 1));
                }
            }
            Action::PopupSearch(c) => {
                self.layout_search_query.push_str(&c);
                self.layout_list_state.borrow_mut().select(Some(0));
            }
        }
        false
    }
}

pub fn spawn_fetch(client: WhatpulseClient, tx: mpsc::Sender<Action>) {
    let tx_user = tx.clone();
    let client_user = client.clone();
    tokio::spawn(async move {
        let res = client_user.get_user().await;
        let _ = tx_user.send(Action::UserLoaded(Box::new(res))).await;
    });

    let tx_pulses = tx.clone();
    let client_pulses = client.clone();
    tokio::spawn(async move {
        let res = client_pulses.get_pulses().await;
        let _ = tx_pulses.send(Action::PulsesLoaded(res)).await;
    });

    let tx_computers = tx.clone();
    let client_computers = client.clone();
    tokio::spawn(async move {
        let res = client_computers.get_computers().await;
        let _ = tx_computers.send(Action::ComputersLoaded(res)).await;
    });

    // Initial Heatmap Fetch (All time)
    let tx_heatmap = tx.clone();
    let client_heatmap = client.clone();
    tokio::spawn(async move {
        match client_heatmap.get_heatmap("all").await {
            Ok((map, source)) => {
                let _ = tx_heatmap.send(Action::HeatmapLoaded(map, source)).await;
            }
            Err(e) => {
                let _ = tx_heatmap.send(Action::HeatmapError(e.to_string())).await;
            }
        }
    });
}

pub fn spawn_fetch_heatmap(client: WhatpulseClient, tx: mpsc::Sender<Action>, period: &str) {
    let period = period.to_string();
    tokio::spawn(async move {
        match client.get_heatmap(&period).await {
            Ok((map, source)) => {
                let _ = tx.send(Action::HeatmapLoaded(map, source)).await;
            }
            Err(e) => {
                let _ = tx.send(Action::HeatmapError(e.to_string())).await;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kinetic_stats_update() {
        let mut stats = KineticStats::default();
        let profile = SwitchProfile::cherry_mx_blue(); // 60g (~0.58N), 4mm (0.004m)

        // Initial state
        assert_eq!(stats.accumulated_work_joules, 0.0);
        assert_eq!(stats.last_keys, 0);

        // Update 1: 10 keys unpulsed, 5 KPS
        let data1 = RealtimeData {
            unpulsed_keys: 10,
            unpulsed_clicks: 0,
            unpulsed_scrolls: 0,
            keys_per_second: 5.0,
            heatmap: HashMap::new(),
        };
        stats.update(&data1, &profile);

        // Delta should be 0 because last_keys was 0 (init logic to avoid huge jump)
        assert_eq!(stats.last_keys, 10);
        assert_eq!(stats.accumulated_work_joules, 0.0);

        // Power = F * d * KPS = 0.588N * 0.004m * 5.0/s = ~0.01176 W
        let expected_power = profile.force_newtons * profile.distance_meters * 5.0;
        assert!((stats.current_power_watts - expected_power).abs() < 1e-6);

        // Update 2: 20 keys unpulsed (delta 10), 5 KPS
        let data2 = RealtimeData {
            unpulsed_keys: 20,
            unpulsed_clicks: 0,
            unpulsed_scrolls: 0,
            keys_per_second: 5.0,
            heatmap: HashMap::new(),
        };
        stats.update(&data2, &profile);

        // Work = F * d * Delta(10)
        let expected_work = profile.force_newtons * profile.distance_meters * 10.0;
        assert!((stats.accumulated_work_joules - expected_work).abs() < 1e-6);
    }
}
