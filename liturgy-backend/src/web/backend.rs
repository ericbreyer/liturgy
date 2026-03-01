//! Backend server implementation using Axum
//!
//! Provides REST API endpoints for the liturgical calendar application

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::Request,
    middleware::{Next, from_fn},
    response::{Json, Response},
    routing::{get, post},
};
use calendar_calc::{calender::YearCalendarHandle, GenericCalendarHandle54, GenericCalendarHandle62, GenericCalendarHandleOf};
use delegate::delegate;
use indexmap::IndexMap;
use ordo::{Vespers, VespersOrdo, ordo_repo::OrdoRepo};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{net::TcpListener, sync::RwLock};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use types::{DayDescription, DayRank62, TrivialDayRank};

use crate::web::WebConfig;

type SharedCalendarHandle = Arc<DynCalendarHandle>;

pub enum DynGenericCalendarHandle {
    SixtyTwo(GenericCalendarHandle62),
    FiftyFour(GenericCalendarHandle54),
    OrdinaryForm(GenericCalendarHandleOf),
}

impl DynGenericCalendarHandle {
    delegate! {
        to match self {
            DynGenericCalendarHandle::SixtyTwo(cal) => cal,
            DynGenericCalendarHandle::FiftyFour(cal) => cal,
            DynGenericCalendarHandle::OrdinaryForm(cal) => cal,
        } {
            pub fn name(&self) -> &str;
            pub fn commemoration_interpretation(&self) -> &str;
            pub fn suggest_feast_names(&self, name: &str) -> Vec<(String, f32)>;

            #[expr($.map(|(info, rank)| (
                info.name.to_string(),
                info.date_rule.to_string(),
                rank.to_string(),
                info.color.clone().to_string(),
            )))]
            pub fn get_feast_info(&self, name: &str) -> Result<(String, String, String, String), anyhow::Error>;
        }
    }

}

pub enum DynCalendarHandle {
    Trivial(YearCalendarHandle<TrivialDayRank>),
    SixtyTwo(YearCalendarHandle<DayRank62>),
}

pub enum DynDayHandle {
    Trivial(DayDescription<TrivialDayRank>),
    SixtyTwo(DayDescription<DayRank62>),
}

impl Serialize for DynDayHandle {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            DynDayHandle::Trivial(d) => d.serialize(serializer),
            DynDayHandle::SixtyTwo(d) => d.serialize(serializer),
        }
    }
}

impl VespersOrdo for DynDayHandle {
    fn vespers_ordo(&self, repo: &OrdoRepo) -> Result<Vespers> {
        match self {
            DynDayHandle::Trivial(d) => d.vespers_ordo(repo),
            DynDayHandle::SixtyTwo(d) => d.vespers_ordo(repo),
        }
    }
    fn vespers_ordo_sources(&self, repo: &OrdoRepo) -> Result<Vec<String>> {
        match self {
            DynDayHandle::Trivial(d) => d.vespers_ordo_sources(repo),
            DynDayHandle::SixtyTwo(d) => d.vespers_ordo_sources(repo),
        }
    }
}

impl DynCalendarHandle {
    #[must_use]
    pub fn generate_csv(&self) -> String {
        match self {
            DynCalendarHandle::Trivial(cal) => cal.generate_csv(),
            DynCalendarHandle::SixtyTwo(cal) => cal.generate_csv(),
        }
    }

    /// Return day info as JSON Value regardless of the concrete `DayRank` type.
    #[must_use]
    pub fn get_day_info_json(&self, date: chrono::NaiveDate) -> Option<Value> {
        match self {
            DynCalendarHandle::Trivial(cal) => cal
                .get_day_info(date)
                .map(|d| serde_json::to_value(&d).unwrap_or(Value::Null)),
            DynCalendarHandle::SixtyTwo(cal) => cal
                .get_day_info(date)
                .map(|d| serde_json::to_value(&d).unwrap_or(Value::Null)),
        }
    }

    pub fn get_day_info(&self, date: chrono::NaiveDate) -> Option<DynDayHandle> {
        Some(match self {
            DynCalendarHandle::Trivial(cal) => DynDayHandle::Trivial(cal.get_day_info(date)?),

            DynCalendarHandle::SixtyTwo(cal) => DynDayHandle::SixtyTwo(cal.get_day_info(date)?),
        })
    }
}
/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub gen_calendars: Arc<RwLock<IndexMap<String, DynGenericCalendarHandle>>>,
    pub year_calendars: Arc<RwLock<HashMap<(String, i32), SharedCalendarHandle>>>,
    pub ordo_repo: Arc<RwLock<Option<OrdoRepo>>>,
    pub config: WebConfig,
}

impl AppState {
    #[cfg(test)]
    pub fn new(config: WebConfig) -> Self {
        Self {
            gen_calendars: Arc::new(tokio::sync::RwLock::new(IndexMap::new())),
            year_calendars: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            ordo_repo: Arc::new(tokio::sync::RwLock::new(None)),
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
        gen_calendars: Arc::new(tokio::sync::RwLock::new(IndexMap::new())),
        year_calendars: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        ordo_repo: Arc::new(tokio::sync::RwLock::new(None)),
        config: config.clone(),
    };

    // Load default calendars
    load_default_calendars(&state).await?;

    // Build our application with routes (not used by the simple test server,
    // keep it available for future use).
    let app = create_router(state);

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
    // Create the api router and attach middleware to it (route_layer must be
    // applied after routes)
    let api_router = create_api_router();
    let api_router = if state.config.debug_delay {
        api_router.route_layer(from_fn(delay_middleware))
    } else {
        api_router
    };
    let mut router = Router::<AppState>::new().nest("/api", api_router);

    // If frontend_dir is provided, serve built assets from `<frontend_dir>/dist` at
    // root
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
                println!("✅ Serving frontend from: {}", dist_path.display());
            } else {
                println!(
                    "⚠️  Frontend dist exists but no index.html found at: {}",
                    dist_path.display()
                );
            }
        } else {
            println!(
                "⚠️  Frontend dist directory not found: {}",
                dist_path.display()
            );
        }
    }
    // Add middleware (Trace and CORS). Note: delay middleware is attached to the
    // API router
    router
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::permissive(), /* Allow all origins, methods, and headers for
                                              * development */
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
        .route("/calendars/{name}", get(api_get_calendar))
        .route("/calendars/{name}/year/{year}", get(api_get_year))
        .route(
            "/calendars/{name}/day/{year}/{month}/{day}",
            get(api_get_day),
        )
        .route("/calendars/{name}/search", get(api_search_feasts))
        .route("/calendars/{name}/generate", post(api_generate_calendar))
        .route("/calendars/{name}/stats/{year}", get(api_get_stats))
        .route(
            "/ordo/vespers/{name}/{year}/{month}/{day}",
            get(api_get_ordo_vespers),
        )
        .route(
            "/ordo/vespers/{name}/sources/{year}/{month}/{day}",
            get(api_get_ordo_vespers_sources),
        )
}

async fn api_get_ordo_vespers(
    Path((name, year, month, day)): Path<(String, i32, u32, u32)>,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vespers>> {
    use chrono::NaiveDate;

    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
        return Json(ApiResponse::error(format!(
            "Invalid date: {year}-{month}-{day}"
        )));
    };

    let Some(day_desc) = get_day_info(&state, &name, date).await else {
        return Json(ApiResponse::error(format!(
            "No day info found for date: {date} in calendar '{name}'"
        )));
    };

    // Use cached OrdoRepo if available, otherwise load from configured path
    let read_repo = state.ordo_repo.read().await;

    if let Some(repo) = read_repo.as_ref() {
        match day_desc.vespers_ordo(repo) {
            Ok(vespers) => Json(ApiResponse::success(vespers)),
            Err(e) => Json(ApiResponse::error(format!("Failed to build vespers: {e}"))),
        }
        
    } else {
        drop(read_repo);
        let mut write_repo = state.ordo_repo.write().await;
                
        if write_repo.is_none() {
            let path = state
                .config
                .ordo_rules_dir
                .clone()
                .unwrap_or_else(|| "../ordo/rules".to_string());
            match OrdoRepo::load_from_dir(path) {
                Ok(r) => *write_repo = Some(r),
                Err(e) => {
                    return Json(ApiResponse::error(format!(
                        "Failed to load ordo rules: {e}"
                    )));
                }
            }
        }
        
        match (&day_desc).vespers_ordo(write_repo.as_ref().unwrap()) {
            Ok(vespers) => Json(ApiResponse::success(vespers)),
            Err(e) => Json(ApiResponse::error(format!("Failed to build vespers: {e}"))),
        }
    }
}

// GET /api/ordo/vespers/{name}/sources/{year}/{month}/{day} - Return list of ordo sources
async fn api_get_ordo_vespers_sources(
    Path((name, year, month, day)): Path<(String, i32, u32, u32)>,
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<String>>> {
    use chrono::NaiveDate;

    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
        return Json(ApiResponse::error(format!(
            "Invalid date: {year}-{month}-{day}"
        )));
    };

    let Some(day_desc) = get_day_info(&state, &name, date).await else {
        return Json(ApiResponse::error(format!(
            "No day info found for date: {date} in calendar '{name}'"
        )));
    };

    let read_repo = state.ordo_repo.read().await;
    if let Some(repo) = read_repo.as_ref() {
        match day_desc.vespers_ordo_sources(repo) {
            Ok(sources) => Json(ApiResponse::success(sources)),
            Err(e) => Json(ApiResponse::error(format!(
                "Failed to build vespers sources: {e}"
            ))),
        }
    } else {
        drop(read_repo);
        let mut write_repo = state.ordo_repo.write().await;
        if write_repo.is_none() {
            let path = state
                .config
                .ordo_rules_dir
                .clone()
                .unwrap_or_else(|| "../ordo/rules".to_string());
            match OrdoRepo::load_from_dir(path) {
                Ok(r) => *write_repo = Some(r),
                Err(e) => {
                    return Json(ApiResponse::error(format!(
                        "Failed to load ordo rules: {e}"
                    )));
                }
            }
        }
        match day_desc.vespers_ordo_sources(write_repo.as_ref().unwrap()) {
            Ok(sources) => Json(ApiResponse::success(sources)),
            Err(e) => Json(ApiResponse::error(format!(
                "Failed to build vespers sources: {e}"
            ))),
        }
    }
}

/// Load default calendars from the calendar data directory
async fn load_default_calendars(state: &AppState) -> Result<()> {
    let mut calendars = state.gen_calendars.write().await;

    // Try to load common calendar files
    let calendar_files = [
        ("54", "54.toml", vec![]),
        ("of", "of.toml", vec![]),
        ("ef", "ef.toml", vec![]),
        ("monastic", "62-monastic.toml", vec![]),
        ("of-us", "of.toml", vec!["of-us-extensions.toml"]),
    ];

    for (name, filename, extensions) in calendar_files {
        let path = format!("{}/{}", state.config.calendar_data_dir, filename);
        let extensions_paths: Vec<String> = extensions
            .iter()
            .map(|ext| format!("{}/{}", state.config.calendar_data_dir, ext))
            .collect();
        if std::path::Path::new(&path).exists() {
            let gen_calendar = match name {
                "54" => {
                    GenericCalendarHandle54::load_with_extensions(
                        &path,
                        extensions_paths.iter().collect::<Vec<_>>().as_slice(),
                    ).map(DynGenericCalendarHandle::FiftyFour)
                }
                "ef" | "monastic" => {
                    GenericCalendarHandle62::load_with_extensions(
                        &path,
                        extensions_paths.iter().collect::<Vec<_>>().as_slice(),
                    ).map(DynGenericCalendarHandle::SixtyTwo)
                }
                _ => {
                    GenericCalendarHandleOf::load_with_extensions(
                        &path,
                        extensions_paths.iter().collect::<Vec<_>>().as_slice(),
                    ).map(DynGenericCalendarHandle::OrdinaryForm)
                }
            };

            match gen_calendar {
                Ok(calendar) => {
                    calendars.insert(name.to_string(), calendar);
                    println!("✅ Loaded calendar: {name} from {path}");
                }
                Err(e) => {
                    println!("⚠️  Failed to load calendar {name}: {e}");
                }
            }
        } else {
            println!("📁 Calendar file not found: {path}");
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
) -> Option<Arc<DynCalendarHandle>> {
    let calendars = state.year_calendars.read().await;
    if let Some(calendar) = calendars.get(&(name.to_string(), year)) {
        return Some(calendar.clone());
    }
    drop(calendars); // Release read lock before acquiring write lock

    // Try to generate the year calendar if not found
    let gen_calendars = state.gen_calendars.read().await;
    if let Some(gen_calendar) = gen_calendars.get(name) {
        // Choose which concrete year calendar to create based on calendar type
        let year_calendar: DynCalendarHandle = match gen_calendar {
            DynGenericCalendarHandle::SixtyTwo(cal) => {
                DynCalendarHandle::SixtyTwo(cal.create_year_calendar(year))
            }
            DynGenericCalendarHandle::FiftyFour(cal) => {
                DynCalendarHandle::Trivial(cal.create_year_calendar(year))
            }
            DynGenericCalendarHandle::OrdinaryForm(cal) => {
                DynCalendarHandle::Trivial(cal.create_year_calendar(year))
            }
        };
        drop(gen_calendars);
        let arc_cal = Arc::new(year_calendar);
        let mut calendars = state.year_calendars.write().await;
        calendars.insert((name.to_string(), year), arc_cal.clone());
        return Some(arc_cal);
    }
    None
}

async fn get_day_info (
    state: &AppState,
    name: &str,
    date: chrono::NaiveDate,
) -> Option<DynDayHandle> {
    use chrono::Datelike;
    let year = date.year();
    let year_calendar = get_year_calendar(state, name, year).await?;
    let day = year_calendar.get_day_info(date);
    if day.is_some() {
        return day;
    }
    // try next year
    let next_year_calendar = get_year_calendar(state, name, year + 1).await?;
    next_year_calendar.get_day_info(date)
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

/// GET /api/calendars/{name} - Get calendar details
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
        None => Json(ApiResponse::error(format!("Calendar '{name}' not found"))),
    }
}

#[derive(Serialize)]
struct YearCalendarData {
    calendar_name: String,
    year: i32,
    csv_data: String,
    total_days: usize,
}

/// GET /api/calendars/{name}/year/{year} - Get full year calendar
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
        None => Json(ApiResponse::error(format!("Calendar '{name}' not found"))),
    }
}

#[derive(Serialize)]
struct DayInfo {
    desc: Value,
}

/// GET /api/calendars/{name}/day/{year}/{month}/{day} - Get specific day info
async fn api_get_day(
    Path((name, year, month, day)): Path<(String, i32, u32, u32)>,
    State(state): State<AppState>,
) -> Json<ApiResponse<DayInfo>> {
    use chrono::NaiveDate;

    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
        return Json(ApiResponse::error(format!(
            "Invalid date: {year}-{month}-{day}"
        )));
    };

    get_day_info(&state, &name, date)
        .await
        .and_then(|day_desc| {
            let desc_json = serde_json::to_value(&day_desc).unwrap_or(Value::Null);
            Some(DayInfo { desc: desc_json })
        })
        .map(|data| Json(ApiResponse::success(data)))
        .unwrap_or_else(|| Json(ApiResponse::error(format!(
            "No day info found for date: {date} in calendar '{name}'"
        ))))

    // match get_year_calendar(&state, &name, year).await {
    //     Some(year_calendar) => match year_calendar.as_ref().get_day_info_json(date) {
    //         Some(day_val) => Json(ApiResponse::success(DayInfo { desc: day_val })),
    //         None => match get_year_calendar(&state, &name, year + 1).await {
    //             Some(next_year_calendar) => {
    //                 match next_year_calendar.as_ref().get_day_info_json(date) {
    //                     Some(day_val) => Json(ApiResponse::success(DayInfo { desc: day_val })),
    //                     None => Json(ApiResponse::error(format!("No data for date: {date}"))),
    //                 }
    //             }
    //             None => Json(ApiResponse::error(format!("No data for date: {date}"))),
    //         },
    //     },
    //     None => Json(ApiResponse::error(format!("Calendar '{name}' not found"))),
    // }
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

/// GET /api/calendars/{name}/search?q=query - Search for feasts
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
                    if let Ok((name, date, rank, color)) = calendar.get_feast_info(feast_name) {
                        let result = SearchResult {
                            name: feast_name.clone(),
                            description: name,
                            date: Some(date),
                            rank,
                            score: *score,
                            color,
                        };
                        results.push(result);
                    } else {
                        // Skip if no info found
                    }
                }

                Json(ApiResponse::success(results))
            }
        }
        None => Json(ApiResponse::error(format!("Calendar '{name}' not found"))),
    }
}

#[derive(Deserialize)]
struct GenerateRequest {
    format: Option<String>,
}

/// POST /api/calendars/{name}/generate - Generate calendar data
async fn api_generate_calendar(
    Path(name): Path<String>,
    Query(params): Query<GenerateRequest>,
    State(state): State<AppState>,
) -> Json<ApiResponse<String>> {
    let calendars = state.gen_calendars.read().await;

    match calendars.get(&name) {
        Some(calendar) => {
            // Pick appropriate concrete year calendar based on calendar type
            let year_calendar_csv = match calendar {
                DynGenericCalendarHandle::SixtyTwo(cal) => cal.create_year_calendar(2025).generate_csv(),
                DynGenericCalendarHandle::FiftyFour(cal) => cal.create_year_calendar(2025).generate_csv(),
                DynGenericCalendarHandle::OrdinaryForm(cal) => cal.create_year_calendar(2025).generate_csv(),
            };
            let data = match params.format.as_deref() {
                Some("csv") | None => year_calendar_csv,
                Some("json") => "{}".to_string(), // TODO: Implement JSON format
                Some(format) => {
                    return Json(ApiResponse::error(format!("Unsupported format: {format}")));
                }
            };

            Json(ApiResponse::success(data))
        }
        None => Json(ApiResponse::error(format!("Calendar '{name}' not found"))),
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

/// GET /api/calendars/{name}/stats/{year} - Get calendar statistics
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
        None => Json(ApiResponse::error(format!("Calendar '{name}' not found"))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use insta::{assert_snapshot, with_settings};
    use test_case::{test_case, test_matrix};

    use super::*;

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
        assert_eq!(data.len(), 5); // Expecting 4 calendars (including 1954)
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

    #[tokio::test]
    async fn test_api_get_ordo_vespers() {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ordo_rules_dir: Some("../ordo/rules".to_string()),
            ..Default::default()
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response =
            api_get_ordo_vespers(Path(("ef".to_string(), 2024, 12, 25)), State(state)).await;
        assert!(response.0.success, "Response error: {:?}", response.0.error);
        let data = response.0.data.unwrap();
        with_settings!({snapshot_suffix => "_of_2023_12_25"}, {
            assert_snapshot!(data);
        });
    }

    #[tokio::test]
    async fn test_api_get_day() {
        let config = WebConfig {
            host: "localhost".to_string(),
            port: 3000,
            calendar_data_dir: "../calendar_calc/calendar_data".to_string(),
            ..Default::default()
        };
        let state = AppState::new(config);
        load_default_calendars(&state).await.unwrap();
        let response = api_get_day(Path(("ef".to_string(), 2024, 12, 25)), State(state)).await;
        assert!(response.0.success, "Response error: {:?}", response.0.error);
        let data = response.0.data.unwrap();
        with_settings!({snapshot_suffix => "_ef_2024_12_25"}, {
            assert_snapshot!(serde_json::to_string_pretty(&data).unwrap());
        });
    }
}
