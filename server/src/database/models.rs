use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Database representation of a Team
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamDb {
    pub id: Uuid,
    pub number: i32,
    pub name: String,
    pub logo: String,
    pub color: String,
    pub pit_efficiency: f32,
    pub cash: i32,
    pub player_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a Driver
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DriverDb {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: NaiveDate,
    pub nationality: String,
    pub gender: String,
    pub skill_level: f32,
    pub stamina: f32,
    pub weather_tolerance: f32,
    pub experience: f32,
    pub consistency: f32,
    pub focus: f32,
    pub team_id: Option<Uuid>,
    pub car_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a Car
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarDb {
    pub id: Uuid,
    pub number: i32,
    pub team_id: Option<Uuid>,
    pub handling: f32,
    pub acceleration: f32,
    pub top_speed: f32,
    pub reliability: f32,
    pub fuel_consumption: f32,
    pub tire_wear: f32,
    pub base_performance: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a Track
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrackDb {
    pub id: Uuid,
    pub track_id: String, // The track identifier (e.g., "bahrain", "monaco")
    pub name: String,
    pub description: Option<String>,
    pub laps: i32,
    pub lap_length_km: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a Player
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerDb {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a JWT Token
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JwtTokenDb {
    pub id: Uuid,
    pub player_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response DTOs for creating entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamRequest {
    #[serde(default)]
    pub number: Option<i32>,
    pub name: String,
    pub logo: String,
    pub color: String,
    #[serde(default)]
    pub pit_efficiency: Option<f32>,
    pub player_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDriverRequest {
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: NaiveDate,
    pub nationality: String,
    pub gender: String,
    pub skill_level: f32,
    pub stamina: f32,
    pub weather_tolerance: f32,
    pub experience: f32,
    pub consistency: f32,
    pub focus: f32,
    pub team_id: Option<Uuid>,
    pub car_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCarRequest {
    pub number: i32,
    pub team_id: Option<Uuid>,
    pub handling: f32,
    pub acceleration: f32,
    pub top_speed: f32,
    pub reliability: f32,
    pub fuel_consumption: f32,
    pub tire_wear: f32,
    pub base_performance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTrackRequest {
    pub track_id: String,
    pub name: String,
    pub description: Option<String>,
    pub laps: i32,
    pub lap_length_km: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlayerRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}
