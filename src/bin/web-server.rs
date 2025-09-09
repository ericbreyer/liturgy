use liturgy::web::{run_web_app, WebConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = WebConfig::default();
    run_web_app(config).await
}
