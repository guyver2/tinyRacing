use crate::commands;
use crate::models::car::CarStatus;
use crate::models::race::{RaceRunState, RaceState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

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
    total_time: f64,
}

// API Error type
#[derive(Debug)]
enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
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

// Implementation of response conversion for ApiError
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
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

// Create the API Router
pub fn create_api_router(race_state: SharedRaceState) -> Router {
    // Create a channel for live updates
    let (tx, _) = broadcast::channel::<LiveEvent>(100);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Race control routes
        .route("/race", get(get_race_status))
        .route("/race/start", post(start_race))
        .route("/race/pause", post(pause_race))
        .route("/race/stop", post(stop_race))
        // Car control routes
        .route("/cars/{car_number}", get(get_car_status))
        .route("/cars/{car_number}/driving-style", put(set_driving_style))
        // Pit stop routes
        .route("/cars/{car_number}/pit", post(request_pit_stop))
        // Apply CORS middleware
        .layer(cors)
        // Share state across handlers
        .with_state(AppState {
            race_state,
            live_tx: tx,
        })
}

// Application state to be shared across handlers
#[derive(Clone)]
struct AppState {
    race_state: SharedRaceState,
    live_tx: broadcast::Sender<LiveEvent>,
}

// Route handler implementations

// Get race status
async fn get_race_status(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<RaceStatusResponse>>> {
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
async fn start_race(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<()>>> {
    let result = commands::handle_command("start".to_string(), state.race_state.clone());

    // Broadcast race update event
    let _ = broadcast_race_update(&state);

    Ok(success(None, Some(result)))
}

// Pause race
async fn pause_race(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<()>>> {
    let result = commands::handle_command("pause".to_string(), state.race_state.clone());

    // Broadcast race update event
    let _ = broadcast_race_update(&state);

    Ok(success(None, Some(result)))
}

// Stop race
async fn stop_race(State(state): State<AppState>) -> ApiResult<Json<ApiResponse<()>>> {
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
    Path(car_number): Path<u32>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<CarStatusResponse>>> {
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
    Path(car_number): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<DrivingStyleRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
    let command = format!("order {} {}", car_number, request.style);
    let result = commands::handle_command(command, state.race_state.clone());

    // Broadcast car update event
    let _ = broadcast_car_update(&state, car_number);

    Ok(success(None, Some(result)))
}

// Request pit stop
async fn request_pit_stop(
    Path(car_number): Path<u32>,
    State(state): State<AppState>,
    Json(request): Json<PitStopRequest>,
) -> ApiResult<Json<ApiResponse<()>>> {
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
fn get_race_time(race_state: &SharedRaceState) -> Option<f64> {
    if let Ok(state) = race_state.lock() {
        Some(state.tick_count as f64 * 0.1) // Convert ticks to seconds
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
