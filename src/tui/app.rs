use crate::client::{ComputerResponse, PulseResponse, UserResponse, WhatpulseClient};
use crate::commands::calorimetry::{EnergyStats, SwitchProfile, calculate_energy};
use crate::commands::get_pages;
use crate::commands::keyboard::layouts::KeyboardLayout;
use crate::commands::keyboard::layouts::get_api_key_from_char;
use crate::db::{AppStats, MouseStats, NetworkStats};
pub use crate::tui::state::{
    AppSortMode, AppsState, ExtendedMouseStats, KeyboardState, MouseState, NavigationState,
    NetworkSortMode, NetworkState, ScrollMode, SortOrder, TimePeriod, UnitSystem,
};

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
    Mouse(crossterm::event::MouseEvent),
    UserLoaded(Box<Result<UserResponse>>),
    PulsesLoaded(Result<Vec<PulseResponse>>),
    ComputersLoaded(Result<Vec<ComputerResponse>>),
    KeyboardHeatmapLoaded(HashMap<String, u64>, String),
    KeyboardHeatmapError(String),
    MouseHeatmapLoaded(Vec<Vec<u64>>),
    MouseHeatmapError(String),
    MouseStatsLoaded(Box<ExtendedMouseStats>),
    AppStatsLoaded(Result<Vec<AppStats>>),
    NetworkStatsLoaded(Result<Vec<NetworkStats>>),
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
    pub client: WhatpulseClient,
    pub tx: mpsc::Sender<Action>,
    pub monitor_tx: Option<mpsc::Sender<MonitorCommand>>,

    // Sub-states
    pub nav: NavigationState,
    pub mouse: MouseState,
    pub keyboard: KeyboardState,
    pub apps: AppsState,
    pub network: NetworkState,

    pub dashboard_period: TimePeriod,
    pub date_picker: DatePickerState,
    pub kinetic_stats: KineticStats,
    pub unit_system: UnitSystem,
    pub data_source: String,

    pub should_quit: bool,
    pub pulses_table_state: RefCell<ratatui::widgets::TableState>,
    pub last_refresh: std::time::Instant,
    pub refresh_rate: std::time::Duration,
    pub config: crate::config::AppConfig,
    pub is_editing_api_key: bool,
    pub api_key_input: String,
    pub notification: Option<(String, std::time::Instant)>,
    pub uptime_period: TimePeriod,
    pub start_time: std::time::Instant,
    pub show_help: bool,
}

impl App {
    pub fn new(client: WhatpulseClient, tx: mpsc::Sender<Action>) -> Self {
        let config = crate::config::AppConfig::load().unwrap_or_default();
        let refresh_rate =
            std::time::Duration::from_secs(config.refresh_rate_seconds.unwrap_or(60));

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
            client,
            tx,
            monitor_tx: None,
            nav: NavigationState::default(),
            mouse: MouseState::default(),
            keyboard: KeyboardState {
                layout: KeyboardLayout::Qwerty,
                ..Default::default()
            },
            apps: AppsState::default(),
            network: NetworkState::default(),

            dashboard_period: TimePeriod::All,
            date_picker: DatePickerState::default(),
            kinetic_stats: KineticStats::default(),
            unit_system: UnitSystem::Metric,
            data_source: String::new(),

            should_quit: false,
            pulses_table_state: RefCell::new(ratatui::widgets::TableState::default()),
            last_refresh: std::time::Instant::now(),
            refresh_rate,
            config,
            is_editing_api_key: false,
            api_key_input: String::new(),
            notification: None,
            uptime_period: TimePeriod::All,
            start_time: std::time::Instant::now(),
            show_help: false,
        }
    }

    pub fn set_notification(&mut self, message: String) {
        self.notification = Some((message, std::time::Instant::now()));
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
        &self.keyboard.profiles[self.keyboard.profile_index]
    }

    pub fn recalculate_energy(&mut self) {
        if let Some(keys) = self.user_stats.as_ref().and_then(|u| u.totals.keys) {
            let profile = self.current_profile();
            self.energy_stats = calculate_energy(&keys.to_string(), Some(profile)).ok();
        }
    }

    pub fn recalculate_unpulsed(&mut self) {
        let (pulsed_clicks, pulsed_scrolls, pulsed_distance_miles) =
            if let Some(user) = &self.user_stats {
                // Try to find local computer first
                let hostname = std::env::var("COMPUTERNAME")
                    .or_else(|_| std::env::var("HOSTNAME"))
                    .unwrap_or_else(|_| "localhost".to_string());

                let local_comp = self
                    .computers
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(&hostname));

                if let Some(comp) = local_comp {
                    (
                        comp.totals.clicks,
                        comp.totals.scrolls.unwrap_or(0),
                        comp.totals.distance_miles.unwrap_or(0.0),
                    )
                } else {
                    // Fallback to user totals (Global)
                    (
                        user.totals.clicks.unwrap_or(0),
                        user.totals.scrolls,
                        user.totals.distance_miles.unwrap_or(0.0),
                    )
                }
            } else {
                (0, 0, 0.0)
            };

        // Unpulsed Clicks
        let total_clicks = self.mouse.stats.all_time.clicks;
        let unpulsed_clicks = total_clicks.saturating_sub(pulsed_clicks);

        // Unpulsed Scrolls
        let total_scrolls = self.mouse.stats.all_time.scrolls;
        let unpulsed_scrolls = total_scrolls.saturating_sub(pulsed_scrolls);

        // Unpulsed Distance
        let total_distance_meters = self.mouse.stats.all_time.distance_meters;
        let pulsed_distance_meters = pulsed_distance_miles * 1609.34;
        let unpulsed_distance_meters = (total_distance_meters - pulsed_distance_meters).max(0.0);

        self.mouse.stats.unpulsed = MouseStats {
            clicks: unpulsed_clicks,
            scrolls: unpulsed_scrolls,
            distance_meters: unpulsed_distance_meters,
            clicks_by_button: HashMap::new(),
        };
    }

    pub async fn update(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,
            Action::Tick => {
                if self.last_refresh.elapsed() >= self.refresh_rate {
                    self.last_refresh = std::time::Instant::now();
                    let _ = self.tx.send(Action::Refresh).await;
                }
            }
            Action::Refresh => {
                self.user_loading = true;
                self.pulses_loading = true;
                spawn_fetch(self.client.clone(), self.tx.clone());
            }
            Action::Key(key) => return self.handle_key_event(key).await,
            Action::Mouse(mouse) => self.handle_mouse_event(mouse),
            Action::UserLoaded(res) => {
                self.user_loading = false;
                match *res {
                    Ok(user) => {
                        self.user_stats = Some(user);
                        self.error = None;
                        self.recalculate_energy();
                        self.recalculate_unpulsed();
                    }
                    Err(e) => self.error = Some(e.to_string()),
                }
            }
            Action::PulsesLoaded(res) => {
                self.pulses_loading = false;
                match res {
                    Ok(pulses) => {
                        self.recent_pulses = pulses;
                        self.pulses_error = None;
                    }
                    Err(e) => self.pulses_error = Some(e.to_string()),
                }
            }
            Action::ComputersLoaded(res) => {
                self.computers_loading = false;
                if let Ok(comps) = res {
                    self.computers = comps;
                }
            }
            Action::KeyboardHeatmapLoaded(map, source) => {
                info!("Heatmap loaded with {} keys from {}", map.len(), source);
                self.keyboard.heatmap_data = map;
                self.error = None;
                self.keyboard.heatmap_error = None;
                self.data_source = source;
            }
            Action::KeyboardHeatmapError(e) => {
                self.error = Some(e.clone());
                self.keyboard.heatmap_error = Some(e);
                self.data_source = "Error".to_string();
            }
            Action::MouseHeatmapLoaded(grid) => {
                info!("Screen Heatmap loaded with {} rows", grid.len());
                self.mouse.screen_heatmap = grid;
                self.error = None;
            }
            Action::MouseHeatmapError(e) => self.error = Some(e),
            Action::MouseStatsLoaded(stats) => {
                self.mouse.stats = *stats;
                self.recalculate_unpulsed();
            }
            Action::AppStatsLoaded(res) => match res {
                Ok(stats) => self.apps.stats = stats,
                Err(e) => self.error = Some(format!("Failed to load app stats: {}", e)),
            },
            Action::NetworkStatsLoaded(res) => match res {
                Ok(stats) => self.network.stats = stats,
                Err(e) => self.error = Some(format!("Failed to load network stats: {}", e)),
            },
            Action::WebSocketStatus(connected, error) => {
                self.kinetic_stats.is_connected = connected;
                self.kinetic_stats.connection_error = error;
            }
            Action::RealtimeUpdate(data) => self.handle_realtime_update(data),
            Action::DebugInfo(msg) => self.kinetic_stats.debug_info = Some(msg),
            Action::PopupSelect => self.handle_popup_select(),
            Action::TogglePopup => self.handle_toggle_popup(),
            Action::SelectLayout => {},
            Action::NextLayoutItem => self.handle_list_nav(1),
            Action::PrevLayoutItem => self.handle_list_nav(-1),
            Action::PopupSearch(c) => {
                self.keyboard.layout_search_query.push_str(&c);
                self.keyboard.layout_list_state.borrow_mut().select(Some(0));
            }
        }
        false
    }

    fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) {
        let pages = get_pages();
        if let Some(page) = pages.get(self.nav.current_tab) {
            let _ = (page.handle_mouse)(self, mouse);
        }
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.error.is_some() {
            self.error = None;
            return false;
        }

        if key.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return false;
        }

        if self.show_help {
             if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                 self.show_help = false;
             }
             return false;
        }

        if let Some(k) = self.key_to_api_string(key.code) {
             *self.keyboard.session_heatmap.entry(k).or_insert(0) += 1;
        }

        let pages = get_pages();
        let mut handled = false;
        if !self.nav.menu_open {
             if let Some(page) = pages.get(self.nav.current_tab) {
                 handled = (page.handle_key)(self, key);
             }
        }

        if !handled && !self.nav.menu_open && self.nav.current_tab == 4 {
             handled = self.handle_scroll_tower_shortcuts(key).await;
        }

        if !handled {
            return self.handle_navigation(key);
        }
        false
    }

    fn key_to_api_string(&self, code: KeyCode) -> Option<String> {
        match code {
            KeyCode::Char(c) => Some(get_api_key_from_char(c)),
            KeyCode::Enter => Some("RETURN".to_string()),
            KeyCode::Backspace => Some("BACKSPACE".to_string()),
            KeyCode::Tab => Some("TAB".to_string()),
            KeyCode::Esc => Some("ESCAPE".to_string()),
            KeyCode::Delete => Some("DELETE".to_string()),
            KeyCode::Insert => Some("INSERT".to_string()),
            KeyCode::Home => Some("HOME".to_string()),
            KeyCode::End => Some("END".to_string()),
            KeyCode::PageUp => Some("PAGEUP".to_string()),
            KeyCode::PageDown => Some("PAGEDOWN".to_string()),
            KeyCode::Left => Some("LEFT".to_string()),
            KeyCode::Right => Some("RIGHT".to_string()),
            KeyCode::Up => Some("UP".to_string()),
            KeyCode::Down => Some("DOWN".to_string()),
            _ => None,
        }
    }

    async fn handle_scroll_tower_shortcuts(&mut self, key: KeyEvent) -> bool {
         match key.code {
            KeyCode::Char('p') => {
                self.keyboard.profile_index =
                    (self.keyboard.profile_index + 1) % self.keyboard.profiles.len();
                true
            }
            KeyCode::Char('w') => {
                self.trigger_open_window().await;
                true
            }
            KeyCode::Char('m') => {
                self.mouse.scroll_mode = match self.mouse.scroll_mode {
                    ScrollMode::Lifetime => ScrollMode::Session,
                    ScrollMode::Session => ScrollMode::Lifetime,
                };
                let total = self.mouse.current_total_scrolls;
                let display_scrolls = match self.mouse.scroll_mode {
                    ScrollMode::Lifetime => total,
                    ScrollMode::Session => total.saturating_sub(
                        self.mouse.session_start_scrolls.unwrap_or(total),
                    ),
                };
                self.mouse.scroll_meters = display_scrolls as f64 * 0.016;
                true
            }
            _ => false
        }
    }

    fn handle_navigation(&mut self, key: KeyEvent) -> bool {
        let pages = get_pages();
        let categories = [
            "Overview", "Input", "Network", "Uptime", "Settings", "Account", "Toys",
        ];
        let mut category_map: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, page) in pages.iter().enumerate() {
            category_map.entry(page.category).or_default().push(i);
        }

        if self.nav.menu_open {
             let current_cat = pages[self.nav.current_tab].category;
             let indices = category_map.get(current_cat).unwrap();
             
             match key.code {
                 KeyCode::Esc | KeyCode::Enter => { self.nav.menu_open = false; }
                 KeyCode::Up | KeyCode::Char('k') => {
                      if let Some(pos) = indices.iter().position(|&x| x == self.nav.current_tab) {
                           let new_pos = if pos == 0 { indices.len() - 1 } else { pos - 1 };
                           self.nav.current_tab = indices[new_pos];
                      }
                 }
                 KeyCode::Down | KeyCode::Char('j') => {
                      if let Some(pos) = indices.iter().position(|&x| x == self.nav.current_tab) {
                           let new_pos = if pos == indices.len() - 1 { 0 } else { pos + 1 };
                           self.nav.current_tab = indices[new_pos];
                      }
                 }
                 KeyCode::Left | KeyCode::Char('h') => self.switch_category(&categories, &category_map, -1),
                 KeyCode::Right | KeyCode::Char('l') => self.switch_category(&categories, &category_map, 1),
                 KeyCode::Char(c) => {
                      if let Some(indices) = category_map.get(current_cat) {
                           for &idx in indices {
                                if let Some(page) = pages.get(idx) 
                                    && page.title.to_lowercase().starts_with(&c.to_string().to_lowercase()) 
                                {
                                     self.nav.current_tab = idx;
                                     self.nav.menu_open = false;
                                     break;
                                }
                           }
                      }
                 }
                 _ => {}
             }
             return false;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.nav.show_quit_confirm {
                    self.nav.show_quit_confirm = false;
                } else {
                    self.nav.show_quit_confirm = true;
                }
            }
            KeyCode::Enter | KeyCode::Char('y') => {
                if self.nav.show_quit_confirm {
                    return true;
                }
                if key.code == KeyCode::Enter {
                     let current_cat = pages[self.nav.current_tab].category;
                     if let Some(indices) = category_map.get(current_cat) {
                         if indices.len() > 1 {
                             self.nav.menu_open = true;
                         }
                     }
                }
            }
            KeyCode::Char('n') => {
                if self.nav.show_quit_confirm {
                    self.nav.show_quit_confirm = false;
                }
            }
            KeyCode::Char('r') => {
                self.user_loading = true;
                self.pulses_loading = true;
                spawn_fetch(self.client.clone(), self.tx.clone());
            }
            KeyCode::Tab => self.nav.menu_open = !self.nav.menu_open,
            KeyCode::Right => self.switch_category(&categories, &category_map, 1),
            KeyCode::Left => self.switch_category(&categories, &category_map, -1),
            KeyCode::Down => {
                 let current_cat = pages[self.nav.current_tab].category;
                 if let Some(indices) = category_map.get(current_cat) {
                     if indices.len() > 1 {
                         self.nav.menu_open = true;
                     }
                 }
            }
            _ => {}
        }
        false
    }

    fn switch_category(&mut self, categories: &[&str], map: &HashMap<&str, Vec<usize>>, dir: i32) {
         let pages = get_pages();
         let current_cat = pages[self.nav.current_tab].category;
         if let Some(pos) = categories.iter().position(|&c| c == current_cat) {
             let new_pos = if dir > 0 {
                 if pos == categories.len() - 1 { 0 } else { pos + 1 }
             } else {
                 if pos == 0 { categories.len() - 1 } else { pos - 1 }
             };
             let new_cat = categories[new_pos];
             if let Some(indices) = map.get(new_cat) {
                 if let Some(&first) = indices.first() {
                     self.nav.current_tab = first;
                 }
                 if indices.len() <= 1 && self.nav.menu_open {
                     self.nav.menu_open = false;
                 }
             }
         }
    }

    fn handle_realtime_update(&mut self, data: RealtimeData) {
        let profile = self.keyboard.profiles[self.keyboard.profile_index].clone();
        let _ = self.kinetic_stats.update(&data, &profile);

        if !data.heatmap.is_empty() {
            self.keyboard.session_heatmap = data.heatmap.clone();
        }

        if let Some(user) = &self.user_stats {
            let baseline = user.totals.scrolls;
            let unpulsed = data.unpulsed_scrolls.max(0) as u64;
            let total = baseline + unpulsed;

            self.mouse.current_total_scrolls = total;

            if self.mouse.session_start_scrolls.is_none() {
                self.mouse.session_start_scrolls = Some(total);
            }

            let display_scrolls = match self.mouse.scroll_mode {
                ScrollMode::Lifetime => total,
                ScrollMode::Session => {
                    total.saturating_sub(self.mouse.session_start_scrolls.unwrap_or(total))
                }
            };

            self.mouse.scroll_meters = display_scrolls as f64 * 0.016;
        }
    }

    fn handle_popup_select(&mut self) {
        if let Some(selected_index) = self.keyboard.layout_list_state.borrow().selected() {
            let layouts = KeyboardLayout::all();
            let filtered: Vec<_> = layouts
                .into_iter()
                .filter(|l| {
                    l.to_string()
                        .to_lowercase()
                        .contains(&self.keyboard.layout_search_query.to_lowercase())
                })
                .collect();

            if let Some(layout) = filtered.get(selected_index) {
                self.keyboard.layout = *layout;
                self.keyboard.show_layout_popup = false;
            }
        }
    }

    fn handle_toggle_popup(&mut self) {
        self.keyboard.show_layout_popup = !self.keyboard.show_layout_popup;
        if self.keyboard.show_layout_popup {
            self.keyboard.layout_search_query.clear();
            self.keyboard.layout_list_state.borrow_mut().select(Some(0));
        }
    }

    fn handle_list_nav(&mut self, dir: i32) {
        let mut state = self.keyboard.layout_list_state.borrow_mut();
        let selected = state.selected().unwrap_or(0);
        if dir > 0 {
             state.select(Some(selected + 1));
        } else if selected > 0 {
             state.select(Some(selected - 1));
        }
    }

    pub fn get_uptime(&self) -> String {
        let elapsed = self.start_time.elapsed();
        let days = elapsed.as_secs() / 86400;
        let hours = (elapsed.as_secs() % 86400) / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;
        if days > 0 {
            format!("{}d {:02}h {:02}m", days, hours, minutes)
        } else {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }
    }

    pub fn sort_app_stats(&mut self) {
        let mode = self.apps.sort_mode;
        let order = self.apps.sort_order;

        self.apps.stats.sort_by(|a, b| {
            let cmp = match mode {
                AppSortMode::Keys => a.keys.cmp(&b.keys),
                AppSortMode::Clicks => a.clicks.cmp(&b.clicks),
                AppSortMode::Scrolls => a.scrolls.cmp(&b.scrolls),
                AppSortMode::Download => a
                    .download_mb
                    .partial_cmp(&b.download_mb)
                    .unwrap_or(std::cmp::Ordering::Equal),
                AppSortMode::Upload => a
                    .upload_mb
                    .partial_cmp(&b.upload_mb)
                    .unwrap_or(std::cmp::Ordering::Equal),
                AppSortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            };

            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    pub fn sort_network_stats(&mut self) {
        let mode = self.network.sort_mode;
        let order = self.network.sort_order;

        self.network.stats.sort_by(|a, b| {
            let cmp = match mode {
                NetworkSortMode::Download => a
                    .download_mb
                    .partial_cmp(&b.download_mb)
                    .unwrap_or(std::cmp::Ordering::Equal),
                NetworkSortMode::Upload => a
                    .upload_mb
                    .partial_cmp(&b.upload_mb)
                    .unwrap_or(std::cmp::Ordering::Equal),
                NetworkSortMode::Total => (a.download_mb + a.upload_mb)
                    .partial_cmp(&(b.download_mb + b.upload_mb))
                    .unwrap_or(std::cmp::Ordering::Equal),
                NetworkSortMode::Interface => {
                    a.interface.to_lowercase().cmp(&b.interface.to_lowercase())
                }
            };

            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
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

    // Initial Heatmap Fetch
    spawn_fetch_keyboard_heatmap(client.clone(), tx.clone(), "all");
    spawn_fetch_mouse_heatmap(client.clone(), tx.clone(), "today");

    spawn_fetch_mouse_stats(tx.clone());
    spawn_fetch_app_stats(tx.clone(), "all");
    spawn_fetch_network_stats(tx.clone(), "all");
}

pub fn spawn_fetch_mouse_stats(tx: mpsc::Sender<Action>) {
    let tx_mouse = tx.clone();
    tokio::spawn(async move {
        let stats = tokio::task::spawn_blocking(move || -> Result<ExtendedMouseStats> {
            let db = crate::db::Database::new()?;

            let today = db.get_mouse_stats("today")?;
            let yesterday = db.get_mouse_stats("yesterday")?;
            let all_time = db.get_mouse_stats("all")?;

            Ok(ExtendedMouseStats {
                today,
                yesterday,
                all_time,
                unpulsed: MouseStats::default(),
            })
        })
        .await;

        match stats {
            Ok(Ok(s)) => {
                let _ = tx_mouse.send(Action::MouseStatsLoaded(Box::new(s))).await;
            }
            Ok(Err(e)) => {
                log::error!("Failed to fetch mouse stats: {}", e);
            }
            Err(e) => {
                log::error!("Join error fetching mouse stats: {}", e);
            }
        }
    });
}

pub fn spawn_fetch_app_stats(tx: mpsc::Sender<Action>, period: &str) {
    let tx_app = tx.clone();
    let period = period.to_string();
    tokio::spawn(async move {
        let stats = tokio::task::spawn_blocking(move || -> Result<Vec<AppStats>> {
            let db = crate::db::Database::new()?;
            db.get_app_stats(&period)
        })
        .await;

        match stats {
            Ok(res) => {
                let _ = tx_app.send(Action::AppStatsLoaded(res)).await;
            }
            Err(e) => {
                let _ = tx_app.send(Action::AppStatsLoaded(Err(e.into()))).await;
            }
        }
    });
}

pub fn spawn_fetch_network_stats(tx: mpsc::Sender<Action>, period: &str) {
    let tx_net = tx.clone();
    let period = period.to_string();
    tokio::spawn(async move {
        let stats = tokio::task::spawn_blocking(move || -> Result<Vec<NetworkStats>> {
            let db = crate::db::Database::new()?;
            db.get_network_stats(&period)
        })
        .await;

        match stats {
            Ok(res) => {
                let _ = tx_net.send(Action::NetworkStatsLoaded(res)).await;
            }
            Err(e) => {
                let _ = tx_net.send(Action::NetworkStatsLoaded(Err(e.into()))).await;
            }
        }
    });
}

pub fn spawn_fetch_mouse_heatmap(_client: WhatpulseClient, tx: mpsc::Sender<Action>, period: &str) {
    let period = period.to_string();
    tokio::spawn(async move {
        // Use standard dimensions (320x200) or config if available?
        // For TUI, 320x200 is high res enough for scaling down to terminal cells.
        let grid_w = 320;
        let grid_h = 200;

        let res = tokio::task::spawn_blocking(move || -> Result<Vec<Vec<u64>>> {
            let db = crate::db::Database::new()?;
            db.get_mouse_heatmap_grid(&period, grid_w, grid_h)
        })
        .await;

        match res {
            Ok(Ok(grid)) => {
                let _ = tx.send(Action::MouseHeatmapLoaded(grid)).await;
            }
            Ok(Err(e)) => {
                let _ = tx.send(Action::MouseHeatmapError(e.to_string())).await;
            }
            Err(e) => {
                let _ = tx.send(Action::MouseHeatmapError(e.to_string())).await;
            }
        }
    });
}

pub fn spawn_fetch_keyboard_heatmap(
    _client: WhatpulseClient,
    tx: mpsc::Sender<Action>,
    period: &str,
) {
    let period = period.to_string();
    tokio::spawn(async move {
        let map = tokio::task::spawn_blocking(move || -> Result<HashMap<String, u64>> {
            let db = crate::db::Database::new()?;
            db.get_heatmap_stats(&period)
        })
        .await;

        match map {
            Ok(Ok(map)) => {
                let _ = tx
                    .send(Action::KeyboardHeatmapLoaded(map, "Local DB".to_string()))
                    .await;
            }
            Ok(Err(e)) => {
                let _ = tx.send(Action::KeyboardHeatmapError(e.to_string())).await;
            }
            Err(e) => {
                let _ = tx.send(Action::KeyboardHeatmapError(e.to_string())).await;
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

    #[test]
    fn test_recalculate_unpulsed() {
        let mut app = App::new(
            crate::client::WhatpulseClient::new_local().unwrap(),
            tokio::sync::mpsc::channel(1).0,
        );
        app.mouse.stats.all_time.clicks = 1000;
        app.mouse.stats.all_time.scrolls = 500;
        app.mouse.stats.all_time.distance_meters = 100.0;
        let user = crate::client::UserResponse {
            id: 1,
            username: "test".to_string(),
            date_joined: None,
            first_pulse_date: None,
            last_pulse_date: None,
            pulses: 0,
            team_id: None,
            team_is_manager: false,
            country_id: None,
            is_premium: false,
            referrals: 0,
            last_referral_date: None,
            avatar: None,
            totals: crate::client::UserTotals {
                keys: None,
                clicks: Some(800),
                download_mb: None,
                upload_mb: None,
                uptime_seconds: None,
                scrolls: 400,
                distance_miles: Some(0.05),
            },
            ranks: None,
            include_in_rankings: false,
            distance_system: "metric".to_string(),
            last_pulse: None,
        };
        app.user_stats = Some(user);

        app.recalculate_unpulsed();
        assert_eq!(app.mouse.stats.unpulsed.clicks, 200);
        assert_eq!(app.mouse.stats.unpulsed.scrolls, 100);
        let expected_dist = 100.0 - (0.05 * 1609.34);
        assert!((app.mouse.stats.unpulsed.distance_meters - expected_dist).abs() < 1e-3);
    }
}
