use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub api_key: Option<String>,
    pub refresh_rate_seconds: Option<u64>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        let mut config: AppConfig = if !config_path.exists() {
            Self::default()
        } else {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file at {:?}", config_path))?;
            toml::from_str(&content).with_context(|| "Failed to parse config file")?
        };

        // Environment variable overrides
        if let Ok(key) = std::env::var("WTFPULSE_API_KEY") {
            config.api_key = Some(key);
        }

        if let Ok(rate_str) = std::env::var("WTFPULSE_REFRESH_RATE")
            && let Ok(rate) = rate_str.parse::<u64>()
        {
            config.refresh_rate_seconds = Some(rate);
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
        }

        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file at {:?}", config_path))?;

        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "wtfpulse", "wtfpulse")
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        Ok(proj_dirs.config_dir().join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_var_override() {
        // Save original vars
        let orig_key = env::var("WTFPULSE_API_KEY").ok();
        let orig_rate = env::var("WTFPULSE_REFRESH_RATE").ok();

        // Set test vars
        unsafe {
            env::set_var("WTFPULSE_API_KEY", "test_key_123");
            env::set_var("WTFPULSE_REFRESH_RATE", "99");
        }

        // Load config (mocking file existence by assuming it doesn't exist or we just care about override)
        // Note: This test relies on load() logic. If config file exists, it loads it, then overrides.
        let config = AppConfig::load().unwrap();

        assert_eq!(config.api_key, Some("test_key_123".to_string()));
        assert_eq!(config.refresh_rate_seconds, Some(99));

        // Restore vars
        unsafe {
            if let Some(k) = orig_key {
                env::set_var("WTFPULSE_API_KEY", k);
            } else {
                env::remove_var("WTFPULSE_API_KEY");
            }
            if let Some(r) = orig_rate {
                env::set_var("WTFPULSE_REFRESH_RATE", r);
            } else {
                env::remove_var("WTFPULSE_REFRESH_RATE");
            }
        }
    }
}
