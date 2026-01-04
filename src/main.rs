use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;

/// WhatPulse Web API client using bearer authentication.
pub struct WhatpulseClient {
    client: Client,
    base_url: String,
    user_id: String,
}

impl WhatpulseClient {
    pub async fn new(api_key: &str) -> Result<Self> {
        // Parse user ID from JWT (middle part)
        let parts: Vec<&str> = api_key.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid API key format (expected JWT)"));
        }
        let payload = parts[1];
        let decoded = URL_SAFE_NO_PAD
            .decode(payload)
            .context("failed to decode JWT payload")?;
        let json: Value = serde_json::from_slice(&decoded)
            .context("failed to parse JWT payload as JSON")?;
        
        let user_id = json
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("JWT payload missing 'sub' claim"))?
            .to_string();

        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

        let mut headers = HeaderMap::new();
        let value = format!("Bearer {}", api_key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&value).context("invalid Authorization header value")?,
        );

        let client = Client::builder()
            .user_agent("whatpulse-rs/0.1.0")
            .default_headers(headers)
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url: "https://api.whatpulse.org".to_string(),
            user_id,
        })
    }

    /// Helper to fetch JSON from the correct PHP endpoint
    async fn get_resource<T: DeserializeOwned>(&self, resource: &str) -> Result<T> {
        // Map abstract resource to PHP endpoint
        let endpoint = match resource {
            "user" => "user.php",
            "pulses" => "pulses.php",
            _ => return Err(anyhow!("Unknown resource type: {}", resource)),
        };

        let url = format!(
            "{}/{}?user={}&format=json",
            self.base_url, endpoint, self.user_id
        );

        self.get_json(&url).await
    }

    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            // Ensure path starts with / if base_url doesn't end with /
            if !path.starts_with('/') {
                format!("{}/{}", self.base_url, path)
            } else {
                format!("{}{}", self.base_url, path)
            }
        };

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("request failed: GET {}", url))?
            .error_for_status()
            .with_context(|| format!("non-success status from {}", url))?
            .json::<T>()
            .await
            .with_context(|| format!("failed to parse JSON from {}", url))?;

        Ok(resp)
    }

    pub async fn get_text(&self, path: &str) -> Result<String> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            if !path.starts_with('/') {
                format!("{}/{}", self.base_url, path)
            } else {
                format!("{}{}", self.base_url, path)
            }
        };

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("request failed: GET {}", url))?
            .error_for_status()
            .with_context(|| format!("non-success status from {}", url))?
            .text()
            .await
            .with_context(|| format!("failed to get text from {}", url))?;

        Ok(resp)
    }
}

#[derive(Debug, Deserialize)]
pub struct UserResponse {
    #[serde(alias = "UserID")]
    pub id: Option<String>,
    #[serde(alias = "AccountName")]
    pub account_name: Option<String>,
    #[serde(alias = "Keys")]
    pub keys: Option<String>,
    #[serde(alias = "Clicks")]
    pub clicks: Option<String>,
    #[serde(alias = "Computers")]
    pub computers: Option<HashMap<String, ComputerResponse>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct PulseResponse {
    #[serde(alias = "PulseID")]
    pub id: Option<String>,
    #[serde(alias = "Timedate")]
    pub date: Option<String>,
    #[serde(alias = "Keys")]
    pub keys: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct ComputerResponse {
    #[serde(alias = "ComputerID")]
    pub id: Option<String>,
    #[serde(alias = "Name")]
    pub name: Option<String>,
    #[serde(alias = "OS")]
    pub os: Option<String>,
    #[serde(alias = "Keys")]
    pub keys: Option<String>,
    #[serde(alias = "Clicks")]
    pub clicks: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Parser)]
#[command(name = "wtfpulse")]
#[command(about = "A WhatPulse Web API client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch current user stats
    User,
    /// Fetch recent pulses
    Pulses,
    /// Fetch computers list
    Computers,
    /// Fetch raw JSON from a specific path
    Raw {
        /// The API path (e.g., /api/v1/user)
        path: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    
    // Read `WHATPULSE_API_KEY` from environment.
    let api_key = env::var("WHATPULSE_API_KEY")
        .context("set WHATPULSE_API_KEY environment variable with your API token")?;

    let client = WhatpulseClient::new(&api_key).await?;

    match args.command {
        Commands::User => {
            let user = client.get_resource::<UserResponse>("user").await?;
            println!("User: {} (ID: {})", 
                user.account_name.as_deref().unwrap_or("unknown"), 
                user.id.as_deref().unwrap_or("unknown")
            );
            if let Some(keys) = &user.keys {
                println!("Keys: {}", keys);
            }
            if let Some(clicks) = &user.clicks {
                println!("Clicks: {}", clicks);
            }
        }
        Commands::Pulses => {
            let pulses_map = client.get_resource::<HashMap<String, PulseResponse>>("pulses").await?;
            println!("Found {} pulses:", pulses_map.len());
            
            // Convert to vector and sort by key (Pulse ID) descending to show newest first
            let mut pulses: Vec<_> = pulses_map.into_iter().collect();
            // Pulse IDs are strings like "Pulse-123", so string sort works reasonably well for ordering
            pulses.sort_by(|a, b| b.0.cmp(&a.0));

            for (id, pulse) in pulses.iter().take(5) {
                println!("{}: {} keys on {}", 
                    id, 
                    pulse.keys.as_deref().unwrap_or("0"),
                    pulse.date.as_deref().unwrap_or("unknown date")
                );
            }
        }
        Commands::Computers => {
            // Computer stats are nested inside the User response
            let user = client.get_resource::<UserResponse>("user").await?;
            if let Some(computers) = user.computers {
                println!("Found {} computers:", computers.len());
                for (_, comp) in computers {
                    println!("{} ({}): {} keys, {} clicks", 
                        comp.name.as_deref().unwrap_or("unknown"),
                        comp.id.as_deref().unwrap_or("unknown"),
                        comp.keys.as_deref().unwrap_or("0"),
                        comp.clicks.as_deref().unwrap_or("0")
                    );
                }
            } else {
                println!("No computers found in user profile.");
            }
        }
        Commands::Raw { path } => {
            let text = client.get_text(&path).await?;
            println!("{}", text);
        }
    }

    Ok(())
}
