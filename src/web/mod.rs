//! Web API module for liturgical calendars
//!
//! This module provides a REST API backend with Axum for liturgical calendar data

pub mod backend;

use anyhow::Result;

/// API server configuration
#[derive(Debug, Clone)]
pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub calendar_data_dir: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            calendar_data_dir: "calendar_data".to_string(),
        }
    }
}

/// Initialize and run the API server
pub async fn run_web_app(config: WebConfig) -> Result<()> {
    backend::start_server(config).await
}
