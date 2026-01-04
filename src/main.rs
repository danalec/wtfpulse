use anyhow::Result;
use clap::Parser;
use simplelog::*;
use std::env;
use std::fs::File;

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
    command: Option<Commands>,
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
    let client = match env::var("WHATPULSE_API_KEY") {
        Ok(key) => WhatpulseClient::new(&key).await?,
        Err(_) => {
            eprintln!(
                "Warning: WHATPULSE_API_KEY not set. Falling back to Local Client API (http://localhost:3490)."
            );
            WhatpulseClient::new_local()?
        }
    };

    let command = args.command.unwrap_or(Commands::Tui);
    command.execute(&client).await
}
