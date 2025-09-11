use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use tempfile::tempdir;

use liturgy_backend::web::{run_web_app, WebConfig};

#[tokio::test]
async fn serves_index_from_dist() {
    // Create a temporary frontend directory with dist/index.html
    let dir = tempdir().expect("tempdir");
    let mut dist = PathBuf::from(dir.path());
    dist.push("dist");
    fs::create_dir_all(&dist).expect("create dist");

    let mut index = dist.clone();
    index.push("index.html");
    let expected = "HELLO_FROM_TEST";
    fs::write(&index, expected).expect("write index");

    // Pick an available port by binding to port 0
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let config = WebConfig {
        host: "127.0.0.1".to_string(),
        port,
        calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
        frontend_dir: Some(dir.path().to_str().unwrap().to_string()),
    };

    // Spawn the server in background
    let server_handle = tokio::spawn(async move {
        run_web_app(config).await.expect("server run");
    });

    // Wait for server to become ready and fetch /
    let url = format!("http://127.0.0.1:{}/", port);
    let client = reqwest::Client::new();

    let mut attempts = 0;
    let body = loop {
        if attempts > 50 {
            panic!("server did not become ready");
        }
        match client.get(&url).send().await {
            Ok(resp) => {
                let text = resp.text().await.expect("text");
                break text;
            }
            Err(_) => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        }
    };

    // Shut down server task
    server_handle.abort();

    assert!(body.contains(expected));
}
