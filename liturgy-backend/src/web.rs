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
    /// Optional path to the frontend project directory (where package.json lives).
    /// If set and `build_frontend` is true, the backend will run the frontend build
    /// before starting and serve files from `<frontend_dir>/dist`.
    pub frontend_dir: Option<String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            frontend_dir: "../liturgy-frontend".to_string().into(),
        }
    }
}

/// Initialize and run the API server
pub async fn run_web_app(config: WebConfig) -> Result<()> {
    backend::start_server(config).await
}
