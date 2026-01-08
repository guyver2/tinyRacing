use crate::auth::{authenticate_user, delete_token, store_token, AuthError};
use crate::commands;
use crate::database::queries as tdb;
use crate::database::{
    CreatePlayerRequest, CreateRaceRequest, CreateTeamRequest, LoginRequest, LoginResponse,
    RegisterRequest,
};
use crate::models::car::CarStatus;
use crate::models::driver_avatar::generate_driver_avatar;
use crate::models::race::{RaceRunState, RaceState, MAX_PARTICIPANTS};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use futures_util::stream;
use multer::Multipart;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::PgPool;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path as StdPath;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use uuid::Uuid;

// Type alias for the shared state used across threads/tasks
type SharedRaceState = Arc<Mutex<RaceState>>;

// Type alias for the API Result with standard error response
type ApiResult<T> = Result<T, ApiError>;

// Type for live update events
#[derive(Clone, Debug, Serialize)]
enum LiveEvent {
    RaceUpdate(RaceUpdateEvent),
    CarUpdate(CarUpdateEvent),
    PitStop(PitStopEvent),
    RaceFinished(RaceFinishedEvent),
}

// Event type definitions
#[derive(Clone, Debug, Serialize)]
struct RaceUpdateEvent {
    run_state: String,
    elapsed_time: f64,
    lap_count: u32,
}

#[derive(Clone, Debug, Serialize)]
struct CarUpdateEvent {
    car_number: u32,
    position: u32,
    lap: u32,
    status: String,
}

#[derive(Clone, Debug, Serialize)]
struct PitStopEvent {
    car_number: u32,
    tires: Option<String>,
    refuel: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
struct RaceFinishedEvent {
    winner: u32,
    total_time: f32,
}

// API Error type
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    Unauthorized(String),
    Forbidden(String),
}

// API Response and Error implementations
#[derive(Serialize)]
struct ApiResponse<T> {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

// Custom deserializer for refuel that accepts both int and float (as number or string)
fn deserialize_refuel<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    match opt {
        Some(value) => {
            match value {
                serde_json::Value::Number(num) => {
                    // Try to parse as f64 first (handles both int and float)
                    if let Some(f) = num.as_f64() {
                        Ok(Some(f as f32))
                    } else {
                        Err(Error::custom("Invalid number format for refuel"))
                    }
                }
                serde_json::Value::String(s) => {
                    // Try to parse string as a number
                    match s.parse::<f32>() {
                        Ok(f) if f >= 0.0 && f <= 100.0 => Ok(Some(f)),
                        Ok(_) => Err(Error::custom("refuel must be between 0 and 100")),
                        Err(_) => Err(Error::custom("refuel must be a number (int or float)")),
                    }
                }
                _ => Err(Error::custom("refuel must be a number (int or float)")),
            }
        }
        _none => Ok(None),
    }
}

// Request DTOs
#[derive(Deserialize)]
struct DrivingStyleRequest {
    style: String,
}

#[derive(Deserialize)]
struct PitStopRequest {
    #[serde(default)]
    tires: Option<String>,
    #[serde(deserialize_with = "deserialize_refuel", default)]
    refuel: Option<f32>,
    #[serde(default)]
    cancel: Option<bool>,
}

#[derive(Deserialize)]
struct TeamQueryParams {
    player_id: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_offset")]
    offset: i64,
}

#[derive(Deserialize)]
struct PaginationParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_offset")]
    offset: i64,
}

#[derive(Deserialize)]
struct RaceQueryParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_offset")]
    offset: i64,
    status: Option<String>, // Filter by status: "upcoming" or "done"
}

fn default_limit() -> i64 {
    crate::constants::DEFAULT_PAGE_SIZE
}

fn default_offset() -> i64 {
    0
}

// Implementation of response conversion for ApiError
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            ApiError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
        };

        let body = Json(ApiResponse::<()> {
            status: "error".to_string(),
            message: Some(error_message),
            data: None,
        });

        (status, body).into_response()
    }
}

// Success response builder
fn success<T>(data: Option<T>, message: Option<String>) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        status: "success".to_string(),
        message,
        data,
    })
}

// Driver response with avatar URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverResponse {
    #[serde(flatten)]
    pub driver: crate::database::DriverDb,
    pub avatar_url: String,
}

/// Generates a deterministic filename for a driver avatar based on name, gender, and DoB
fn get_avatar_filename(
    first_name: &str,
    last_name: &str,
    gender: &str,
    date_of_birth: &chrono::NaiveDate,
) -> String {
    let mut hasher = DefaultHasher::new();
    let full_name = format!("{} {}", first_name, last_name);
    full_name.hash(&mut hasher);
    gender.hash(&mut hasher);
    date_of_birth.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:x}.svg", hash)
}

/// Ensures the avatar exists for a driver, generating it if necessary
/// Returns the URL path to the avatar
async fn ensure_driver_avatar(driver: &crate::database::DriverDb) -> Result<String, ApiError> {
    let filename = get_avatar_filename(
        &driver.first_name,
        &driver.last_name,
        &driver.gender,
        &driver.date_of_birth,
    );
    let avatar_dir = StdPath::new("assets/avatars/drivers");
    let avatar_path = avatar_dir.join(&filename);

    // If avatar already exists, just return the URL
    if fs::metadata(&avatar_path).await.is_ok() {
        return Ok(format!("/assets/avatars/drivers/{}", filename));
    }

    // Generate the avatar
    let full_name = format!("{} {}", driver.first_name, driver.last_name);
    let svg_content = generate_driver_avatar(&full_name, &driver.gender, &driver.date_of_birth);

    // Ensure directory exists
    fs::create_dir_all(avatar_dir).await.map_err(|e| {
        ApiError::InternalError(format!("Failed to create avatar directory: {}", e))
    })?;

    // Write the SVG file
    fs::write(&avatar_path, svg_content)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to write avatar file: {}", e)))?;

    Ok(format!("/assets/avatars/drivers/{}", filename))
}

/// Converts a DriverDb to DriverResponse with avatar URL
async fn driver_to_response(driver: crate::database::DriverDb) -> Result<DriverResponse, ApiError> {
    let avatar_url = ensure_driver_avatar(&driver).await?;
    Ok(DriverResponse { driver, avatar_url })
}

// Create the API Router
pub fn create_api_router(race_state: SharedRaceState, db_pool: Option<PgPool>) -> Router {
    // Create a channel for live updates
    let (tx, _) = broadcast::channel::<LiveEvent>(100);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Authentication routes (public)
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/logout", post(logout))
        // DB content routes
        .route("/teams", get(get_teams))
        .route("/teams/my", get(get_my_team))
        .route("/drivers", get(get_drivers))
        .route("/drivers/unassigned", get(get_unassigned_drivers))
        .route("/cars", get(get_cars))
        .route("/cars/unassigned", get(get_unassigned_cars))
        .route("/tracks", get(get_tracks))
        .route("/players", get(get_players))
        .route("/races", get(get_races))
        .route("/races", post(create_race_handler))
        .route("/races/{race_id}", get(get_race))
        .route("/races/{race_id}/register", post(register_for_race))
        .route("/races/{race_id}/register", delete(unregister_from_race))
        .route(
            "/races/{race_id}/registrations",
            get(get_race_registrations),
        )
        .route("/races/{race_id}/start-now", post(start_race_now))
        .route("/races/{race_id}/results", get(get_race_results))
        .route("/teams/{team_id}", get(get_team))
        .route("/teams/{team_id}/drivers", get(get_team_drivers))
        .route("/teams/{team_id}/cars", get(get_team_cars))
        .route(
            "/teams/{team_id}/registrations",
            get(get_team_registrations),
        )
        .route("/drivers/{driver_id}", get(get_driver))
        .route(
            "/drivers/{driver_id}/race-results",
            get(get_driver_race_results),
        )
        .route("/drivers/{driver_id}/buy", post(buy_driver))
        .route("/drivers/{driver_id}/assign-car", post(assign_driver_car))
        .route(
            "/drivers/{driver_id}/level-up",
            post(level_up_driver_handler),
        )
        .route("/cars/{car_id}", get(get_car))
        .route("/cars/{car_id}/buy", post(buy_car))
        .route("/tracks/{track_id}", get(get_track))
        .route("/players/{player_id}", get(get_player))
        // create routes
        .route("/teams", post(create_team_handler))
        // Race control routes
        .route("/race/{race_id}", get(get_race_status))
        .route("/race/{race_id}/start", post(start_race))
        .route("/race/{race_id}/pause", post(pause_race))
        .route("/race/{race_id}/stop", post(stop_race))
        // Car control routes
        .route("/race/{race_id}/car/{car_number}", get(get_car_status))
        .route(
            "/race/{race_id}/car/{car_number}/driving-style",
            put(set_driving_style),
        )
        // Pit stop routes
        .route(
            "/race/{race_id}/car/{car_number}/pit",
            post(request_pit_stop),
        )
        // Static file serving for assets
        .nest_service("/assets", ServeDir::new("assets"))
        // Apply CORS middleware
        .layer(cors)
        // Share state across handlers
        .with_state(AppState {
            race_state,
            live_tx: tx,
            db_pool,
        })
}

// Application state to be shared across handlers
#[derive(Clone)]
struct AppState {
    race_state: SharedRaceState,
    live_tx: broadcast::Sender<LiveEvent>,
    db_pool: Option<PgPool>,
}

// Route handler implementations

// ========== Database Content Getters ==========

// Get all teams (optionally filtered by player_id)
async fn get_teams(
    State(state): State<AppState>,
    Query(params): Query<TeamQueryParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::TeamDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    let teams = if let Some(player_id_str) = params.player_id {
        let player_id = Uuid::parse_str(&player_id_str).map_err(|_| {
            ApiError::BadRequest(format!("Invalid player ID format: {}", player_id_str))
        })?;
        tdb::list_teams_by_player(pool, player_id, params.limit, params.offset)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to fetch teams: {}", e)))?
    } else {
        tdb::list_teams(pool, params.limit, params.offset)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to fetch teams: {}", e)))?
    };

    Ok(success(Some(teams), None))
}

// Get a single team by ID
async fn get_team(
    Path(team_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<crate::database::TeamDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&team_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid team ID format: {}", team_id)))?;

    let team = tdb::get_team_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Team with ID {} not found", team_id)))?;

    Ok(success(Some(team), None))
}

// Get the current player's team
async fn get_my_team(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<Option<crate::database::TeamDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    let team = tdb::get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?;

    Ok(success(Some(team), None))
}

// Create a new team
async fn create_team_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<ApiResponse<crate::database::TeamDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Get content type from headers
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Extract boundary from content-type
    let boundary = content_type
        .split("boundary=")
        .nth(1)
        .ok_or_else(|| ApiError::BadRequest("Missing boundary in content-type".to_string()))?;

    // Convert Bytes to a stream for multer by chunking into 8KB pieces
    const CHUNK_SIZE: usize = 8192;
    let chunks: Vec<Result<Bytes, multer::Error>> = body
        .chunks(CHUNK_SIZE)
        .map(|chunk| Ok(Bytes::from(chunk.to_vec())))
        .collect();
    let body_stream = stream::iter(chunks);
    let mut multipart = Multipart::new(body_stream, boundary);

    let mut name: Option<String> = None;
    let mut color: Option<String> = None;
    let mut number: Option<i32> = None;
    let mut pit_efficiency: Option<f32> = None;
    let mut logo_path: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to parse multipart form: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "name" => {
                if let Ok(value) = field.text().await {
                    name = Some(value);
                }
            }
            "color" => {
                if let Ok(value) = field.text().await {
                    color = Some(value);
                }
            }
            "number" => {
                if let Ok(value) = field.text().await {
                    if let Ok(num) = value.parse::<i32>() {
                        number = Some(num);
                    }
                }
            }
            "pit_efficiency" => {
                if let Ok(value) = field.text().await {
                    if let Ok(eff) = value.parse::<f32>() {
                        pit_efficiency = Some(eff);
                    }
                }
            }
            "logo" => {
                // Check if this is a file field
                let content_type_opt = field.content_type().map(|s| s.to_string());
                if let Some(ref content_type) = content_type_opt {
                    if content_type.starts_with("image/") {
                        // Validate file size (1MB = 1048576 bytes)
                        const MAX_FILE_SIZE: usize = 1048576;
                        let mut file_data = Vec::new();
                        let mut field_mut = field;
                        while let Ok(Some(chunk)) = field_mut.chunk().await {
                            file_data.extend_from_slice(&chunk);
                            if file_data.len() > MAX_FILE_SIZE {
                                return Err(ApiError::BadRequest(
                                    "Logo file size exceeds 1MB limit".to_string(),
                                ));
                            }
                        }

                        // Validate file type
                        let is_jpeg = content_type == "image/jpeg" || content_type == "image/jpg";
                        let is_png = content_type == "image/png";

                        if !is_jpeg && !is_png {
                            return Err(ApiError::BadRequest(
                                "Logo must be a JPG or PNG image".to_string(),
                            ));
                        }

                        // Determine file extension
                        let extension = if is_jpeg { "jpg" } else { "png" };

                        // Generate unique filename
                        let filename = format!("team_{}.{}", Uuid::new_v4(), extension);
                        let avatar_dir = StdPath::new("assets/avatars/teams");
                        let file_path = avatar_dir.join(&filename);

                        // Ensure directory exists
                        fs::create_dir_all(avatar_dir).await.map_err(|e| {
                            ApiError::InternalError(format!(
                                "Failed to create avatar directory: {}",
                                e
                            ))
                        })?;

                        // Write the file
                        fs::write(&file_path, file_data).await.map_err(|e| {
                            ApiError::InternalError(format!("Failed to save logo file: {}", e))
                        })?;

                        // Store the path relative to assets root for serving
                        logo_path = Some(format!("assets/avatars/teams/{}", filename));
                    }
                }
            }
            _ => {}
        }
    }

    // Validate required fields
    let team_name = name.ok_or_else(|| ApiError::BadRequest("name is required".to_string()))?;
    let team_color = color.ok_or_else(|| ApiError::BadRequest("color is required".to_string()))?;

    // Set player_id from token if not provided in request
    let final_player_id = player_id;

    // Check if player already has a team
    if let Some(pid) = final_player_id {
        let existing_team = tdb::get_team_by_player(pool, pid).await.map_err(|e| {
            ApiError::InternalError(format!("Failed to check existing team: {}", e))
        })?;

        if existing_team.is_some() {
            return Err(ApiError::BadRequest(
                "You already have a team. Each player can only manage one team.".to_string(),
            ));
        }
    }

    // Check if team number already exists (only if number is provided)
    if let Some(num) = number {
        let existing_team = crate::database::get_team_by_number(pool, num)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to check team number: {}", e)))?;

        if existing_team.is_some() {
            return Err(ApiError::BadRequest(format!(
                "Team number {} already exists",
                num
            )));
        }
    }

    // Create team request
    let team_request = CreateTeamRequest {
        number,
        name: team_name,
        logo: logo_path, // Will be converted to empty string in query if None
        color: team_color,
        pit_efficiency,
        player_id: final_player_id,
    };

    let team = tdb::create_team(pool, team_request)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to create team: {}", e)))?;

    Ok(success(
        Some(team),
        Some("Team created successfully".to_string()),
    ))
}

// Get all drivers
async fn get_drivers(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<DriverResponse>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let drivers = tdb::list_drivers(pool, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch drivers: {}", e)))?;

    // Convert each driver to DriverResponse with avatar
    let mut drivers_with_avatars = Vec::new();
    for driver in drivers {
        let driver_response = driver_to_response(driver).await?;
        drivers_with_avatars.push(driver_response);
    }

    Ok(success(Some(drivers_with_avatars), None))
}

// Get unassigned drivers (for market)
async fn get_unassigned_drivers(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<DriverResponse>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let drivers = tdb::list_unassigned_drivers(pool, params.limit, params.offset)
        .await
        .map_err(|e| {
            ApiError::InternalError(format!("Failed to fetch unassigned drivers: {}", e))
        })?;

    // Convert each driver to DriverResponse with avatar
    let mut drivers_with_avatars = Vec::new();
    for driver in drivers {
        let driver_response = driver_to_response(driver).await?;
        drivers_with_avatars.push(driver_response);
    }

    Ok(success(Some(drivers_with_avatars), None))
}

// Get a single driver by ID
async fn get_driver(
    Path(driver_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<DriverResponse>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&driver_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid driver ID format: {}", driver_id)))?;

    let driver = tdb::get_driver_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch driver: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Driver with ID {} not found", driver_id)))?;

    // Convert to DriverResponse with avatar
    let driver_response = driver_to_response(driver).await?;

    Ok(success(Some(driver_response), None))
}

// Get race results for a driver
async fn get_driver_race_results(
    Path(driver_id): Path<String>,
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::DriverRaceResultDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&driver_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid driver ID format: {}", driver_id)))?;

    // Verify driver exists
    let _driver = tdb::get_driver_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch driver: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Driver with ID {} not found", driver_id)))?;

    // Get race results
    let results = tdb::get_race_results_by_driver(pool, uuid, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch race results: {}", e)))?;

    Ok(success(Some(results), None))
}

// Buy a driver
async fn buy_driver(
    Path(driver_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<crate::database::TeamDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    // Get the player's team
    let team = tdb::get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("You don't have a team yet".to_string()))?;

    // Parse driver ID
    let driver_uuid = Uuid::parse_str(&driver_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid driver ID format: {}", driver_id)))?;

    // Get the driver
    let driver = tdb::get_driver_by_id(pool, driver_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch driver: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Driver with ID {} not found", driver_id)))?;

    // Check if driver is already assigned
    if driver.team_id.is_some() {
        return Err(ApiError::BadRequest(
            "Driver is already assigned to a team".to_string(),
        ));
    }

    // Calculate driver price: 100 * average of stat values
    let avg_stat = (driver.skill_level
        + driver.stamina
        + driver.weather_tolerance
        + driver.experience
        + driver.consistency
        + driver.focus)
        / 6.0;
    let price = (avg_stat * 100.0) as i32;

    // Check if team has enough cash
    if team.cash < price {
        return Err(ApiError::BadRequest(format!(
            "Insufficient cash. Required: {}, Available: {}",
            price, team.cash
        )));
    }

    // Check if team already has 4 drivers
    let driver_count = tdb::count_drivers_by_team(pool, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to count drivers: {}", e)))?;

    if driver_count >= 4 {
        return Err(ApiError::BadRequest(
            "Team already has the maximum of 4 drivers".to_string(),
        ));
    }

    // Assign driver to team
    tdb::assign_driver_to_team(pool, driver_uuid, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to assign driver: {}", e)))?;

    // Deduct cash from team
    let new_cash = team.cash - price;
    let updated_team = tdb::update_team_cash(pool, team.id, new_cash)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to update team cash: {}", e)))?;

    Ok(success(
        Some(updated_team),
        Some(format!("Driver purchased successfully for ${}", price)),
    ))
}

// Buy a car
async fn buy_car(
    Path(car_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<crate::database::TeamDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    // Get the player's team
    let team = tdb::get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("You don't have a team yet".to_string()))?;

    // Parse car ID
    let car_uuid = Uuid::parse_str(&car_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid car ID format: {}", car_id)))?;

    // Get the car
    let car = tdb::get_car_by_id(pool, car_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch car: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Car with ID {} not found", car_id)))?;

    // Check if car is already assigned
    if car.team_id.is_some() {
        return Err(ApiError::BadRequest(
            "Car is already assigned to a team".to_string(),
        ));
    }

    // Calculate car price: 100 * average of stat values
    let avg_stat = (car.handling
        + car.acceleration
        + car.top_speed
        + car.reliability
        + car.fuel_consumption
        + car.tire_wear
        + car.base_performance)
        / 7.0;
    let price = (avg_stat * 100.0) as i32;

    // Check if team has enough cash
    if team.cash < price {
        return Err(ApiError::BadRequest(format!(
            "Insufficient cash. Required: {}, Available: {}",
            price, team.cash
        )));
    }

    // Check if team already has 2 cars
    let car_count = tdb::count_cars_by_team(pool, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to count cars: {}", e)))?;

    if car_count >= 2 {
        return Err(ApiError::BadRequest(
            "Team already has the maximum of 2 cars".to_string(),
        ));
    }

    // Assign car to team
    tdb::assign_car_to_team(pool, car_uuid, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to assign car: {}", e)))?;

    // Deduct cash from team
    let new_cash = team.cash - price;
    let updated_team = tdb::update_team_cash(pool, team.id, new_cash)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to update team cash: {}", e)))?;

    Ok(success(
        Some(updated_team),
        Some(format!("Car purchased successfully for ${}", price)),
    ))
}

// Assign/unassign driver to/from car
#[derive(Deserialize)]
struct AssignDriverCarRequest {
    car_id: Option<String>,
}

#[derive(Deserialize)]
struct LevelUpDriverRequest {
    stat: String, // "skill_level", "stamina", "weather_tolerance", "consistency", or "focus"
}

async fn assign_driver_car(
    Path(driver_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AssignDriverCarRequest>,
) -> ApiResult<Json<ApiResponse<DriverResponse>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    // Get the player's team
    let team = tdb::get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("You don't have a team yet".to_string()))?;

    // Parse driver ID
    let driver_uuid = Uuid::parse_str(&driver_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid driver ID format: {}", driver_id)))?;

    // Get the driver
    let driver = tdb::get_driver_by_id(pool, driver_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch driver: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Driver with ID {} not found", driver_id)))?;

    // Verify driver belongs to the team
    if driver.team_id != Some(team.id) {
        return Err(ApiError::BadRequest(
            "Driver does not belong to your team".to_string(),
        ));
    }

    // Parse car_id if provided
    let car_uuid = if let Some(car_id_str) = request.car_id {
        let uuid = Uuid::parse_str(&car_id_str)
            .map_err(|_| ApiError::BadRequest(format!("Invalid car ID format: {}", car_id_str)))?;

        // Verify car belongs to the team
        let car = tdb::get_car_by_id(pool, uuid)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to fetch car: {}", e)))?
            .ok_or_else(|| ApiError::NotFound(format!("Car with ID {} not found", car_id_str)))?;

        if car.team_id != Some(team.id) {
            return Err(ApiError::BadRequest(
                "Car does not belong to your team".to_string(),
            ));
        }

        Some(uuid)
    } else {
        None
    };

    // If assigning to a car, check if another driver is already assigned to it
    if let Some(car_uuid_val) = car_uuid {
        // Get all drivers for the team (max 4, so no need for pagination)
        let drivers_with_car = tdb::list_drivers_by_team(pool, team.id, 100, 0)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to fetch drivers: {}", e)))?;

        // Check if another driver is already assigned to this car
        for d in drivers_with_car {
            if d.id != driver_uuid && d.car_id == Some(car_uuid_val) {
                // Unassign the other driver first
                tdb::assign_driver_to_car(pool, d.id, None)
                    .await
                    .map_err(|e| {
                        ApiError::InternalError(format!("Failed to unassign driver: {}", e))
                    })?;
                break;
            }
        }
    }

    // Assign/unassign driver to/from car
    let updated_driver = tdb::assign_driver_to_car(pool, driver_uuid, car_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to assign driver to car: {}", e)))?;

    // Convert to DriverResponse with avatar
    let driver_response = driver_to_response(updated_driver).await?;

    Ok(success(
        Some(driver_response),
        Some(if car_uuid.is_some() {
            "Driver assigned to car".to_string()
        } else {
            "Driver unassigned from car".to_string()
        }),
    ))
}

// Level up a driver by spending experience points
async fn level_up_driver_handler(
    Path(driver_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<LevelUpDriverRequest>,
) -> ApiResult<Json<ApiResponse<DriverResponse>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    // Get the player's team
    let team = tdb::get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("You don't have a team yet".to_string()))?;

    // Parse driver ID
    let driver_uuid = Uuid::parse_str(&driver_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid driver ID format: {}", driver_id)))?;

    // Get the driver
    let driver = tdb::get_driver_by_id(pool, driver_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch driver: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Driver with ID {} not found", driver_id)))?;

    // Verify driver belongs to the team
    if driver.team_id != Some(team.id) {
        return Err(ApiError::BadRequest(
            "Driver does not belong to your team".to_string(),
        ));
    }

    // Validate stat name
    let valid_stats = [
        "skill_level",
        "stamina",
        "weather_tolerance",
        "experience",
        "consistency",
        "focus",
    ];
    if !valid_stats.contains(&request.stat.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Invalid stat name: {}. Valid stats are: {}",
            request.stat,
            valid_stats.join(", ")
        )));
    }

    // Check if driver has enough experience (100 points = 1 level)
    let available_exp = driver.total_exp - driver.spent_exp;
    if available_exp < 100 {
        return Err(ApiError::BadRequest(format!(
            "Not enough experience points. Need 100 points, but only have {} available",
            available_exp
        )));
    }

    // Check if the stat is already at max (1.0) - only prevent if already at max
    let current_stat_value = match request.stat.as_str() {
        "skill_level" => driver.skill_level,
        "stamina" => driver.stamina,
        "weather_tolerance" => driver.weather_tolerance,
        "experience" => driver.experience,
        "consistency" => driver.consistency,
        "focus" => driver.focus,
        _ => 0.0, // Already validated above
    };

    // Only prevent if already at max (1.0)
    // If adding 0.1 would exceed 1.0, we allow it and the SQL will cap it at 1.0
    if current_stat_value >= 1.0 {
        return Err(ApiError::BadRequest(format!(
            "{} is already at maximum (1.0) and cannot be increased further",
            request.stat
        )));
    }

    // Level up the driver
    let updated_driver = tdb::level_up_driver(pool, driver_uuid, &request.stat)
        .await
        .map_err(|e| {
            // Convert database errors to appropriate API errors
            let error_msg = e.to_string();
            if error_msg.contains("maximum") || error_msg.contains("exceed") {
                ApiError::BadRequest(error_msg)
            } else {
                ApiError::InternalError(format!("Failed to level up driver: {}", e))
            }
        })?;

    // Convert to DriverResponse with avatar
    let driver_response = driver_to_response(updated_driver).await?;

    Ok(success(
        Some(driver_response),
        Some(format!(
            "Driver {} increased by 0.1. Spent 100 experience points.",
            request.stat
        )),
    ))
}

// Get all cars
async fn get_cars(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::CarDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let cars = tdb::list_cars(pool, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch cars: {}", e)))?;

    Ok(success(Some(cars), None))
}

// Get unassigned cars (for market)
async fn get_unassigned_cars(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::CarDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let cars = tdb::list_unassigned_cars(pool, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch unassigned cars: {}", e)))?;

    Ok(success(Some(cars), None))
}

// Get a single car by ID
async fn get_car(
    Path(car_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<crate::database::CarDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&car_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid car ID format: {}", car_id)))?;

    let car = tdb::get_car_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch car: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Car with ID {} not found", car_id)))?;

    Ok(success(Some(car), None))
}

// Get drivers for a team
async fn get_team_drivers(
    Path(team_id): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<DriverResponse>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&team_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid team ID format: {}", team_id)))?;

    let drivers = tdb::list_drivers_by_team(pool, uuid, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team drivers: {}", e)))?;

    // Convert each driver to DriverResponse with avatar
    let mut drivers_with_avatars = Vec::new();
    for driver in drivers {
        let driver_response = driver_to_response(driver).await?;
        drivers_with_avatars.push(driver_response);
    }

    Ok(success(Some(drivers_with_avatars), None))
}

// Get cars for a team
async fn get_team_cars(
    Path(team_id): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::CarDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&team_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid team ID format: {}", team_id)))?;

    let cars = tdb::list_cars_by_team(pool, uuid, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team cars: {}", e)))?;

    Ok(success(Some(cars), None))
}

// Get all tracks
async fn get_tracks(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::TrackDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let tracks = tdb::list_tracks(pool, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch tracks: {}", e)))?;

    Ok(success(Some(tracks), None))
}

// Get a single track by ID
async fn get_track(
    Path(track_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<crate::database::TrackDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&track_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid track ID format: {}", track_id)))?;

    let track = tdb::get_track_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch track: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Track with ID {} not found", track_id)))?;

    Ok(success(Some(track), None))
}

// Get all players
async fn get_players(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::PlayerDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let players = tdb::list_players(pool, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch players: {}", e)))?;

    Ok(success(Some(players), None))
}

// Get a single player by ID
async fn get_player(
    Path(player_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<crate::database::PlayerDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&player_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid player ID format: {}", player_id)))?;

    let player = tdb::get_player_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch player: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Player with ID {} not found", player_id)))?;

    Ok(success(Some(player), None))
}

// Get all races
async fn get_races(
    State(state): State<AppState>,
    Query(params): Query<RaceQueryParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::RaceDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Determine status filter based on the status parameter
    let status_filter = match params.status.as_deref() {
        Some("upcoming") => Some(vec!["REGISTRATION_OPEN", "REGISTRATION_CLOSED", "ONGOING"]),
        Some("done") => Some(vec!["FINISHED", "CANCELED"]),
        _ => None, // No filter, return all races
    };

    let races = tdb::list_races(pool, params.limit, params.offset, status_filter)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch races: {}", e)))?;

    Ok(success(Some(races), None))
}

// Get a single race by ID
async fn get_race(
    Path(race_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<crate::database::RaceDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&race_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid race ID format: {}", race_id)))?;

    let race = tdb::get_race_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch race: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Race with ID {} not found", race_id)))?;

    Ok(success(Some(race), None))
}

// Get race results for a race
async fn get_race_results(
    Path(race_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::RaceResultDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let uuid = Uuid::parse_str(&race_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid race ID format: {}", race_id)))?;

    // Verify race exists
    let _race = tdb::get_race_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch race: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Race with ID {} not found", race_id)))?;

    // Get race results
    let results = tdb::get_race_results_by_race(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch race results: {}", e)))?;

    Ok(success(Some(results), None))
}

// Create a new race
async fn create_race_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateRaceRequest>,
) -> ApiResult<Json<ApiResponse<crate::database::RaceDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    // Validate track exists
    let _track = tdb::get_track_by_id(pool, request.track_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch track: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Track not found".to_string()))?;

    // Validate laps is positive
    if request.laps <= 0 {
        return Err(ApiError::BadRequest(
            "Laps must be greater than 0".to_string(),
        ));
    }

    // Validate start_datetime is not in the past
    if let Some(start_datetime) = request.start_datetime {
        let now = Utc::now();
        if start_datetime < now {
            return Err(ApiError::BadRequest(
                "Race start time cannot be in the past".to_string(),
            ));
        }
    }

    // Create race
    let race = tdb::create_race(pool, request, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to create race: {}", e)))?;

    Ok(success(
        Some(race),
        Some("Race created successfully".to_string()),
    ))
}

// Register team for race
async fn register_for_race(
    Path(race_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<crate::database::RegistrationDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    // Get the player's team
    let team = tdb::get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("You don't have a team yet".to_string()))?;

    // Parse race ID
    let race_uuid = Uuid::parse_str(&race_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid race ID format: {}", race_id)))?;

    // Verify race exists
    let race = tdb::get_race_by_id(pool, race_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch race: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Race with ID {} not found", race_id)))?;

    // Check if race is open for registration
    if race.status != "REGISTRATION_OPEN" {
        return Err(ApiError::BadRequest(format!(
            "Race is not open for registration. Current status: {}",
            race.status
        )));
    }

    // Check if race start time is in the past
    if let Some(start_datetime) = race.start_datetime {
        let now = Utc::now();
        if start_datetime < now {
            // Set race status as REGISTRATION_CLOSED
            tdb::update_race_status(pool, race_uuid, "REGISTRATION_CLOSED")
                .await
                .map_err(|e| {
                    ApiError::InternalError(format!(
                        "Failed to close registration for past-start race: {}",
                        e
                    ))
                })?;
            return Err(ApiError::BadRequest(
                "Cannot register for a race that has already started or is in the past".to_string(),
            ));
        }
    }

    // Check if already registered
    let existing_registration = tdb::get_registration(pool, race_uuid, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to check registration: {}", e)))?;

    if existing_registration.is_some() {
        return Err(ApiError::BadRequest(
            "Your team is already registered for this race".to_string(),
        ));
    }

    // Check current registration count
    let current_count = tdb::count_registrations_by_race(pool, race_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to count registrations: {}", e)))?;

    // Check if race is already full (MAX_PARTICIPANTS participants)
    if current_count >= MAX_PARTICIPANTS {
        return Err(ApiError::BadRequest(format!(
            "Race is full. Maximum {} participants allowed.",
            MAX_PARTICIPANTS
        )));
    }

    // Create registration
    let registration = tdb::create_registration(pool, race_uuid, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to register for race: {}", e)))?;

    // Check if we just reached MAX_PARTICIPANTS // 2 participants and close registration
    let new_count = current_count + 1;
    if new_count >= MAX_PARTICIPANTS {
        tdb::update_race_status(pool, race_uuid, "REGISTRATION_CLOSED")
            .await
            .map_err(|e| {
                ApiError::InternalError(format!(
                    "Failed to close registration after reaching capacity: {}",
                    e
                ))
            })?;
    }

    Ok(success(
        Some(registration),
        Some("Successfully registered for race".to_string()),
    ))
}

// Unregister team from race
async fn unregister_from_race(
    Path(race_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<()>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header to get player_id
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let player_id =
        player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))?;

    // Get the player's team
    let team = tdb::get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("You don't have a team yet".to_string()))?;

    // Parse race ID
    let race_uuid = Uuid::parse_str(&race_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid race ID format: {}", race_id)))?;

    // Verify race exists
    let race = tdb::get_race_by_id(pool, race_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch race: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Race with ID {} not found", race_id)))?;

    // Check if race is open or closed for registration (allow unregistration in both cases)
    // Only prevent unregistration if race is already ongoing or finished
    if race.status != "REGISTRATION_OPEN" && race.status != "REGISTRATION_CLOSED" {
        return Err(ApiError::BadRequest(format!(
            "Cannot unregister from race. Current status: {}",
            race.status
        )));
    }

    // Check if registered
    let existing_registration = tdb::get_registration(pool, race_uuid, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to check registration: {}", e)))?;

    if existing_registration.is_none() {
        return Err(ApiError::BadRequest(
            "Your team is not registered for this race".to_string(),
        ));
    }

    // Check current registration count
    let current_count = tdb::count_registrations_by_race(pool, race_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to count registrations: {}", e)))?;

    // Delete registration
    let deleted = tdb::delete_registration(pool, race_uuid, team.id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to unregister from race: {}", e)))?;

    if !deleted {
        return Err(ApiError::InternalError(
            "Failed to unregister from race".to_string(),
        ));
    }

    // If race was closed (full at MAX_PARTICIPANTS) and now has less than MAX_PARTICIPANTS participants after deletion, reopen registration
    // current_count was the count before deletion, so after deletion it's current_count - 1
    if race.status == "REGISTRATION_CLOSED" && current_count == MAX_PARTICIPANTS {
        tdb::update_race_status(pool, race_uuid, "REGISTRATION_OPEN")
            .await
            .map_err(|e| {
                ApiError::InternalError(format!(
                    "Failed to reopen registration after unregistration: {}",
                    e
                ))
            })?;
    }

    Ok(success(
        None,
        Some("Successfully unregistered from race".to_string()),
    ))
}

// Get registrations for a race
async fn get_race_registrations(
    Path(race_id): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::RegistrationDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Parse race ID
    let race_uuid = Uuid::parse_str(&race_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid race ID format: {}", race_id)))?;

    // Verify race exists
    let _race = tdb::get_race_by_id(pool, race_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch race: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Race with ID {} not found", race_id)))?;

    // Get registrations
    let registrations =
        crate::database::list_registrations_by_race(pool, race_uuid, params.limit, params.offset)
            .await
            .map_err(|e| {
                ApiError::InternalError(format!("Failed to fetch registrations: {}", e))
            })?;

    Ok(success(Some(registrations), None))
}

// Get registrations for a team (with race details)
async fn get_team_registrations(
    Path(team_id): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::RegistrationWithRaceDetails>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Parse team ID
    let team_uuid = Uuid::parse_str(&team_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid team ID format: {}", team_id)))?;

    // Verify team exists
    let _team = tdb::get_team_by_id(pool, team_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Team with ID {} not found", team_id)))?;

    // Get registrations with race details
    let registrations = tdb::list_registrations_with_race_details_by_team(
        pool,
        team_uuid,
        params.limit,
        params.offset,
    )
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to fetch registrations: {}", e)))?;

    Ok(success(Some(registrations), None))
}

// Login endpoint
async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<LoginResponse>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Authenticate user
    let (player_id, token) = authenticate_user(pool, &request.username, &request.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => {
                ApiError::Unauthorized("Invalid username or password".to_string())
            }
            AuthError::DatabaseError(msg) => ApiError::InternalError(msg),
            _ => ApiError::InternalError("Authentication failed".to_string()),
        })?;

    // Store token in database
    let jwt_token = store_token(pool, player_id, &token)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to store token: {}", e)))?;

    let response = LoginResponse {
        token,
        expires_at: jwt_token.expires_at,
    };

    Ok(success(
        Some(response),
        Some("Login successful".to_string()),
    ))
}

// Register endpoint
async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> ApiResult<Json<ApiResponse<crate::database::PlayerDb>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Check if username already exists
    let existing_player = crate::database::get_player_by_username(pool, &request.username)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to check username: {}", e)))?;

    if existing_player.is_some() {
        return Err(ApiError::BadRequest("Username already exists".to_string()));
    }

    // Create player with password hash
    let create_request = CreatePlayerRequest {
        username: request.username,
        email: request.email,
        password: request.password,
    };
    let player = tdb::create_player(pool, create_request)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                ApiError::BadRequest("Username already exists".to_string())
            } else {
                ApiError::InternalError(format!("Failed to create player: {}", e))
            }
        })?;

    Ok(success(
        Some(player),
        Some("Registration successful".to_string()),
    ))
}

// Logout endpoint
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ApiResponse<()>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Extract token from Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

    // Extract token from "Bearer <token>" format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization header format".to_string()))?;

    // Delete token from database
    delete_token(pool, token)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to delete token: {}", e)))?;

    Ok(success(None, Some("Logout successful".to_string())))
}

// ========== Race Control Handlers ==========

// Get race status
async fn get_race_status(
    Path(race_id): Path<u32>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<RaceStatusResponse>>> {
    assert_eq!(race_id, 1);
    let race_state = state
        .race_state
        .lock()
        .map_err(|_| ApiError::InternalError("Failed to acquire race state lock".to_string()))?;

    // Build response data
    let run_state_str = match race_state.run_state {
        RaceRunState::Running => "running",
        RaceRunState::Paused => "paused",
        RaceRunState::Finished => "finished",
        RaceRunState::LastLap => "last_lap",
    };

    let mut cars_data = Vec::new();
    for (number, car) in &race_state.cars {
        cars_data.push(CarStatusResponse {
            number: *number,
            position: car.race_position,
            lap: car.lap,
            status: format!("{:?}", car.status).to_lowercase(),
            driving_style: format!("{:?}", car.driving_style).to_lowercase(),
            tires: TireStatusResponse {
                tire_type: format!("{:?}", car.tire.type_).to_lowercase(),
                condition: car.tire.wear,
            },
            fuel: car.fuel,
        });
    }

    // Calculate lap count from cars data
    let lap_count = race_state
        .cars
        .values()
        .map(|car| car.lap)
        .max()
        .unwrap_or(0);

    let response = RaceStatusResponse {
        run_state: run_state_str.to_string(),
        elapsed_time: race_state.tick_count as f64 * 0.1, // Convert ticks to seconds (assuming 100ms per tick)
        lap_count,
        cars: cars_data,
    };

    Ok(success(Some(response), None))
}

// Start race now - loads a scheduled race from DB and starts it
async fn start_race_now(
    Path(race_id): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<()>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    let race_uuid = Uuid::parse_str(&race_id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid race ID format: {}", race_id)))?;

    // Load the race from the database
    let mut new_race_state = crate::models::race::RaceState::load_scheduled_race(pool, race_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to load race: {}", e)))?;

    // Set the database pool for saving events
    use std::sync::Arc;
    new_race_state.set_db_pool(Arc::new(pool.clone()));

    // Replace the current race state
    {
        let mut race_state_guard = state.race_state.lock().map_err(|_| {
            ApiError::InternalError("Failed to acquire race state lock".to_string())
        })?;
        *race_state_guard = new_race_state;
    }

    // Update race status to ONGOING and set start_datetime
    tdb::start_race(pool, race_uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to start race: {:?}", e)))?;

    // Start the race
    let result = commands::handle_command("start".to_string(), state.race_state.clone());

    // Broadcast race update event
    let _ = broadcast_race_update(&state);

    Ok(success(None, Some(result)))
}

// Start race
async fn start_race(
    Path(race_id): Path<u32>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<()>>> {
    assert_eq!(race_id, 1);
    let result = commands::handle_command("start".to_string(), state.race_state.clone());

    // Broadcast race update event
    let _ = broadcast_race_update(&state);

    Ok(success(None, Some(result)))
}

// Pause race
async fn pause_race(
    Path(race_id): Path<u32>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<()>>> {
    assert_eq!(race_id, 1);
    let result = commands::handle_command("pause".to_string(), state.race_state.clone());

    // Broadcast race update event
    let _ = broadcast_race_update(&state);

    Ok(success(None, Some(result)))
}

// Stop race
async fn stop_race(
    Path(race_id): Path<u32>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<()>>> {
    assert_eq!(race_id, 1);
    let result = commands::handle_command("stop".to_string(), state.race_state.clone());

    // Broadcast race finished event
    let _ = state
        .live_tx
        .send(LiveEvent::RaceFinished(RaceFinishedEvent {
            winner: find_winner(&state.race_state).unwrap_or(0),
            total_time: get_race_time(&state.race_state).unwrap_or(0.0),
        }));

    Ok(success(None, Some(result)))
}

// Get car status
async fn get_car_status(
    Path((race_id, car_number)): Path<(u32, u32)>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<CarStatusResponse>>> {
    assert_eq!(race_id, 1);
    let race_state = state
        .race_state
        .lock()
        .map_err(|_| ApiError::InternalError("Failed to acquire race state lock".to_string()))?;

    let car = race_state
        .cars
        .get(&car_number)
        .ok_or_else(|| ApiError::NotFound(format!("Car number {} not found.", car_number)))?;

    let response = CarStatusResponse {
        number: car_number,
        position: car.race_position,
        lap: car.lap,
        status: format!("{:?}", car.status).to_lowercase(),
        driving_style: format!("{:?}", car.driving_style).to_lowercase(),
        tires: TireStatusResponse {
            tire_type: format!("{:?}", car.tire.type_).to_lowercase(),
            condition: car.tire.wear,
        },
        fuel: car.fuel,
    };

    Ok(success(Some(response), None))
}

// Set driving style
async fn set_driving_style(
    Path((race_id, car_number)): Path<(u32, u32)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DrivingStyleRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    assert_eq!(race_id, 1);

    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Authenticate and verify ownership and registration
    let player_id = extract_player_id(&headers)?;
    verify_car_ownership_and_registration(pool, &state.race_state, car_number, player_id).await?;

    let command = format!("order {} {}", car_number, request.style);
    let result = commands::handle_command(command, state.race_state.clone());

    // Broadcast car update event
    let _ = broadcast_car_update(&state, car_number);

    Ok(success(None, Some(result)))
}

// Request pit stop
async fn request_pit_stop(
    Path((race_id, car_number)): Path<(u32, u32)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PitStopRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    assert_eq!(race_id, 1);

    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;

    // Authenticate and verify ownership and registration
    let player_id = extract_player_id(&headers)?;
    verify_car_ownership_and_registration(pool, &state.race_state, car_number, player_id).await?;

    // Handle cancel request
    if request.cancel == Some(true) {
        let command = format!("nopit {}", car_number);
        let result = commands::handle_command(command, state.race_state.clone());

        // Broadcast pit stop cancel event
        let _ = state.live_tx.send(LiveEvent::PitStop(PitStopEvent {
            car_number,
            tires: None,
            refuel: None,
        }));

        return Ok(success(None, Some(result)));
    }

    // Clone the values we'll need multiple times
    let tires_clone = request.tires.clone();
    let refuel_clone = request.refuel;

    // Build the pit command based on provided options
    let mut command = format!("pit {}", car_number);

    match (request.tires, request.refuel) {
        (Some(tires), Some(refuel)) => {
            command = format!("{} {} refuel {}", command, tires, refuel);
        }
        (Some(tires), _) => {
            command = format!("{} {}", command, tires);
        }
        (_, Some(refuel)) => {
            command = format!("{} refuel {}", command, refuel);
        }
        (_, _) => {
            return Err(ApiError::BadRequest(
                "Pit stop request must specify at least tire change or refuel operation."
                    .to_string(),
            ));
        }
    }

    let result = commands::handle_command(command, state.race_state.clone());

    // Broadcast pit stop event
    let _ = state.live_tx.send(LiveEvent::PitStop(PitStopEvent {
        car_number,
        tires: tires_clone,
        refuel: refuel_clone,
    }));

    Ok(success(None, Some(result)))
}

// Helper functions

// Extract player_id from Authorization header
fn extract_player_id(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let player_id = if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::auth::validate_token(token) {
                    Some(claims.sub)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    player_id.ok_or_else(|| ApiError::Unauthorized("Authentication required".to_string()))
}

// Verify that the player owns the car and is registered for the race
async fn verify_car_ownership_and_registration(
    pool: &PgPool,
    race_state: &SharedRaceState,
    car_number: u32,
    player_id: Uuid,
) -> Result<(), ApiError> {
    let (car, race_id) = {
        let race_state_guard = race_state.lock().map_err(|_| {
            ApiError::InternalError("Failed to acquire race state lock".to_string())
        })?;

        let car = race_state_guard
            .cars
            .get(&car_number)
            .ok_or_else(|| ApiError::NotFound(format!("Car number {} not found.", car_number)))?
            .clone();

        let race_id = race_state_guard.race_id;

        (car, race_id)
    };

    // Check if car has a player_uuid and if it matches the authenticated player
    let car_player_uuid = match &car.player_uuid {
        Some(car_player_uuid_str) => Uuid::parse_str(car_player_uuid_str)
            .map_err(|_| ApiError::InternalError("Invalid player UUID in car".to_string()))?,
        None => {
            // Car has no player_uuid, meaning it's an AI car - no one can control it
            return Err(ApiError::Forbidden(
                "This car is not controlled by a player".to_string(),
            ));
        }
    };

    if car_player_uuid != player_id {
        return Err(ApiError::Forbidden("You do not own this car".to_string()));
    }

    // If this is a scheduled race (has a race_id), verify the player's team is registered
    if let Some(race_id) = race_id {
        let team = tdb::get_team_by_player(pool, player_id)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?
            .ok_or_else(|| ApiError::Forbidden("You do not have a team".to_string()))?;

        let registration = tdb::get_registration(pool, race_id, team.id)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to check registration: {}", e)))?;

        if registration.is_none() {
            return Err(ApiError::Forbidden(
                "Your team is not registered for this race".to_string(),
            ));
        }
    }
    // If race_id is None, it's a race loaded from config file - ownership check is sufficient

    Ok(())
}

// Find the current leader/winner
fn find_winner(race_state: &SharedRaceState) -> Option<u32> {
    if let Ok(state) = race_state.lock() {
        state
            .cars
            .iter()
            .filter(|(_, car)| car.status == CarStatus::Racing)
            .min_by_key(|(_, car)| car.race_position)
            .map(|(number, _)| *number)
    } else {
        None
    }
}

// Get the current race time
fn get_race_time(race_state: &SharedRaceState) -> Option<f32> {
    if let Ok(state) = race_state.lock() {
        Some(state.tick_count as f32 * state.tick_duration_seconds) // Convert ticks to seconds
    } else {
        None
    }
}

// Broadcast a race update event
fn broadcast_race_update(state: &AppState) -> Result<(), broadcast::error::SendError<LiveEvent>> {
    if let Ok(race_state) = state.race_state.lock() {
        let run_state = match race_state.run_state {
            RaceRunState::Running => "running",
            RaceRunState::Paused => "paused",
            RaceRunState::Finished => "finished",
            RaceRunState::LastLap => "last_lap",
        };

        state
            .live_tx
            .send(LiveEvent::RaceUpdate(RaceUpdateEvent {
                run_state: run_state.to_string(),
                elapsed_time: race_state.tick_count as f64 * 0.1, // Convert ticks to seconds
                lap_count: race_state
                    .cars
                    .values()
                    .map(|car| car.lap)
                    .max()
                    .unwrap_or(0),
            }))
            .map(|_| ())
    } else {
        Ok(()) // Silently fail if we can't get the lock
    }
}

// Broadcast a car update event
fn broadcast_car_update(
    state: &AppState,
    car_number: u32,
) -> Result<(), broadcast::error::SendError<LiveEvent>> {
    if let Ok(race_state) = state.race_state.lock() {
        if let Some(car) = race_state.cars.get(&car_number) {
            state
                .live_tx
                .send(LiveEvent::CarUpdate(CarUpdateEvent {
                    car_number,
                    position: car.race_position,
                    lap: car.lap,
                    status: format!("{:?}", car.status).to_lowercase(),
                }))
                .map(|_| ())
        } else {
            Ok(()) // Car not found
        }
    } else {
        Ok(()) // Silently fail if we can't get the lock
    }
}

// Response DTOs
#[derive(Serialize)]
struct RaceStatusResponse {
    run_state: String,
    elapsed_time: f64,
    lap_count: u32,
    cars: Vec<CarStatusResponse>,
}

#[derive(Serialize)]
struct CarStatusResponse {
    number: u32,
    position: u32,
    lap: u32,
    status: String,
    driving_style: String,
    tires: TireStatusResponse,
    fuel: f32,
}

#[derive(Serialize)]
struct TireStatusResponse {
    tire_type: String,
    condition: f32,
}
