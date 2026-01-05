use anyhow::Result;
use clap::Parser;
use simplelog::*;
use std::env;
use std::fs::File;

mod client;
mod commands;
pub mod config;
pub mod db;

pub mod key_mapping;
pub mod tui;

use client::WhatpulseClient;
use commands::Commands;
use config::AppConfig;

#[derive(Parser)]
#[command(name = "wtfpulse")]
#[command(about = "A WhatPulse Web API client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenvy::dotenv().ok();

    // Initialize logging
    if let Ok(file) = File::create("wtfpulse.log") {
        let _ = WriteLogger::init(LevelFilter::Info, Config::default(), file);
    }

    let args = Cli::parse();

    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        log::warn!("Failed to load config: {}", e);
        AppConfig::default()
    });

    // Determine API Key: Env Var > Config File
    let api_key = env::var("WHATPULSE_API_KEY").ok().or(config.api_key);

    let client = match api_key {
        Some(key) if !key.is_empty() => WhatpulseClient::new(&key).await?,
        _ => WhatpulseClient::new_local()?,
    };

    let command = args.command.unwrap_or(Commands::Tui);
    command.execute(&client).await
}
