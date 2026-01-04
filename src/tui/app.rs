use anyhow::Result;
use tokio::sync::mpsc;
use crate::client::{WhatpulseClient, UserResponse, PulseResponse};
use crate::commands::calorimetry::{calculate_energy, EnergyStats, SwitchProfile};
use crate::commands::get_pages;
use crossterm::event::{KeyCode, KeyEvent};

pub enum Action {
    Tick,
    Quit,
    Refresh,
    Key(KeyEvent),
    UserLoaded(Result<UserResponse>),
    PulsesLoaded(Result<Vec<PulseResponse>>),
}

use chrono::{NaiveDate, Local};

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

pub struct App {
    pub user_stats: Option<UserResponse>,
    pub recent_pulses: Vec<PulseResponse>,
    pub energy_stats: Option<EnergyStats>,
    pub user_loading: bool,
    pub pulses_loading: bool,
    pub error: Option<String>,
    pub pulses_error: Option<String>,
    pub current_tab: usize,
    pub client: WhatpulseClient,
    pub tx: mpsc::Sender<Action>,
    pub profiles: Vec<SwitchProfile>,
    pub profile_index: usize,
    pub dashboard_period: TimePeriod,
    pub date_picker: DatePickerState,
}

impl App {
    pub fn new(client: WhatpulseClient, tx: mpsc::Sender<Action>) -> Self {
        Self {
            user_stats: None,
            recent_pulses: Vec::new(),
            energy_stats: None,
            user_loading: true,
            pulses_loading: true,
            error: None,
            pulses_error: None,
            current_tab: 0,
            client,
            tx,
            profiles: vec![
                SwitchProfile::cherry_mx_red(),
                SwitchProfile::cherry_mx_blue(),
                SwitchProfile::cherry_mx_brown(),
                SwitchProfile::membrane(),
            ],
            profile_index: 0,
            dashboard_period: TimePeriod::default(),
            date_picker: DatePickerState::default(),
        }
    }

    pub fn current_profile(&self) -> &SwitchProfile {
        &self.profiles[self.profile_index]
    }

    pub fn recalculate_energy(&mut self) {
        if let Some(keys) = self.user_stats.as_ref().and_then(|u| u.keys.as_deref()) {
            let profile = self.current_profile();
            self.energy_stats = calculate_energy(keys, Some(profile)).ok();
        }
    }

    pub async fn update(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,
            Action::Tick => {},
            Action::Refresh => {
                self.user_loading = true;
                self.pulses_loading = true;
                spawn_fetch(self.client.clone(), self.tx.clone());
            }
            Action::Key(key) => {
                let pages = get_pages();
                
                // Let the current page handle the key first
                let mut handled = false;
                if let Some(page) = pages.get(self.current_tab) {
                    handled = (page.handle_key)(self, key);
                }

                if !handled {
                    match key.code {
                        KeyCode::Esc => return true,
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
                match res {
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
        }
        false
    }
}

pub fn spawn_fetch(client: WhatpulseClient, tx: mpsc::Sender<Action>) {
    let tx_user = tx.clone();
    let client_user = client.clone();
    tokio::spawn(async move {
        let res = client_user.get_resource::<UserResponse>("user").await;
        let _ = tx_user.send(Action::UserLoaded(res)).await;
    });

    let tx_pulses = tx.clone();
    let client_pulses = client.clone();
    tokio::spawn(async move {
        let res = client_pulses.get_resource::<Vec<PulseResponse>>("pulses").await;
        let _ = tx_pulses.send(Action::PulsesLoaded(res)).await;
    });
}
