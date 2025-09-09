//! Backend server implementation using Axum
//!
//! Provides REST API endpoints for the liturgical calendar application

use crate::calender::year_calendar::DayDescription;
use crate::{YearCalendarHandle, calender::GenericCalendarHandle};
use crate::web::WebConfig;
use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub gen_calendars: Arc<tokio::sync::RwLock<HashMap<String, GenericCalendarHandle>>>,
    pub year_calendars: Arc<tokio::sync::RwLock<HashMap<(String, i32), YearCalendarHandle>>>,
    pub config: WebConfig,
}

impl AppState {
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

    // Create listener
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    println!(
        "🚀 Liturgical Calendar Web App starting on http://{}:{}",
        config.host, config.port
    );
    println!("📅 Calendar data directory: {}", config.calendar_data_dir);

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the main router with all routes
fn create_router(state: AppState) -> Router {
    Router::new()
        // API routes only - no frontend serving
        .nest("/api", create_api_router())
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::permissive(), // Allow all origins, methods, and headers for development
                ),
        )
        .with_state(state)
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
                extensions_paths
                    .iter()
                    .map(|s| s)
                    .collect::<Vec<_>>()
                    .as_slice(),
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
) -> Option<YearCalendarHandle> {
    let calendars = state.year_calendars.read().await;
    if let Some(calendar) = calendars.get(&(name.to_string(), year)) {
        return Some(calendar.clone());
    }
    drop(calendars); // Release read lock before acquiring write lock

    // Try to generate the year calendar if not found
    let gen_calendars = state.gen_calendars.read().await;
    if let Some(gen_calendar) = gen_calendars.get(name) {
        let year_calendar = gen_calendar.create_year_calendar(year);
        drop(gen_calendars);
        let mut calendars = state.year_calendars.write().await;
        calendars.insert((name.to_string(), year), year_calendar.clone());
        return Some(year_calendar);
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
    desc: DayDescription,
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
        Some(year_calendar) => {
            match year_calendar.get_day_info(date) {
                Some(day_desc) => {
                    let info = DayInfo { desc: day_desc.clone() };
                    Json(ApiResponse::success(info))
                }
                None => get_year_calendar(&state, &name, year + 1).await .and_then(|next_year_calendar| next_year_calendar.get_day_info(date)).map_or_else(
                    || Json(ApiResponse::error(format!("No data for date: {}", date))),
                    |day_desc| {
                        let info = DayInfo { desc: day_desc.clone() };
                        Json(ApiResponse::success(info))
                    },
                ),
            }
        }
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
                for (feast_name, score) in feast_names.iter().take(6) { // Limit to 6 results for cleaner display
                    match calendar.get_feast_info(feast_name) {
                        Ok((info, rankstr)) => {
                            let result = SearchResult {
                                name: feast_name.clone(),
                                description: info.to_string(),
                                date: info.date_rule.to_string().into(), // Convert date rule to string
                                rank: rankstr,
                                score: *score,
                                color: info.color.clone(),
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
            let year_calendar = calendar.create_year_calendar(2025); // TODO: Make year configurable
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
            calendar_data_dir: "calendar_data".to_string(),
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
            calendar_data_dir: "calendar_data".to_string(),
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_list_calendars(State(state)).await;
        assert!(response.0.success);
        let data = response.0.data.unwrap();
        assert_eq!(data.len(), 4); // Expecting 4 calendars (including 1954)
        let mut expecting_to_see = HashSet::from(["1954 Roman Calendar", "1962 Roman Calendar", "Ordinary Form of the Roman Calendar", "Ordinary Form of the Roman Calendar with USA Extensions"]);
        for cal in data {
            expecting_to_see.remove(cal.display_name.as_str());
        }
        assert!(expecting_to_see.is_empty(), "Missing calendars: {:?}", expecting_to_see);
    }

    #[tokio::test]
    #[test_case("ef", "1962 Roman Calendar")]
    #[test_case("of", "Ordinary Form of the Roman Calendar")]
    #[test_case("of-us", "Ordinary Form of the Roman Calendar with USA Extensions")]
    async fn test_api_get_calendar(name: &str, display_name: &str) {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "calendar_data".to_string(),
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
            calendar_data_dir: "calendar_data".to_string(),
        };
       let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_get_calendar(Path("nonexistent".to_string()), State(state)).await;
        assert!(!response.0.success);
        assert_eq!(response.0.error.unwrap(), "Calendar 'nonexistent' not found");
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
            calendar_data_dir: "calendar_data".to_string(),
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
            calendar_data_dir: "calendar_data".to_string(),
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_get_year(Path(("nonexistent".to_string(), 2025)), State(state)).await;
        assert!(!response.0.success);
        assert_eq!(response.0.error.unwrap(), "Calendar 'nonexistent' not found");
    }

    

}
