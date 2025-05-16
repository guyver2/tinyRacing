use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TireType {
    Soft,
    Medium,
    Hard,
    Intermediate,
    Wet,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DrivingStyle {
    Relax,
    Normal,
    Aggressive,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum CarStatus {
    Racing,
    Pit,
    Finished,
    Dnf, // Did Not Finish
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tire {
    pub type_: TireType,
    pub wear: f32, // 0.0 to 100.0 %
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Driver {
    pub name: String,
    pub skill_level: f32, // 0.5 to 1.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Car {
    pub number: u32,
    pub team_number: u32,
    pub team_name: String,
    pub driver: Driver,
    pub tire: Tire,
    pub fuel: f32, // 0.0 to 100.0 %
    pub driving_style: DrivingStyle,
    pub status: CarStatus,
    pub race_position: u32, // 1st, 2nd, etc.
    pub lap: u32,
    pub lap_percentage: f32,   // 0.0 to 1.0
    pub total_distance: f32,   // Total distance traveled in km
    pub finished_time: u64,    // Ticks taken to finish
    pub base_performance: f32, // 0.9 to 1.1 multiplier
    pub speed: f32,            // Current speed in km/h
    // Fields for pit stop planning
    pub pit_request: bool,
    pub target_tire: Option<TireType>,
    pub target_fuel: Option<f32>,
    pub pit_time_remaining: u32, // Ticks remaining in pit stop
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team {
    pub number: u32,
    pub name: String,
    // Could add team-specific attributes later
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Track {
    pub name: String,
    pub laps: u32,
    pub lap_length_km: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RaceRunState {
    Paused,
    Running,
    LastLap,
    Finished,
}



#[derive(Serialize, Debug, Clone)] // Only Serialize for sending to clients
pub struct CarClientData {
    pub car_number: u32,
    pub driver: String,    // Just the name for the client
    pub team_number: u32,  // Team number
    pub team_name: String, // Added for UI
    pub race_position: u32,
    pub track_position: f32, // Combined lap.percentage
    pub status: CarStatus,
    pub tire: ClientTireData,
    pub fuel: f32,
    pub driving_style: DrivingStyle,
    pub speed: f32,         // Current speed in km/h
    pub finished_time: u64, // Ticks taken to finish
}

#[derive(Serialize, Debug, Clone)]
pub struct ClientTireData {
    #[serde(rename = "type")]
    pub type_: TireType,
    pub wear: f32,
}

#[derive(Serialize, Debug, Clone)] // Only Serialize for sending to clients
pub struct RaceStateClientView {
    pub cars: Vec<CarClientData>,
    pub current_lap: u32, // Max lap across all cars? Or based on leader?
    pub total_laps: u32,
    pub race_status: RaceRunState,
}
