use clap::Subcommand;
use anyhow::Result;
use crate::client::WhatpulseClient;

pub mod calorimetry;
pub mod user;
pub mod pulses;
pub mod computers;
pub mod raw;

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
    /// Fetch raw JSON from a specific path
    Raw {
        /// The API path (e.g., /api/v1/user)
        path: String,
    },
}

impl Commands {
    pub async fn execute(self, client: &WhatpulseClient) -> Result<()> {
        match self {
            Commands::User => user::execute(client).await,
            Commands::Pulses => pulses::execute(client).await,
            Commands::Computers => computers::execute(client).await,
            Commands::Calorimetry => calorimetry::execute(client).await,
            Commands::Raw { path } => raw::execute(client, path).await,
        }
    }
}
