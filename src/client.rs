use crate::key_mapping::map_key_id_to_name;
use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Local};
use log::{debug, info, warn};
use reqwest::Client;
use rusqlite::Connection;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::task;

/// WhatPulse Web API client using bearer authentication.
#[derive(Clone)]
pub struct WhatpulseClient {
    client: Client,
    base_url: String,
    _user_id: String,
    is_local: bool,
}

impl WhatpulseClient {
    pub async fn new(api_key: &str) -> Result<Self> {
        // Parse user ID from JWT (middle part) if possible
        // New API keys might be opaque, but we'll try to extract ID.
        // If the key is not a JWT, we might need another way to get the ID.
        // For now, we assume the key structure allows extraction or we fail.
        let user_id = Self::extract_user_id(api_key).unwrap_or_else(|_| {
            // Fallback: If we can't extract, we use "me" and rely on /user endpoints.
            "me".to_string()
        });

        use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

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
            base_url: "https://whatpulse.org/api/v1".to_string(),
            _user_id: user_id,
            is_local: false,
        })
    }

    pub fn new_local() -> Result<Self> {
        let client = Client::builder()
            .user_agent("whatpulse-rs/0.1.0")
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url: "http://localhost:3490".to_string(),
            _user_id: "local".to_string(),
            is_local: true,
        })
    }

    fn extract_user_id(api_key: &str) -> Result<String> {
        let parts: Vec<&str> = api_key.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid API key format (expected JWT)"));
        }
        let payload = parts[1];
        let decoded = URL_SAFE_NO_PAD
            .decode(payload)
            .context("failed to decode JWT payload")?;
        let json: Value =
            serde_json::from_slice(&decoded).context("failed to parse JWT payload as JSON")?;

        json.get("sub")
            .and_then(|v| v.as_str())
            .or_else(|| {
                json.get("uid").and_then(|v| {
                    v.as_str()
                        .or_else(|| v.as_u64().map(|u| u.to_string().leak() as &str))
                })
            })
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("JWT payload missing 'sub' claim"))
    }

    pub fn is_local(&self) -> bool {
        self.is_local
    }

    // -- Typed API Methods --

    pub async fn get_user(&self) -> Result<UserResponse> {
        if self.is_local {
            return self.get_user_local().await;
        }
        let url = format!("/users/{}", self._user_id);
        let wrapper = self.get_json::<UserWrapper>(&url).await?;
        Ok(wrapper.user)
    }

    pub async fn get_pulses(&self) -> Result<Vec<PulseResponse>> {
        if self.is_local {
            return Ok(Vec::new()); // Local API doesn't support history
        }
        let url = format!("/users/{}/pulses", self._user_id);
        let wrapper = self.get_json::<PulseListResponse>(&url).await?;
        Ok(wrapper.pulses)
    }

    pub async fn get_computers(&self) -> Result<Vec<ComputerResponse>> {
        if self.is_local {
            return Ok(Vec::new());
        }
        let url = format!("/users/{}/computers", self._user_id);
        // Assuming ComputerListResponse is just a wrapper, or directly list?
        // Let's assume list for now based on other patterns, or check ComputerListResponse definition.
        // If the API returns { computers: [...] }, we need to keep ComputerListResponse but remove ApiResponse wrapper.
        let resp = self.get_json::<ComputerListResponse>(&url).await?;
        Ok(resp.computers)
    }

    // -- Internal Helpers --

    async fn get_user_local(&self) -> Result<UserResponse> {
        let url = format!("{}/v1/account-totals", self.base_url);
        let val = self.get_json::<Value>(&url).await?;

        // Map Local API format to New Web API format
        let keys = val
            .get("keys")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let clicks = val
            .get("clicks")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let uptime = val
            .get("uptime")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        let download_mb = val
            .get("download")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);
        let upload_mb = val
            .get("upload")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);

        Ok(UserResponse {
            id: 0,
            username: "Local User".to_string(),
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
            totals: UserTotals {
                keys: Some(keys),
                clicks: Some(clicks),
                download_mb: Some(download_mb),
                upload_mb: Some(upload_mb),
                uptime_seconds: Some(uptime),
                scrolls: 0,
                distance_miles: Some(0.0),
            },
            ranks: None,
            include_in_rankings: false,
            distance_system: "metric".to_string(),
            last_pulse: None,
        })
    }

    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            // Ensure path starts with / if base_url doesn't end with /
            // New API base_url: https://whatpulse.org/api/v1 (no trailing slash)
            if !path.starts_with('/') {
                format!("{}/{}", self.base_url, path)
            } else {
                format!("{}{}", self.base_url, path)
            }
        };

        debug!("Requesting JSON from: {}", url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("request failed: GET {}", url))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            // Truncate if too long or HTML
            let error_msg =
                if text.trim().starts_with("<!DOCTYPE") || text.trim().starts_with("<html") {
                    format!("(HTML response, length: {})", text.len())
                } else {
                    text.chars().take(200).collect::<String>()
                };
            return Err(anyhow!("API Error {}: {}", status, error_msg));
        }

        let text = resp
            .text()
            .await
            .with_context(|| format!("failed to read text from {}", url))?;
        serde_json::from_str::<T>(&text)
            .with_context(|| format!("failed to parse JSON from {}: {}", url, text))
    }

    pub async fn get_heatmap(&self, period: &str) -> Result<(HashMap<String, u64>, String)> {
        // WhatPulse Public API does not expose a Heatmap endpoint (JSON).
        // We must rely on the local SQLite database for this data.

        if self.is_local {
            let map = self.get_heatmap_from_db(period).await?;
            return Ok((map, "Local DB".to_string()));
        }

        // Even if in API mode, we fallback to Local DB because API doesn't exist for this.
        match self.get_heatmap_from_db(period).await {
            Ok(db_map) => Ok((db_map, "Local DB (API)".to_string())),
            Err(e) => Err(e.context("Failed to load Heatmap from Local DB (API)")),
        }
    }

    async fn get_heatmap_from_db(&self, period: &str) -> Result<HashMap<String, u64>> {
        // Use LOCALAPPDATA environment variable
        let local_app_data = std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| r"C:\Users\danalec\AppData\Local".to_string());

        let path = PathBuf::from(local_app_data)
            .join("WhatPulse")
            .join("whatpulse.db");

        if !path.exists() {
            return Err(anyhow!("Local WhatPulse database not found at {:?}", path));
        }

        let period_owned = period.to_string();

        task::spawn_blocking(move || {
            let conn = Connection::open(&path)
                .context(format!("failed to open local database at {:?}", path))?;

            let (where_clause, param) = match period_owned.as_str() {
                "today" => {
                    let d = Local::now().format("%Y-%m-%d").to_string();
                    ("WHERE day = ?", Some(d))
                }
                "yesterday" => {
                    let d = (Local::now() - Duration::days(1))
                        .format("%Y-%m-%d")
                        .to_string();
                    ("WHERE day = ?", Some(d))
                }
                "week" => {
                    let d = (Local::now() - Duration::days(7))
                        .format("%Y-%m-%d")
                        .to_string();
                    ("WHERE day >= ?", Some(d))
                }
                "month" => {
                    let d = (Local::now() - Duration::days(30))
                        .format("%Y-%m-%d")
                        .to_string();
                    ("WHERE day >= ?", Some(d))
                }
                "year" => {
                    let d = (Local::now() - Duration::days(365))
                        .format("%Y-%m-%d")
                        .to_string();
                    ("WHERE day >= ?", Some(d))
                }
                _ => ("", None),
            };

            let sql = format!(
                "SELECT key, sum(count) as total FROM keypress_frequency {} GROUP BY key",
                where_clause
            );
            let mut stmt = conn.prepare(&sql)?;

            let params: Vec<&dyn rusqlite::ToSql> = if let Some(ref p) = param {
                vec![p]
            } else {
                vec![]
            };

            let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?;

            let mut map = HashMap::new();
            let mut row_count = 0;
            for (key_id, count) in rows.flatten() {
                let key_name = map_key_id_to_name(key_id);
                *map.entry(key_name).or_insert(0) += count as u64;
                row_count += 1;
            }

            if map.is_empty() {
                // Don't error out if no data for a specific period, just return empty map + warning log
                warn!(
                    "Local DB returned no heatmap data from {:?} for period {}. Row count: {}",
                    path, period_owned, row_count
                );
            } else {
                info!(
                    "Loaded {} keys from local DB at {:?} for period {}.",
                    map.len(),
                    path,
                    period_owned
                );
            }

            Ok(map)
        })
        .await?
    }
}

// -- API Response Structs --

#[derive(Debug, Deserialize)]
struct UserWrapper {
    user: UserResponse,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PaginationResource {
    pub total: u64,
    #[serde(rename = "last_page")]
    pub total_pages: u64,
    #[serde(rename = "per_page")]
    pub per_page: u64,
    #[serde(rename = "current_page")]
    pub current_page: u64,
    pub from: Option<u64>,
    pub to: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct LinksResource {
    pub first: String,
    pub last: String,
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PulseFilters {
    pub computer_id: Option<u64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PulseListResponse {
    pub pulses: Vec<PulseResponse>,
    pub pagination: Option<PaginationResource>,
    pub links: Option<LinksResource>,
    pub filters: Option<PulseFilters>,
}

#[derive(Debug, Deserialize)]
pub struct UserResponse {
    pub id: u64,
    pub username: String,
    #[serde(rename = "date_joined")]
    pub date_joined: Option<String>,
    #[serde(rename = "first_pulse_date")]
    pub first_pulse_date: Option<String>,
    #[serde(rename = "last_pulse_date")]
    pub last_pulse_date: Option<String>,
    #[serde(default)]
    pub pulses: u64,
    #[serde(rename = "team_id")]
    pub team_id: Option<u64>,
    #[serde(rename = "team_is_manager", default)]
    pub team_is_manager: bool,
    #[serde(rename = "country_id")]
    pub country_id: Option<u64>,
    #[serde(rename = "is_premium", default)]
    pub is_premium: bool,
    #[serde(default)]
    pub referrals: u64,
    #[serde(rename = "last_referral_date")]
    pub last_referral_date: Option<String>,
    pub avatar: Option<String>,
    pub totals: UserTotals,
    pub ranks: Option<UserRanks>,
    #[serde(rename = "include_in_rankings", default)]
    pub include_in_rankings: bool,
    #[serde(rename = "distance_system", default)]
    pub distance_system: String,
    #[serde(rename = "last_pulse")]
    pub last_pulse: Option<LastPulse>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LastPulse {
    pub date: String,
    pub keys: Option<u64>,
    pub clicks: Option<u64>,
    #[serde(rename = "download_mb")]
    pub download_mb: Option<f64>,
    #[serde(rename = "upload_mb")]
    pub upload_mb: Option<f64>,
    #[serde(rename = "uptime_seconds")]
    pub uptime_seconds: Option<u64>,
    pub scrolls: Option<u64>,
    #[serde(rename = "distance_miles")]
    pub distance_miles: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserTotals {
    pub keys: Option<u64>,
    pub clicks: Option<u64>,
    #[serde(rename = "download_mb")]
    pub download_mb: Option<f64>,
    #[serde(rename = "upload_mb")]
    pub upload_mb: Option<f64>,
    #[serde(rename = "uptime_seconds")]
    pub uptime_seconds: Option<u64>,
    #[serde(default)]
    pub scrolls: u64,
    #[serde(rename = "distance_miles", default)]
    pub distance_miles: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserRanks {
    pub keys: u64,
    pub clicks: u64,
    pub download: u64,
    pub upload: u64,
    pub uptime: u64,
    pub scrolls: u64,
    pub distance: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PulseResponse {
    pub id: u64,
    pub date: String,
    pub keys: Option<u64>,
    pub clicks: Option<u64>,
    #[serde(rename = "download_mb")]
    pub download_mb: Option<f64>,
    #[serde(rename = "upload_mb")]
    pub upload_mb: Option<f64>,
    #[serde(rename = "uptime_seconds")]
    pub uptime_seconds: Option<u64>,
    pub scrolls: Option<u64>,
    #[serde(rename = "distance_miles")]
    pub distance_miles: Option<f64>,
    #[serde(rename = "auto_pulse")]
    pub auto_pulse: Option<bool>,
    #[serde(rename = "client_version")]
    pub client_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ComputerFilters {
    pub is_archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ComputerListResponse {
    pub computers: Vec<ComputerResponse>,
    #[allow(dead_code)]
    pub pagination: Option<PaginationResource>,
    #[allow(dead_code)]
    pub links: Option<LinksResource>,
    #[allow(dead_code)]
    pub filters: Option<ComputerFilters>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComputerResponse {
    pub id: u64,
    pub name: String,
    #[serde(rename = "client_version")]
    pub client_version: String,
    pub os: String,
    #[serde(rename = "is_archived", default)]
    pub is_archived: bool,
    pub totals: ComputerTotals,
    pub pulses: Option<u64>,
    #[serde(rename = "last_pulse_date")]
    pub last_pulse_date: Option<String>,
    pub hardware: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ComputerTotals {
    pub keys: u64,
    pub clicks: u64,
    #[serde(rename = "download_mb")]
    pub download_mb: Option<f64>,
    #[serde(rename = "upload_mb")]
    pub upload_mb: Option<f64>,
    #[serde(rename = "uptime_seconds")]
    pub uptime_seconds: Option<u64>,
    pub scrolls: Option<u64>,
    #[serde(rename = "distance_miles")]
    pub distance_miles: Option<f64>,
}
