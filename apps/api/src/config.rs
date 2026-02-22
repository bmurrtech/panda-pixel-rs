use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub rust_log: String,
}

impl Config {
    /// Load configuration from environment variables
    /// Fails fast if any required variable is missing or invalid
    pub fn from_env() -> Result<Self> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("PORT must be a valid u16")?;

        let cors_allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            app_env,
            port,
            cors_allowed_origins,
            rust_log,
        })
    }

    /// Validate that all required configuration is present
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            anyhow::bail!("PORT must be greater than 0");
        }
        Ok(())
    }
}
