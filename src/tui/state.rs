use crate::commands::calorimetry::SwitchProfile;
use crate::commands::keyboard::layouts::KeyboardLayout;
use crate::db::{AppStats, MouseStats, NetworkStats};
use ratatui::widgets::{ListState, TableState};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum UnitSystem {
    #[default]
    Metric,
    Centimeters,
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ScrollMode {
    #[default]
    Lifetime,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SortOrder {
    #[default]
    Descending,
    Ascending,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AppSortMode {
    #[default]
    Keys,
    Clicks,
    Scrolls,
    Download,
    Upload,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum NetworkSortMode {
    #[default]
    Download,
    Upload,
    Total,
    Interface,
}

#[derive(Debug, Clone, Default)]
pub struct ExtendedMouseStats {
    pub today: MouseStats,
    pub yesterday: MouseStats,
    pub all_time: MouseStats,
    pub unpulsed: MouseStats,
}

#[derive(Debug, Clone, Default)]
pub struct NavigationState {
    pub current_tab: usize,
    pub menu_open: bool,
    pub show_quit_confirm: bool,
}

pub struct MouseState {
    pub stats: ExtendedMouseStats,
    pub screen_heatmap: Vec<Vec<u64>>,
    pub period: TimePeriod,
    pub heatmap_error: Option<String>,
    pub show_stats: bool,
    // Scroll specific
    pub scroll_meters: f64,
    pub scroll_mode: ScrollMode,
    pub session_start_scrolls: Option<u64>,
    pub current_total_scrolls: u64,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            stats: ExtendedMouseStats::default(),
            screen_heatmap: Vec::new(),
            period: TimePeriod::Today,
            heatmap_error: None,
            show_stats: false,
            scroll_meters: 0.0,
            scroll_mode: ScrollMode::default(),
            session_start_scrolls: None,
            current_total_scrolls: 0,
        }
    }
}

pub struct KeyboardState {
    pub profiles: Vec<SwitchProfile>,
    pub profile_index: usize,
    pub layout: KeyboardLayout,
    pub show_layout_popup: bool,
    pub layout_search_query: String,
    pub layout_list_state: RefCell<ListState>,
    pub heatmap_data: HashMap<String, u64>,
    pub session_heatmap: HashMap<String, u64>,
    pub heatmap_error: Option<String>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            profiles: vec![
                SwitchProfile::cherry_mx_red(),
                SwitchProfile::cherry_mx_blue(),
                SwitchProfile::cherry_mx_brown(),
                SwitchProfile::membrane(),
            ],
            profile_index: 0,
            layout: KeyboardLayout::Qwerty,
            show_layout_popup: false,
            layout_search_query: String::new(),
            layout_list_state: RefCell::new(ListState::default()),
            heatmap_data: HashMap::new(),
            session_heatmap: HashMap::new(),
            heatmap_error: None,
        }
    }
}

pub struct AppsState {
    pub stats: Vec<AppStats>,
    pub period: TimePeriod,
    pub table_state: RefCell<TableState>,
    pub sort_mode: AppSortMode,
    pub sort_order: SortOrder,
}

impl Default for AppsState {
    fn default() -> Self {
        Self {
            stats: Vec::new(),
            period: TimePeriod::All,
            table_state: RefCell::new(TableState::default()),
            sort_mode: AppSortMode::default(),
            sort_order: SortOrder::default(),
        }
    }
}

pub struct NetworkState {
    pub stats: Vec<NetworkStats>,
    pub period: TimePeriod,
    pub table_state: RefCell<TableState>,
    pub sort_mode: NetworkSortMode,
    pub sort_order: SortOrder,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            stats: Vec::new(),
            period: TimePeriod::All,
            table_state: RefCell::new(TableState::default()),
            sort_mode: NetworkSortMode::default(),
            sort_order: SortOrder::default(),
        }
    }
}
