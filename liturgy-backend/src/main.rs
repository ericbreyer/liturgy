mod web;

use crate::web::{run_web_app, WebConfig};
use anyhow::Result;
use clap::Parser;

/// CLI options for the web server
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Directory containing calendar data (toml files)
    #[arg(long, default_value = "../calendar_calc/calendar_data")]
    calendar_data_dir: String,

    /// Optional frontend project directory (where package.json lives)
    #[arg(long)]
    frontend_dir: Option<String>,

    /// Host/IP to bind the server to (use 0.0.0.0 in containers)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = WebConfig {
        host: args.host,
        port: args.port,
        calendar_data_dir: args.calendar_data_dir,
        frontend_dir: args.frontend_dir,
    };

    run_web_app(config).await
}
