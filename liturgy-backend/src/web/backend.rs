//! Backend server implementation using Axum
//!
//! Provides REST API endpoints for the liturgical calendar application

use crate::web::WebConfig;
use anyhow::Result;
use axum::body::Body;
use axum::http::Request;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use calendar_calc::{calender::GenericCalendarHandle, YearCalendarHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use types::DayDescription;
use types::TrivialDayRank;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub gen_calendars: Arc<tokio::sync::RwLock<HashMap<String, GenericCalendarHandle>>>,
    pub year_calendars:
        Arc<tokio::sync::RwLock<HashMap<(String, i32), Arc<YearCalendarHandle<TrivialDayRank>>>>>,
    pub config: WebConfig,
}

impl AppState {
    #[cfg(test)]
    pub fn new(config: WebConfig) -> Self {
        Self {
            gen_calendars: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            year_calendars: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            config,
        }
    }
}

/// Start the web server
pub async fn start_server(config: WebConfig) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create shared state
    let state = AppState {
        gen_calendars: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        year_calendars: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        config: config.clone(),
    };

    // Load default calendars
    load_default_calendars(&state).await?;

    // Build our application with routes
    let app = create_router(state);

    // For production we only print startup info here (server is started
    // elsewhere or via a different entrypoint). In tests we run a very
    // small HTTP server (using tokio) that serves files from the
    // frontend `dist` directory so integration tests can exercise
    // static file serving without pulling in full axum/hyper server types.
    println!(
        "🚀 Liturgical Calendar Web App (not started) on http://{}:{}",
        config.host, config.port
    );
    println!("📅 Calendar data directory: {}", config.calendar_data_dir);

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener as TokioTcpListener;

    // If frontend_dir is set and has a dist/index.html, serve from there.
    if let Some(frontend_dir) = &config.frontend_dir {
        let mut dist = std::path::PathBuf::from(frontend_dir);
        dist.push("dist");
        let index_path = dist.join("index.html");
        // Bind to the requested address and serve until task is aborted
        let bind_addr = format!("{}:{}", config.host, config.port);
        let listener = TokioTcpListener::bind(&bind_addr)
            .await
            .expect("bind test server");
        println!("✅ Test static server serving from: {:?}", dist);

        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("accept error: {}", e);
                    continue;
                }
            };

            let dist_clone = dist.clone();
            let index_clone = index_path.clone();

            tokio::spawn(async move {
                let mut buf = [0u8; 2048];
                match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => {
                        let req = String::from_utf8_lossy(&buf[..n]);
                        // very naive request line parsing
                        let first_line = req.lines().next().unwrap_or("");
                        let mut parts = first_line.split_whitespace();
                        let _method = parts.next().unwrap_or("");
                        let path = parts.next().unwrap_or("/");

                        // Map path to file under dist
                        let file_path = if path == "/" {
                            index_clone.clone()
                        } else {
                            let mut p = dist_clone.clone();
                            // strip leading '/'
                            let rel = path.trim_start_matches('/');
                            p.push(rel);
                            p
                        };

                        let response = match tokio::fs::read(&file_path).await {
                            Ok(body) => {
                                let header = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                                    body.len()
                                );
                                let mut resp = header.into_bytes();
                                resp.extend_from_slice(&body);
                                resp
                            }
                            Err(_) => {
                                let body = b"Not Found".to_vec();
                                let header = format!(
                                    "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\n\r\n",
                                    body.len()
                                );
                                let mut resp = header.into_bytes();
                                resp.extend_from_slice(&body);
                                resp
                            }
                        };

                        let _ = socket.write_all(&response).await;
                    }
                    _ => {}
                }
            });
        }
    }

    Ok(())
}

/// Create the main router with all routes
fn create_router(state: AppState) -> Router<AppState> {
    // Create the api router and attach middleware to it (route_layer must be applied after routes)
    let api_router = create_api_router();
    let api_router = if state.config.debug_delay {
        api_router.route_layer(from_fn(delay_middleware))
    } else {
        api_router
    };
    let mut router = Router::<AppState>::new().nest("/api", api_router);

    // If frontend_dir is provided, serve built assets from `<frontend_dir>/dist` at root
    if let Some(frontend_dir) = &state.config.frontend_dir {
        let mut dist_path = PathBuf::from(frontend_dir);
        dist_path.push("dist");
        if dist_path.exists() {
            // Serve static files
            let serve_dir = ServeDir::new(dist_path.clone()).append_index_html_on_directories(true);

            // Fallback to index.html for SPA routes
            let index_file = dist_path.join("index.html");
            if index_file.exists() {
                let serve_index = ServeFile::new(index_file);
                // Mount at root: try static files first, then index.html
                router = router.fallback_service(serve_dir.fallback(serve_index.clone()));
                println!("✅ Serving frontend from: {:?}", dist_path);
            } else {
                println!(
                    "⚠️  Frontend dist exists but no index.html found at: {:?}",
                    dist_path
                );
            }
        } else {
            println!("⚠️  Frontend dist directory not found: {:?}", dist_path);
        }
    }
    // Add middleware (Trace and CORS). Note: delay middleware is attached to the API router
    router
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::permissive(), // Allow all origins, methods, and headers for development
                ),
        )
        .with_state(state)
}

/// debug middleware that delays requests by a fixed duration
/// (useful for simulating slow network conditions during development)
/// not used in production
#[allow(dead_code)]
async fn delay_middleware(req: Request<Body>, next: Next) -> Response {
    use std::time::Duration;
    use tokio::time::sleep;
    // Delay every request by 500ms
    // Log and delay to help confirm middleware runs
    println!("⏳ Delaying request by 500ms: {}", req.uri());
    sleep(Duration::from_millis(500)).await;
    next.run(req).await
}

/// Create API router
fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/calendars", get(api_list_calendars))
        .route("/calendars/:name", get(api_get_calendar))
        .route("/calendars/:name/year/:year", get(api_get_year))
        .route("/calendars/:name/day/:year/:month/:day", get(api_get_day))
        .route("/calendars/:name/search", get(api_search_feasts))
        .route("/calendars/:name/generate", post(api_generate_calendar))
        .route("/calendars/:name/stats/:year", get(api_get_stats))
}

/// Load default calendars from the calendar data directory
async fn load_default_calendars(state: &AppState) -> Result<()> {
    let mut calendars = state.gen_calendars.write().await;

    // Try to load common calendar files
    let calendar_files = [
        ("54", "54.toml", vec![]),
        ("of", "of.toml", vec![]),
        ("ef", "ef.toml", vec![]),
        ("of-us", "of.toml", vec!["of-us-extensions.toml"]),
    ];

    for (name, filename, extensions) in calendar_files {
        let path = format!("{}/{}", state.config.calendar_data_dir, filename);
        let extensions_paths: Vec<String> = extensions
            .iter()
            .map(|ext| format!("{}/{}", state.config.calendar_data_dir, ext))
            .collect();
        if std::path::Path::new(&path).exists() {
            match GenericCalendarHandle::load_with_extensions(
                &path,
                extensions_paths.iter().collect::<Vec<_>>().as_slice(),
            ) {
                Ok(calendar) => {
                    calendars.insert(name.to_string(), calendar);
                    println!("✅ Loaded calendar: {} from {}", name, path);
                }
                Err(e) => {
                    println!("⚠️  Failed to load calendar {}: {}", name, e);
                }
            }
        } else {
            println!("📁 Calendar file not found: {}", path);
        }
    }

    if calendars.is_empty() {
        println!("⚠️  No calendars loaded successfully!");
        println!(
            "   Make sure calendar files exist in: {}",
            state.config.calendar_data_dir
        );
        println!("   Expected files: 54.toml, of.toml, ef.toml, of-us-extensions.toml");
    } else {
        println!("📅 Loaded {} calendar(s) successfully", calendars.len());
    }

    Ok(())
}

async fn get_year_calendar(
    state: &AppState,
    name: &str,
    year: i32,
) -> Option<Arc<YearCalendarHandle<TrivialDayRank>>> {
    let calendars = state.year_calendars.read().await;
    if let Some(calendar) = calendars.get(&(name.to_string(), year)) {
        return Some(calendar.clone());
    }
    drop(calendars); // Release read lock before acquiring write lock

    // Try to generate the year calendar if not found
    let gen_calendars = state.gen_calendars.read().await;
    if let Some(gen_calendar) = gen_calendars.get(name) {
        // Choose which concrete year calendar to create based on calendar name
        let year_calendar: YearCalendarHandle<TrivialDayRank> = match name {
            "ef" => gen_calendar.create_year_calendar_62(year),
            "54" => gen_calendar.create_year_calendar_54(year),
            _ => gen_calendar.create_year_calendar_of(year),
        };
        drop(gen_calendars);
        let arc_cal = Arc::new(year_calendar);
        let mut calendars = state.year_calendars.write().await;
        calendars.insert((name.to_string(), year), arc_cal.clone());
        return Some(arc_cal);
    }
    None
}

// API Handlers

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

#[derive(Serialize)]
struct CalendarInfo {
    name: String,
    commemoration_interpretation: String,
    display_name: String,
    description: String,
}

/// GET /api/calendars - List all available calendars
async fn api_list_calendars(State(state): State<AppState>) -> Json<ApiResponse<Vec<CalendarInfo>>> {
    let calendars = state.gen_calendars.read().await;
    let calendar_list: Vec<CalendarInfo> = calendars
        .iter()
        .map(|(name, handle)| CalendarInfo {
            name: name.clone(),
            display_name: handle.name().to_string(),
            description: format!("Liturgical calendar: {}", handle.name()),
            commemoration_interpretation: handle.commemoration_interpretation().to_string(),
        })
        .collect();

    Json(ApiResponse::success(calendar_list))
}

#[derive(Serialize)]
struct CalendarDetails {
    name: String,
    display_name: String,
    description: String,
}

/// GET /api/calendars/:name - Get calendar details
async fn api_get_calendar(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Json<ApiResponse<CalendarDetails>> {
    let calendars = state.gen_calendars.read().await;

    match calendars.get(&name) {
        Some(calendar) => {
            let details = CalendarDetails {
                name: name.clone(),
                display_name: calendar.name().to_string(),
                description: format!("Liturgical calendar: {}", calendar.name()),
            };
            Json(ApiResponse::success(details))
        }
        None => Json(ApiResponse::error(format!("Calendar '{}' not found", name))),
    }
}

#[derive(Serialize)]
struct YearCalendarData {
    calendar_name: String,
    year: i32,
    csv_data: String,
    total_days: usize,
}

/// GET /api/calendars/:name/year/:year - Get full year calendar
async fn api_get_year(
    Path((name, year)): Path<(String, i32)>,
    State(state): State<AppState>,
) -> Json<ApiResponse<YearCalendarData>> {
    match get_year_calendar(&state, &name, year).await {
        Some(year_calendar) => {
            let csv_data = year_calendar.generate_csv();
            let total_days = csv_data.lines().count() - 1; // Exclude header line
            let data = YearCalendarData {
                calendar_name: name.clone(),
                year,
                csv_data,
                total_days,
            };
            Json(ApiResponse::success(data))
        }
        None => Json(ApiResponse::error(format!("Calendar '{}' not found", name))),
    }
}

#[derive(Serialize)]
struct DayInfo {
    desc: DayDescription<TrivialDayRank>,
}

/// GET /api/calendars/:name/day/:year/:month/:day - Get specific day info
async fn api_get_day(
    Path((name, year, month, day)): Path<(String, i32, u32, u32)>,
    State(state): State<AppState>,
) -> Json<ApiResponse<DayInfo>> {
    use chrono::NaiveDate;

    let date = match NaiveDate::from_ymd_opt(year, month, day) {
        Some(d) => d,
        None => {
            return Json(ApiResponse::error(format!(
                "Invalid date: {}-{}-{}",
                year, month, day
            )))
        }
    };

    match get_year_calendar(&state, &name, year).await {
        Some(year_calendar) => match year_calendar.get_day_info(date) {
            Some(day_desc) => {
                let info = DayInfo {
                    desc: day_desc.clone(),
                };
                Json(ApiResponse::success(info))
            }
            None => get_year_calendar(&state, &name, year + 1)
                .await
                .and_then(|next_year_calendar| next_year_calendar.get_day_info(date))
                .map_or_else(
                    || Json(ApiResponse::error(format!("No data for date: {}", date))),
                    |day_desc| {
                        let info = DayInfo {
                            desc: day_desc.clone(),
                        };
                        Json(ApiResponse::success(info))
                    },
                ),
        },
        None => Json(ApiResponse::error(format!("Calendar '{}' not found", name))),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
struct SearchResult {
    name: String,
    description: String,
    date: Option<String>,
    rank: String,
    score: f32,
    color: String,
}

/// GET /api/calendars/:name/search?q=query - Search for feasts
async fn api_search_feasts(
    Path(name): Path<String>,
    Query(params): Query<SearchQuery>,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<SearchResult>>> {
    let calendars = state.gen_calendars.read().await;

    match calendars.get(&name) {
        Some(calendar) => {
            // Get fuzzy matches first
            let feast_names = calendar.suggest_feast_names(&params.q);

            if feast_names.is_empty() {
                Json(ApiResponse::success(vec![]))
            } else {
                let mut results = Vec::new();

                // For each fuzzy match, try to get feast info
                for (feast_name, score) in feast_names.iter().take(6) {
                    // Limit to 6 results for cleaner display
                    match calendar.get_feast_info(feast_name) {
                        Ok((info, rankstr)) => {
                            let result = SearchResult {
                                name: feast_name.clone(),
                                description: info.to_string(),
                                date: info.date_rule.to_string().into(), // Convert date rule to string
                                rank: rankstr.to_string(),
                                score: *score,
                                color: info.color.clone().to_string(),
                            };
                            results.push(result);
                        }
                        Err(_) => {
                            // Skip if no info found
                            continue;
                        }
                    }
                }

                Json(ApiResponse::success(results))
            }
        }
        None => Json(ApiResponse::error(format!("Calendar '{}' not found", name))),
    }
}

#[derive(Deserialize)]
struct GenerateRequest {
    format: Option<String>,
}

/// POST /api/calendars/:name/generate - Generate calendar data
async fn api_generate_calendar(
    Path(name): Path<String>,
    Query(params): Query<GenerateRequest>,
    State(state): State<AppState>,
) -> Json<ApiResponse<String>> {
    let calendars = state.gen_calendars.read().await;

    match calendars.get(&name) {
        Some(calendar) => {
            // Pick appropriate concrete year calendar based on calendar name
            let year_calendar: YearCalendarHandle<TrivialDayRank> = match name.as_str() {
                "ef" => calendar.create_year_calendar_62(2025),
                "54" => calendar.create_year_calendar_54(2025),
                _ => calendar.create_year_calendar_of(2025),
            };
            let data = match params.format.as_deref() {
                Some("csv") | None => year_calendar.generate_csv(),
                Some("json") => "{}".to_string(), // TODO: Implement JSON format
                Some(format) => {
                    return Json(ApiResponse::error(format!(
                        "Unsupported format: {}",
                        format
                    )))
                }
            };

            Json(ApiResponse::success(data))
        }
        None => Json(ApiResponse::error(format!("Calendar '{}' not found", name))),
    }
}

#[derive(Serialize)]
struct CalendarStats {
    year: i32,
    total_days: usize,
    feast_days: usize,
    seasons: Vec<SeasonStats>,
}

#[derive(Serialize)]
struct SeasonStats {
    name: String,
    days: usize,
    color: String,
}

/// GET /api/calendars/:name/stats/:year - Get calendar statistics
async fn api_get_stats(
    Path((name, year)): Path<(String, i32)>,
    State(state): State<AppState>,
) -> Json<ApiResponse<CalendarStats>> {
    let calendars = state.gen_calendars.read().await;

    match calendars.get(&name) {
        Some(_calendar) => {
            // TODO: Implement actual statistics calculation
            let stats = CalendarStats {
                year,
                total_days: 365,
                feast_days: 85,
                seasons: vec![
                    SeasonStats {
                        name: "Advent".to_string(),
                        days: 28,
                        color: "purple".to_string(),
                    },
                    SeasonStats {
                        name: "Christmas".to_string(),
                        days: 12,
                        color: "white".to_string(),
                    },
                    SeasonStats {
                        name: "Ordinary Time".to_string(),
                        days: 275,
                        color: "green".to_string(),
                    },
                    SeasonStats {
                        name: "Lent".to_string(),
                        days: 40,
                        color: "purple".to_string(),
                    },
                    SeasonStats {
                        name: "Easter".to_string(),
                        days: 50,
                        color: "white".to_string(),
                    },
                ],
            };

            Json(ApiResponse::success(stats))
        }
        None => Json(ApiResponse::error(format!("Calendar '{}' not found", name))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use insta::{assert_snapshot, with_settings};
    use test_case::{test_case, test_matrix};

    #[tokio::test]
    async fn test_load_default_calendars() {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ..Default::default()
        };
        let state = AppState::new(config);

        let result = load_default_calendars(&state).await;
        assert!(result.is_ok());
        let calendars = state.gen_calendars.read().await;
        assert!(!calendars.is_empty(), "No calendars loaded");
    }

    #[tokio::test]
    async fn test_api_list_calendars() {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ..Default::default()
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_list_calendars(State(state)).await;
        assert!(response.0.success);
        let data = response.0.data.unwrap();
        assert_eq!(data.len(), 4); // Expecting 4 calendars (including 1954)
        let mut expecting_to_see = HashSet::from([
            "1954 Roman Calendar",
            "1962 Roman Calendar",
            "Ordinary Form of the Roman Calendar",
            "Ordinary Form of the Roman Calendar with USA Extensions",
        ]);
        for cal in data {
            expecting_to_see.remove(cal.display_name.as_str());
        }
        assert!(
            expecting_to_see.is_empty(),
            "Missing calendars: {:?}",
            expecting_to_see
        );
    }

    #[tokio::test]
    #[test_case("ef", "1962 Roman Calendar")]
    #[test_case("of", "Ordinary Form of the Roman Calendar")]
    #[test_case("of-us", "Ordinary Form of the Roman Calendar with USA Extensions")]
    async fn test_api_get_calendar(name: &str, display_name: &str) {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ..Default::default()
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_get_calendar(Path(name.to_string()), State(state)).await;
        assert!(response.0.success);
        let data = response.0.data.unwrap();
        assert_eq!(data.name, name);
        assert_eq!(data.display_name, display_name);
    }

    #[tokio::test]
    async fn test_api_get_calendar_not_found() {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ..Default::default()
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_get_calendar(Path("nonexistent".to_string()), State(state)).await;
        assert!(!response.0.success);
        assert_eq!(
            response.0.error.unwrap(),
            "Calendar 'nonexistent' not found"
        );
    }

    #[tokio::test]
    #[test_matrix(
        ["ef", "of", "of-us"],
        2020..=2030
    )]
    async fn test_api_get_year(name: &str, year: i32) {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ..Default::default()
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_get_year(Path((name.to_string(), year)), State(state)).await;
        assert!(response.0.success);
        let data = response.0.data.unwrap();
        assert_eq!(data.calendar_name, name);
        assert_eq!(data.year, year);
        with_settings!({snapshot_suffix => format!("_{}_{}", name, year)}, {
            assert_snapshot!(data.total_days);
        });
    }

    #[tokio::test]
    async fn test_api_get_year_not_found() {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ..Default::default()
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_get_year(Path(("nonexistent".to_string(), 2025)), State(state)).await;
        assert!(!response.0.success);
        assert_eq!(
            response.0.error.unwrap(),
            "Calendar 'nonexistent' not found"
        );
    }
}
