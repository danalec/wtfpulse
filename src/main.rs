use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::fs::File;
use simplelog::*;

mod client;
mod commands;
pub mod tui;

use client::WhatpulseClient;
use commands::Commands;

#[derive(Parser)]
#[command(name = "wtfpulse")]
#[command(about = "A WhatPulse Web API client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    // Initialize logging
    if let Ok(file) = File::create("wtfpulse.log") {
        let _ = WriteLogger::init(LevelFilter::Info, Config::default(), file);
    }

    let args = Cli::parse();
    
    // Read `WHATPULSE_API_KEY` from environment.
    let api_key = env::var("WHATPULSE_API_KEY")
        .context("set WHATPULSE_API_KEY environment variable with your API token")?;

    let client = WhatpulseClient::new(&api_key).await?;

    args.command.execute(&client).await
}
