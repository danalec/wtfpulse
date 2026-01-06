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
            Action::Quit => {
                return true;
            }
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
            Action::Key(key) => {
                // If there is an error popup, any key dismisses it
                if self.error.is_some() {
                    self.error = None;
                    return false;
                }

                // Update Session Heatmap from TUI inputs
                let key_str = match key.code {
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
                };
                if let Some(k) = key_str {
                    *self.keyboard.session_heatmap.entry(k).or_insert(0) += 1;
                }

                let pages = get_pages();

                use std::collections::HashMap;
                let categories = [
                    "Overview", "Input", "Network", "Uptime", "Settings", "Account", "Toys",
                ];
                let mut category_map: HashMap<&str, Vec<usize>> = HashMap::new();
                for (i, page) in pages.iter().enumerate() {
                    category_map.entry(page.category).or_default().push(i);
                }

                // --- Navigation Logic ---
                if self.nav.menu_open {
                    // Identify current category
                    let current_cat = pages[self.nav.current_tab].category;
                    let indices = category_map.get(current_cat).unwrap();

                    match key.code {
                        KeyCode::Esc => {
                            self.nav.menu_open = false;
                            return false;
                        }
                        KeyCode::Enter => {
                            self.nav.menu_open = false;
                            return false;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            // Find current index in the sub-list
                            if let Some(pos) =
                                indices.iter().position(|&x| x == self.nav.current_tab)
                            {
                                let new_pos = if pos == 0 { indices.len() - 1 } else { pos - 1 };
                                self.nav.current_tab = indices[new_pos];
                            }
                            return false;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(pos) =
                                indices.iter().position(|&x| x == self.nav.current_tab)
                            {
                                let new_pos = if pos == indices.len() - 1 { 0 } else { pos + 1 };
                                self.nav.current_tab = indices[new_pos];
                            }
                            return false;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            // Switch to prev category, first item
                            if let Some(pos) = categories.iter().position(|&c| c == current_cat) {
                                let new_cat_idx = if pos == 0 {
                                    categories.len() - 1
                                } else {
                                    pos - 1
                                };
                                let new_cat = categories[new_cat_idx];
                                if let Some(new_indices) = category_map.get(new_cat) {
                                    if let Some(&first) = new_indices.first() {
                                        self.nav.current_tab = first;
                                    }
                                    // Auto-close menu if single item
                                    if new_indices.len() <= 1 {
                                        self.nav.menu_open = false;
                                    }
                                }
                            }
                            return false;
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            // Switch to next category, first item
                            if let Some(pos) = categories.iter().position(|&c| c == current_cat) {
                                let new_cat_idx = if pos == categories.len() - 1 {
                                    0
                                } else {
                                    pos + 1
                                };
                                let new_cat = categories[new_cat_idx];
                                if let Some(new_indices) = category_map.get(new_cat) {
                                    if let Some(&first) = new_indices.first() {
                                        self.nav.current_tab = first;
                                    }
                                    // Auto-close menu if single item
                                    if new_indices.len() <= 1 {
                                        self.nav.menu_open = false;
                                    }
                                }
                            }
                            return false;
                        }
                        KeyCode::Char(c) => {
                            // Generic shortcut: Check if 'c' matches first char of any page in current category
                            if let Some(indices) = category_map.get(current_cat) {
                                for &idx in indices {
                                    if let Some(page) = pages.get(idx)
                                        && page
                                            .title
                                            .to_lowercase()
                                            .starts_with(&c.to_string().to_lowercase())
                                    {
                                        self.nav.current_tab = idx;
                                        self.nav.menu_open = false;
                                        return false;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // If menu is NOT open, check if we should open it or navigate categories
                // But first allow specific page shortcuts if they don't conflict?
                // Actually, standard TUI navigation (h/j/k/l) might conflict with inner page logic.
                // We typically use 'Tab' to switch focus or 'Ctrl+...'
                // Let's stick to:
                // - Tab: Next Category
                // - Shift+Tab: Prev Category
                // - Enter/Space (while on tab bar? no concept of focus yet): Open Menu?
                //
                // The previous logic was: Tab/Right -> Next Page.
                // New Logic:
                // - Left/Right: Switch Category (first item)
                // - Up/Down: Nothing? Or Open Menu?
                //
                // IMPORTANT: Many pages use h/j/k/l for THEIR own navigation (tables, etc).
                // So we shouldn't steal those unless we are in a "Navigation Mode" (nav_menu_open).
                //
                // So, how to enter Navigation Mode?
                // Maybe 'Tab' opens the menu for current category?
                // Or 'Ctrl+N'?
                // Let's try: 'Ctrl+N' (Navigate).
                // Or, if the user hits 'Tab', we cycle categories.
                // If the user hits 'Enter' on a category... we don't have focus on the tab bar.

                // Let's define:
                // Global Shortcuts:
                // - Tab: Open Nav Menu (if closed) OR Next Category (if open?) -> Let's make Tab toggle Nav Menu?
                // - Left/Right (Arrow): Switch Category (Immediate) - *Might conflict with page widgets?*
                // - Most pages handle Left/Right? No, usually h/l for period, or j/k for table.
                //
                // Let's check existing code:
                // "if !handled { ... KeyCode::Tab | KeyCode::Right => next tab ... }"
                // So tab navigation only happened if the page didn't consume the key.

                // Proposed:
                // Keep the "if !handled" pattern.
                // If page doesn't handle key, then checks for global nav.

                // Let the current page handle the key first
                let mut handled = false;
                if !self.nav.menu_open
                    && let Some(page) = pages.get(self.nav.current_tab)
                {
                    handled = (page.handle_key)(self, key);
                }

                // Handle Scroll Tower Tab Shortcuts (Index 4) - Specific override
                // This looks brittle index-based. Ideally Scroll Tower should handle this in its handle_key.
                // But it modifies App state fields that are specific to it.
                // We'll leave it for now but wrap in !nav_menu_open check implicitly by 'handled' or after.

                if !self.nav.menu_open && self.nav.current_tab == 4 {
                    // Scroll Tower
                    match key.code {
                        KeyCode::Char('p') => {
                            self.keyboard.profile_index =
                                (self.keyboard.profile_index + 1) % self.keyboard.profiles.len();
                            handled = true;
                        }
                        KeyCode::Char('w') => {
                            self.trigger_open_window().await;
                            handled = true;
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
                            handled = true;
                        }
                        _ => {}
                    }
                }

                if !handled {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            if self.nav.menu_open {
                                self.nav.menu_open = false;
                            } else {
                                self.nav.show_quit_confirm = !self.nav.show_quit_confirm;
                            }
                            return false;
                        }
                        KeyCode::Enter | KeyCode::Char('y') => {
                            if self.nav.show_quit_confirm {
                                return true; // Quit
                            } else if key.code == KeyCode::Enter {
                                // If not quitting, Enter might open the nav menu (if expandable)
                                let current_cat = categories
                                    .iter()
                                    .find(|&&cat| {
                                        if let Some(indices) = category_map.get(cat) {
                                            indices.contains(&self.nav.current_tab)
                                        } else {
                                            false
                                        }
                                    })
                                    .copied()
                                    .unwrap_or(categories[0]); // Fallback safe

                                if let Some(indices) = category_map.get(current_cat)
                                    && indices.len() > 1
                                {
                                    self.nav.menu_open = true;
                                }
                            }
                        }
                        KeyCode::Char('n') => {
                            if self.nav.show_quit_confirm {
                                self.nav.show_quit_confirm = false;
                                return false;
                            }
                        }
                        KeyCode::Char('r') => {
                            self.user_loading = true;
                            self.pulses_loading = true;
                            spawn_fetch(self.client.clone(), self.tx.clone());
                        }
                        KeyCode::Tab => {
                            // Toggle Nav Menu
                            self.nav.menu_open = !self.nav.menu_open;
                        }
                        // Allow Arrow Keys to switch categories if not handled by page
                        KeyCode::Right => {
                            // Logic to switch to next category's first item
                            use std::collections::HashMap;
                            let categories = [
                                "Overview", "Input", "Network", "Uptime", "Settings", "Account",
                                "Toys",
                            ];
                            let mut category_map: HashMap<&str, Vec<usize>> = HashMap::new();
                            for (i, page) in pages.iter().enumerate() {
                                category_map.entry(page.category).or_default().push(i);
                            }
                            let current_cat = pages[self.nav.current_tab].category;
                            if let Some(pos) = categories.iter().position(|&c| c == current_cat) {
                                let new_cat_idx = if pos == categories.len() - 1 {
                                    0
                                } else {
                                    pos + 1
                                };
                                let new_cat = categories[new_cat_idx];
                                if let Some(new_indices) = category_map.get(new_cat)
                                    && let Some(&first) = new_indices.first()
                                {
                                    self.nav.current_tab = first;
                                }
                            }
                        }
                        KeyCode::Left => {
                            // Logic to switch to prev category's first item
                            use std::collections::HashMap;
                            let categories = [
                                "Overview", "Input", "Network", "Uptime", "Settings", "Account",
                                "Toys",
                            ];
                            let mut category_map: HashMap<&str, Vec<usize>> = HashMap::new();
                            for (i, page) in pages.iter().enumerate() {
                                category_map.entry(page.category).or_default().push(i);
                            }
                            let current_cat = pages[self.nav.current_tab].category;
                            if let Some(pos) = categories.iter().position(|&c| c == current_cat) {
                                let new_cat_idx = if pos == 0 {
                                    categories.len() - 1
                                } else {
                                    pos - 1
                                };
                                let new_cat = categories[new_cat_idx];
                                if let Some(new_indices) = category_map.get(new_cat)
                                    && let Some(&first) = new_indices.first()
                                {
                                    self.nav.current_tab = first;
                                }
                            }
                        }
                        KeyCode::Down => {
                            // Open Nav Menu ONLY if category has > 1 item
                            let current_cat = pages[self.nav.current_tab].category;
                            let count = pages.iter().filter(|p| p.category == current_cat).count();
                            if count > 1 {
                                self.nav.menu_open = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Action::Mouse(mouse) => {
                let pages = get_pages();
                if let Some(page) = pages.get(self.nav.current_tab) {
                    let _ = (page.handle_mouse)(self, mouse);
                }
            }
            Action::UserLoaded(res) => {
                self.user_loading = false;
                match *res {
                    Ok(user) => {
                        self.user_stats = Some(user);
                        self.error = None;
                        self.recalculate_energy();
                        self.recalculate_unpulsed();
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
            Action::MouseHeatmapError(e) => {
                self.error = Some(e);
            }
            Action::MouseStatsLoaded(stats) => {
                self.mouse.stats = *stats;
                self.recalculate_unpulsed();
            }
            Action::AppStatsLoaded(res) => match res {
                Ok(stats) => {
                    self.apps.stats = stats;
                }
                Err(e) => {
                    self.error = Some(format!("Failed to load app stats: {}", e));
                }
            },
            Action::NetworkStatsLoaded(res) => match res {
                Ok(stats) => {
                    self.network.stats = stats;
                }
                Err(e) => {
                    self.error = Some(format!("Failed to load network stats: {}", e));
                }
            },
            Action::WebSocketStatus(connected, error) => {
                self.kinetic_stats.is_connected = connected;
                self.kinetic_stats.connection_error = error;
            }
            Action::RealtimeUpdate(data) => {
                let profile = self.keyboard.profiles[self.keyboard.profile_index].clone();
                let _ = self.kinetic_stats.update(&data, &profile);

                // Update Session Heatmap
                if !data.heatmap.is_empty() {
                    self.keyboard.session_heatmap = data.heatmap.clone();
                }

                // Update Scroll Meters with absolute total (User Baseline + Unpulsed)
                // 1 tick = 0.016 meters (1.6 cm)
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
            Action::DebugInfo(msg) => {
                self.kinetic_stats.debug_info = Some(msg);
            }
            Action::PopupSelect => {
                if let Some(selected_index) = self.keyboard.layout_list_state.borrow().selected() {
                    let layouts = KeyboardLayout::all();
                    // Need to filter again to find the correct item if searching
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
            Action::TogglePopup => {
                self.keyboard.show_layout_popup = !self.keyboard.show_layout_popup;
                // Reset search when opening
                if self.keyboard.show_layout_popup {
                    self.keyboard.layout_search_query.clear();
                    self.keyboard.layout_list_state.borrow_mut().select(Some(0));
                }
            }
            Action::SelectLayout => {
                // Already handled in PopupSelect, but kept for compatibility if needed
            }
            Action::NextLayoutItem => {
                let mut state = self.keyboard.layout_list_state.borrow_mut();
                let selected = state.selected().unwrap_or(0);
                // We don't know the filtered count here easily without recalculating
                // For simplicity, just increment (the UI rendering handles bounds usually, but logic is better)
                state.select(Some(selected + 1));
            }
            Action::PrevLayoutItem => {
                let mut state = self.keyboard.layout_list_state.borrow_mut();
                let selected = state.selected().unwrap_or(0);
                if selected > 0 {
                    state.select(Some(selected - 1));
                }
            }
            Action::PopupSearch(c) => {
                self.keyboard.layout_search_query.push_str(&c);
                self.keyboard.layout_list_state.borrow_mut().select(Some(0));
            }
        }
        false
    }

    pub fn get_uptime(&self) -> String {
        let _uptime_seconds = self.kinetic_stats.unpulsed_keys; // Placeholder
        // Real implementation would track start time
        "N/A".to_string()
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
