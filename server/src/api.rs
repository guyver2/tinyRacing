use crate::auth::{authenticate_user, delete_token, hash_password, store_token, AuthError};
use crate::commands;
use crate::database::{
    create_team, CreateTeamRequest, LoginRequest, LoginResponse, RegisterRequest,
};
use crate::database::{
    get_car_by_id, get_driver_by_id, get_player_by_id, get_team_by_id, get_team_by_player,
    get_track_by_id,
};
use crate::database::{
    list_cars, list_drivers, list_players, list_teams, list_teams_by_player, list_tracks,
    list_unassigned_cars, list_unassigned_drivers,
};
use crate::models::car::CarStatus;
use crate::models::driver_avatar::generate_driver_avatar;
use crate::models::race::{RaceRunState, RaceState};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::PgPool;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path as StdPath;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
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
    tires: Option<String>,
    #[serde(deserialize_with = "deserialize_refuel")]
    refuel: Option<f32>,
}

#[derive(Deserialize)]
struct TeamQueryParams {
    player_id: Option<String>,
}

// Implementation of response conversion for ApiError
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
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
        .route("/teams/{team_id}", get(get_team))
        .route("/drivers/{driver_id}", get(get_driver))
        .route("/cars/{car_id}", get(get_car))
        .route("/tracks/{track_id}", get(get_track))
        .route("/players/{player_id}", get(get_player))
        // create routes
        .route("/teams", post(create_team_handler))
        // .route("/drivers", post(create_driver))
        // .route("/cars", post(create_car))
        // .route("/tracks", post(create_track))
        //.route("/players", post(create_player)) // players are created through register
        // update routes
        // .route("/teams/{team_id}", put(update_team))
        // .route("/drivers/{driver_id}", put(update_driver))
        // .route("/cars/{car_id}", put(update_car))
        // .route("/tracks/{track_id}", put(update_track))
        // delete routes
        // .route("/teams/{team_id}", delete(delete_team))
        // .route("/drivers/{driver_id}", delete(delete_driver))
        // .route("/cars/{car_id}", delete(delete_car))
        // .route("/tracks/{track_id}", delete(delete_track))
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
        list_teams_by_player(pool, player_id)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to fetch teams: {}", e)))?
    } else {
        list_teams(pool)
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

    let team = get_team_by_id(pool, uuid)
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

    let team = get_team_by_player(pool, player_id)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch team: {}", e)))?;

    Ok(success(Some(team), None))
}

// Create a new team
async fn create_team_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateTeamRequest>,
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

    // Set player_id from token if not provided in request
    let mut team_request = request;
    let final_player_id = if team_request.player_id.is_some() {
        team_request.player_id
    } else {
        player_id
    };

    // Check if player already has a team
    if let Some(pid) = final_player_id {
        let existing_team = get_team_by_player(pool, pid).await.map_err(|e| {
            ApiError::InternalError(format!("Failed to check existing team: {}", e))
        })?;

        if existing_team.is_some() {
            return Err(ApiError::BadRequest(
                "You already have a team. Each player can only manage one team.".to_string(),
            ));
        }

        team_request.player_id = Some(pid);
    }

    // Check if team number already exists (only if number is provided)
    if let Some(number) = team_request.number {
        let existing_team = crate::database::get_team_by_number(pool, number)
            .await
            .map_err(|e| ApiError::InternalError(format!("Failed to check team number: {}", e)))?;

        if existing_team.is_some() {
            return Err(ApiError::BadRequest(format!(
                "Team number {} already exists",
                number
            )));
        }
    }

    let team = create_team(pool, team_request)
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
) -> ApiResult<Json<ApiResponse<Vec<DriverResponse>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let drivers = list_drivers(pool)
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
) -> ApiResult<Json<ApiResponse<Vec<DriverResponse>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let drivers = list_unassigned_drivers(pool).await.map_err(|e| {
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

    let driver = get_driver_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch driver: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Driver with ID {} not found", driver_id)))?;

    // Convert to DriverResponse with avatar
    let driver_response = driver_to_response(driver).await?;

    Ok(success(Some(driver_response), None))
}

// Get all cars
async fn get_cars(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::CarDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let cars = list_cars(pool)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch cars: {}", e)))?;

    Ok(success(Some(cars), None))
}

// Get unassigned cars (for market)
async fn get_unassigned_cars(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::CarDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let cars = list_unassigned_cars(pool)
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

    let car = get_car_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch car: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Car with ID {} not found", car_id)))?;

    Ok(success(Some(car), None))
}

// Get all tracks
async fn get_tracks(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::TrackDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let tracks = list_tracks(pool)
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

    let track = get_track_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch track: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Track with ID {} not found", track_id)))?;

    Ok(success(Some(track), None))
}

// Get all players
async fn get_players(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<crate::database::PlayerDb>>>> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Database not available".to_string()))?;
    let players = list_players(pool)
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

    let player = get_player_by_id(pool, uuid)
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to fetch player: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("Player with ID {} not found", player_id)))?;

    Ok(success(Some(player), None))
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

    // Hash password
    let password_hash = hash_password(&request.password)
        .map_err(|e| ApiError::InternalError(format!("Failed to hash password: {}", e)))?;

    // Create player with password hash
    let player = sqlx::query_as::<_, crate::database::PlayerDb>(
        r#"
        INSERT INTO player (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&request.username)
    .bind(&request.email)
    .bind(&password_hash)
    .fetch_one(pool)
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
    Json(request): Json<DrivingStyleRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    assert_eq!(race_id, 1);
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
    Json(request): Json<PitStopRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    assert_eq!(race_id, 1);
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
