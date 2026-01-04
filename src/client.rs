use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

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
    pub async fn get_resource<T: DeserializeOwned>(&self, resource: &str) -> Result<T> {
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
