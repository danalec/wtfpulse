use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use log::debug;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

fn from_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;
    match v {
        Value::String(s) => Ok(Some(s)),
        Value::Number(n) => Ok(Some(n.to_string())),
        Value::Null => Ok(None),
        _ => Err(serde::de::Error::custom("expected string or number")),
    }
}

/// WhatPulse Web API client using bearer authentication.
#[derive(Clone)]
pub struct WhatpulseClient {
    client: Client,
    base_url: String,
    user_id: String,
    is_local: bool,
}

impl WhatpulseClient {
    pub async fn new(api_key: &str) -> Result<Self> {
        // Parse user ID from JWT (middle part)
        let user_id = Self::extract_user_id(api_key)?;

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
            base_url: "https://api.whatpulse.org".to_string(),
            user_id,
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
            user_id: "local".to_string(),
            is_local: true,
        })
    }

    fn extract_user_id(api_key: &str) -> Result<String> {
        let parts: Vec<&str> = api_key.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid API key format (expected JWT)"));
        }
        let payload = parts[1];
        // Pad if necessary? JWT is usually unpadded base64url, but base64 crate might be strict.
        // URL_SAFE_NO_PAD handles it.
        let decoded = URL_SAFE_NO_PAD
            .decode(payload)
            .context("failed to decode JWT payload")?;
        let json: Value =
            serde_json::from_slice(&decoded).context("failed to parse JWT payload as JSON")?;

        json.get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("JWT payload missing 'sub' claim"))
            .map(|s| s.to_string())
    }

    pub fn is_local(&self) -> bool {
        self.is_local
    }

    /// Helper to fetch JSON from the correct PHP endpoint
    pub async fn get_resource<T: DeserializeOwned>(&self, resource: &str) -> Result<T> {
        if self.is_local {
            return self.get_resource_local(resource).await;
        }

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

        // Fetch as generic Value first to check for API-level errors (like "No pulses found")
        let val = self.get_json::<Value>(&url).await?;

        if let Some(err_val) = val.get("error") {
            let err_msg = err_val.as_str().unwrap_or("Unknown error");
            // Special case for pulses: "No pulses found!" means empty list, not a hard error
            if resource == "pulses" && err_msg.contains("No pulses found") {
                // Return empty Vec for PulseResponse
                return serde_json::from_value(Value::Array(Vec::new()))
                    .context("failed to deserialize empty pulse list");
            }
            return Err(anyhow!("API Error: {}", err_msg));
        }

        // Special handling for pulses: API returns HashMap<String, PulseResponse>, we want Vec<PulseResponse>
        if resource == "pulses" {
            let map: HashMap<String, PulseResponse> =
                serde_json::from_value(val).context("failed to deserialize pulse map")?;

            // Convert to Vec and sort by timestamp descending
            let mut pulses: Vec<PulseResponse> = map.into_values().collect();
            pulses.sort_by(|a, b| {
                let ta = a
                    .timestamp
                    .as_deref()
                    .unwrap_or("0")
                    .parse::<i64>()
                    .unwrap_or(0);
                let tb = b
                    .timestamp
                    .as_deref()
                    .unwrap_or("0")
                    .parse::<i64>()
                    .unwrap_or(0);
                tb.cmp(&ta) // Descending order
            });

            // Re-serialize to Value to satisfy the generic return type T
            let vec_val = serde_json::to_value(pulses)?;
            return serde_json::from_value(vec_val).context("failed to convert pulse vector");
        }

        serde_json::from_value(val).context("failed to deserialize response")
    }

    async fn get_resource_local<T: DeserializeOwned>(&self, resource: &str) -> Result<T> {
        match resource {
            "user" => {
                let url = format!("{}/v1/account-totals", self.base_url);
                let val = self.get_json::<Value>(&url).await?;

                // Map Local API format to Web API UserResponse format
                let keys = val.get("keys").and_then(|v| v.as_str()).unwrap_or("0");
                let clicks = val.get("clicks").and_then(|v| v.as_str()).unwrap_or("0");
                let uptime = val.get("uptime").and_then(|v| v.as_str()).unwrap_or("0");

                let download_mb_val = val
                    .get("download")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let upload_mb_val = val
                    .get("upload")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                let download_mb = format!("{:.2}", download_mb_val);
                let upload_mb = format!("{:.2}", upload_mb_val);

                let mapped_json = serde_json::json!({
                    "AccountName": "Local User",
                    "UserID": "0",
                    "Country": "Localhost",
                    "DateJoined": "N/A",
                    "Keys": keys,
                    "Clicks": clicks,
                    "DownloadMB": download_mb,
                    "UploadMB": upload_mb,
                    "UptimeSeconds": uptime
                });

                serde_json::from_value(mapped_json).context("failed to map local user stats")
            }
            "pulses" => {
                // Local API doesn't support pulse history
                serde_json::from_value(Value::Array(Vec::new()))
                    .context("failed to return empty pulse list")
            }
            _ => Err(anyhow!("Resource {} not supported in Local Mode", resource)),
        }
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

        debug!("Requesting JSON from: {}", url);

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
        } else if !path.starts_with('/') {
            format!("{}/{}", self.base_url, path)
        } else {
            format!("{}{}", self.base_url, path)
        };

        debug!("Requesting text from: {}", url);

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

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct UserResponse {
    #[serde(rename = "AccountName")]
    pub account_name: Option<String>,
    #[serde(rename = "Country")]
    pub country: Option<String>,
    #[serde(rename = "DateJoined")]
    pub date_joined: Option<String>,
    #[serde(rename = "UserID", default, deserialize_with = "from_string_or_number")]
    pub id: Option<String>,
    #[serde(rename = "Keys", default, deserialize_with = "from_string_or_number")]
    pub keys: Option<String>,
    #[serde(rename = "Clicks", default, deserialize_with = "from_string_or_number")]
    pub clicks: Option<String>,
    #[serde(
        rename = "DownloadMB",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub download_mb: Option<String>,
    #[serde(
        rename = "UploadMB",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub upload_mb: Option<String>,
    #[serde(
        rename = "UptimeSeconds",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub uptime_seconds: Option<String>,
    #[serde(rename = "Computers")]
    pub computers: Option<HashMap<String, ComputerResponse>>,
    #[serde(rename = "Ranks")]
    pub ranks: Option<HashMap<String, Value>>,
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct PulseResponse {
    // PulseID is NOT returned in the pulse object itself by the API
    // Instead, the keys of the JSON object are "Pulse-ID".
    #[serde(rename = "Timedate")]
    pub date: Option<String>,
    #[serde(
        rename = "Timestamp",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub timestamp: Option<String>,
    #[serde(rename = "Keys", default, deserialize_with = "from_string_or_number")]
    pub keys: Option<String>,
    #[serde(rename = "Clicks", default, deserialize_with = "from_string_or_number")]
    pub clicks: Option<String>,
    #[serde(
        rename = "DownloadMB",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub download_mb: Option<String>,
    #[serde(
        rename = "UploadMB",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub upload_mb: Option<String>,
    #[serde(
        rename = "UptimeSeconds",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub uptime_seconds: Option<String>,
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ComputerResponse {
    #[serde(
        rename = "ComputerID",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub id: Option<String>,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "OS")]
    #[allow(dead_code)]
    pub os: Option<String>,
    #[serde(rename = "Keys", default, deserialize_with = "from_string_or_number")]
    pub keys: Option<String>,
    #[serde(rename = "Clicks", default, deserialize_with = "from_string_or_number")]
    pub clicks: Option<String>,
    #[serde(
        rename = "DownloadMB",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub download_mb: Option<String>,
    #[serde(
        rename = "UploadMB",
        default,
        deserialize_with = "from_string_or_number"
    )]
    pub upload_mb: Option<String>,
    #[serde(flatten)]
    #[allow(dead_code)]
    pub extra: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_id() {
        // Helper to create a dummy JWT
        fn create_jwt(sub: &str) -> String {
            let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
            let payload =
                URL_SAFE_NO_PAD.encode(format!(r#"{{"sub":"{}","name":"John Doe"}}"#, sub));
            let signature = "signature";
            format!("{}.{}.{}", header, payload, signature)
        }

        let jwt = create_jwt("12345");
        assert_eq!(WhatpulseClient::extract_user_id(&jwt).unwrap(), "12345");

        // Test invalid format
        assert!(WhatpulseClient::extract_user_id("invalid").is_err());

        // Test missing sub
        let jwt_no_sub = format!(
            "{}.{}.{}",
            URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256"}"#),
            URL_SAFE_NO_PAD.encode(r#"{"name":"No Sub"}"#),
            "sig"
        );
        assert!(WhatpulseClient::extract_user_id(&jwt_no_sub).is_err());
    }

    #[test]
    fn test_deserialize_string_or_number() {
        // Test mixed types in JSON
        let json = r#"{
            "AccountName": "TestUser",
            "UserID": 12345,
            "Keys": "10,000",
            "Clicks": 5000,
            "DownloadMB": "100.5",
            "UploadMB": 200,
            "UptimeSeconds": 3600
        }"#;

        let user: UserResponse = serde_json::from_str(json).unwrap();

        assert_eq!(user.account_name.as_deref(), Some("TestUser"));
        assert_eq!(user.id.as_deref(), Some("12345"));
        assert_eq!(user.keys.as_deref(), Some("10,000"));
        assert_eq!(user.clicks.as_deref(), Some("5000"));
        assert_eq!(user.download_mb.as_deref(), Some("100.5"));
        assert_eq!(user.upload_mb.as_deref(), Some("200"));
        assert_eq!(user.uptime_seconds.as_deref(), Some("3600"));
    }
}
