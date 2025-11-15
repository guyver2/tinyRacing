use chrono::{DateTime, Utc};
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a Driver
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DriverDb {
    pub id: Uuid,
    pub name: String,
    pub skill_level: f32,
    pub stamina: f32,
    pub weather_tolerance: f32,
    pub experience: f32,
    pub consistency: f32,
    pub focus: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a Car
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CarDb {
    pub id: Uuid,
    pub number: i32,
    pub team_id: Uuid,
    pub driver_id: Uuid,
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
    pub svg_start_offset: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database representation of a Player
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerDb {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response DTOs for creating entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamRequest {
    pub number: i32,
    pub name: String,
    pub logo: String,
    pub color: String,
    pub pit_efficiency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDriverRequest {
    pub name: String,
    pub skill_level: f32,
    pub stamina: f32,
    pub weather_tolerance: f32,
    pub experience: f32,
    pub consistency: f32,
    pub focus: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCarRequest {
    pub number: i32,
    pub team_id: Uuid,
    pub driver_id: Uuid,
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
    pub svg_start_offset: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlayerRequest {
    pub username: String,
    pub email: Option<String>,
}

