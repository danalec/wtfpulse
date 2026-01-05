use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use log::debug;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct ClientCache {
    pulses: Option<(Vec<PulseResponse>, Instant)>,
    user: Option<(UserResponse, Instant)>,
    computers: Option<(Vec<ComputerResponse>, Instant)>,
}

#[derive(Clone)]
pub struct WhatpulseClient {
    client: Client,
    base_url: String,
    _user_id: String,
    is_local: bool,
    cache: Arc<Mutex<ClientCache>>,
}

impl WhatpulseClient {
    pub async fn new(api_key: &str) -> Result<Self> {
        let user_id = Self::extract_user_id(api_key).unwrap_or_else(|_| "me".to_string());

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
            cache: Arc::new(Mutex::new(ClientCache {
                pulses: None,
                user: None,
                computers: None,
            })),
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
            cache: Arc::new(Mutex::new(ClientCache {
                pulses: None,
                user: None,
                computers: None,
            })),
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

    pub async fn get_user(&self) -> Result<UserResponse> {
        if self.is_local {
            return self.get_user_local().await;
        }

        // Check cache
        if let Ok(cache) = self.cache.lock() {
            if let Some((user, timestamp)) = &cache.user {
                if timestamp.elapsed() < Duration::from_secs(300) {
                    debug!("Returning cached user");
                    return Ok(user.clone());
                }
            }
        }

        let url = format!("/users/{}", self._user_id);
        let wrapper = self.get_json::<UserWrapper>(&url).await?;

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.user = Some((wrapper.user.clone(), Instant::now()));
        }

        Ok(wrapper.user)
    }

    pub async fn get_pulses(&self) -> Result<Vec<PulseResponse>> {
        if self.is_local {
            return Ok(Vec::new());
        }

        // Check cache
        if let Ok(cache) = self.cache.lock() {
            if let Some((pulses, timestamp)) = &cache.pulses {
                if timestamp.elapsed() < Duration::from_secs(300) {
                    debug!("Returning cached pulses");
                    return Ok(pulses.clone());
                }
            }
        }

        let mut all_pulses = Vec::new();
        let mut current_url = Some(format!("/users/{}/pulses?per_page=100", self._user_id));
        let mut page_count = 0;

        while let Some(url) = current_url {
            let wrapper = self.get_json::<PulseListResponse>(&url).await?;
            all_pulses.extend(wrapper.pulses);

            current_url = wrapper.links.and_then(|l| l.next);

            // Be a good citizen: yield and sleep slightly between pages
            // This prevents hammering the API in a tight loop
            page_count += 1;
            if page_count % 5 == 0 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            } else {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.pulses = Some((all_pulses.clone(), Instant::now()));
        }

        Ok(all_pulses)
    }

    pub async fn get_computers(&self) -> Result<Vec<ComputerResponse>> {
        if self.is_local {
            return Ok(Vec::new());
        }

        // Check cache
        if let Ok(cache) = self.cache.lock() {
            if let Some((computers, timestamp)) = &cache.computers {
                if timestamp.elapsed() < Duration::from_secs(300) {
                    debug!("Returning cached computers");
                    return Ok(computers.clone());
                }
            }
        }

        let url = format!("/users/{}/computers", self._user_id);
        let resp = self.get_json::<ComputerListResponse>(&url).await?;

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.computers = Some((resp.computers.clone(), Instant::now()));
        }

        Ok(resp.computers)
    }

    async fn get_user_local(&self) -> Result<UserResponse> {
        let url = format!("{}/v1/account-totals", self.base_url);
        let val = self.get_json::<Value>(&url).await?;
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

        let scrolls = val
            .get("scrolls")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

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
                scrolls,
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
        } else if !path.starts_with('/') {
            format!("{}/{}", self.base_url, path)
        } else {
            format!("{}{}", self.base_url, path)
        };

        debug!("Requesting JSON from: {}", url);

        let max_retries = 3;
        let mut retry_count = 0;
        let mut backoff_ms = 1000; // 1 second start

        loop {
            // We need to clone the request builder or build it new each time?
            // Client is reusable, so we build the request each iteration.
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .with_context(|| format!("request failed: GET {}", url))?;

            let status = resp.status();
            if status.is_success() {
                let text = resp
                    .text()
                    .await
                    .with_context(|| format!("failed to read text from {}", url))?;
                return serde_json::from_str::<T>(&text)
                    .with_context(|| format!("failed to parse JSON from {}: {}", url, text));
            }

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if retry_count >= max_retries {
                    let text = resp.text().await.unwrap_or_default();
                    return Err(anyhow!(
                        "API Error {}: Too many requests (rate limit exceeded). Body: {}",
                        status,
                        text
                    ));
                }

                // Parse Retry-After if available (seconds), otherwise exponential backoff
                let wait_ms =
                    if let Some(retry_after) = resp.headers().get(reqwest::header::RETRY_AFTER) {
                        retry_after
                            .to_str()
                            .ok()
                            .and_then(|s| s.parse::<u64>().ok())
                            .map(|s| s * 1000) // Convert seconds to ms
                            .unwrap_or(backoff_ms)
                    } else {
                        backoff_ms
                    };

                debug!(
                    "Rate limited. Waiting {}ms before retry {}/{}",
                    wait_ms,
                    retry_count + 1,
                    max_retries
                );
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

                retry_count += 1;
                backoff_ms *= 2; // Exponential backoff
                continue;
            }

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
    }

    pub async fn get_heatmap(&self, period: &str) -> Result<(HashMap<String, u64>, String)> {
        let period_owned = period.to_string();

        let map = tokio::task::spawn_blocking(move || -> Result<HashMap<String, u64>> {
            let db = crate::db::Database::new()?;
            db.get_heatmap_stats(&period_owned)
        })
        .await??;

        Ok((map, "Local DB".to_string()))
    }

    pub async fn get_screen_heatmap(&self, period: &str) -> Result<Vec<Vec<u64>>> {
        let period_owned = period.to_string();

        let grid = tokio::task::spawn_blocking(move || -> Result<Vec<Vec<u64>>> {
            let db = crate::db::Database::new()?;
            let points = db.get_mouse_points(&period_owned)?;

            if points.is_empty() {
                return Ok(Vec::new());
            }

            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;

            for (x, y) in &points {
                if *x < min_x {
                    min_x = *x;
                }
                if *x > max_x {
                    max_x = *x;
                }
                if *y < min_y {
                    min_y = *y;
                }
                if *y > max_y {
                    max_y = *y;
                }
            }

            // Heuristic: If values are normalized (0.0-1.0), use 0-1 bounds to preserve screen context
            // Otherwise use observed bounds (auto-zoom)
            let is_normalized = min_x >= 0.0 && max_x <= 1.0 && min_y >= 0.0 && max_y <= 1.0;

            let (use_min_x, use_max_x, use_min_y, use_max_y) = if is_normalized {
                (0.0, 1.0, 0.0, 1.0)
            } else {
                (min_x, max_x, min_y, max_y)
            };

            // Grid size: 320x200 (approx VGA resolution, good for TUI)
            let grid_w = 320;
            let grid_h = 200;
            let mut grid = vec![vec![0u64; grid_w]; grid_h];

            let width = use_max_x - use_min_x;
            let height = use_max_y - use_min_y;

            if width <= 0.0 || height <= 0.0 {
                return Ok(grid);
            }

            for (x, y) in points {
                let norm_x = (x - use_min_x) / width;
                let norm_y = (y - use_min_y) / height;

                // Clamp to [0, 1] just in case floating point errors or outlier if not normalized
                let norm_x = norm_x.clamp(0.0, 1.0);
                let norm_y = norm_y.clamp(0.0, 1.0);

                let gx = (norm_x * (grid_w as f64 - 1.0)).round() as usize;
                let gy = (norm_y * (grid_h as f64 - 1.0)).round() as usize;

                if gx < grid_w && gy < grid_h {
                    grid[gy][gx] += 1;
                }
            }

            Ok(grid)
        })
        .await??;

        Ok(grid)
    }
}

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

#[derive(Debug, Deserialize, Clone)]
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
