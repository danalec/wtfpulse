use crate::client::WhatpulseClient;
use crate::tui::app::App;
use anyhow::Result;
use clap::Subcommand;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;

pub mod calorimetry;
pub mod computers;
pub mod heatmap;
pub mod monitor;
pub mod pulses;
pub mod raw;
pub mod scroll_tower;
pub mod tui;
pub mod user;

pub struct TuiPage {
    pub title: &'static str,
    pub render: fn(&mut Frame, &App, Rect),
    pub handle_key: fn(&mut App, KeyEvent) -> bool,
    pub priority: usize,
}

inventory::collect!(TuiPage);

pub fn get_pages() -> Vec<&'static TuiPage> {
    let mut pages: Vec<&'static TuiPage> = inventory::iter::<TuiPage>.into_iter().collect();
    pages.sort_by_key(|p| p.priority);
    pages
}

#[derive(Subcommand)]
pub enum Commands {
    /// Fetch current user stats
    User,
    /// Fetch recent pulses
    Pulses,
    /// Fetch computers list
    Computers,
    /// Calculate energy expenditure
    Calorimetry,
    /// Launch the interactive dashboard
    Tui,
    /// Fetch raw JSON from a specific path
    Raw {
        /// The API path (e.g., /api/v1/user)
        path: String,
    },
    /// Monitor real-time pulses (CLI Mode)
    Monitor,
}

impl Commands {
    pub async fn execute(self, client: &WhatpulseClient) -> Result<()> {
        match self {
            Commands::User => user::execute(client).await,
            Commands::Pulses => pulses::execute(client).await,
            Commands::Computers => computers::execute(client).await,
            Commands::Calorimetry => calorimetry::execute(client).await,
            Commands::Tui => tui::execute(client).await,
            Commands::Raw { path } => raw::execute(client, path).await,
            Commands::Monitor => monitor::execute(client).await,
        }
    }
}
