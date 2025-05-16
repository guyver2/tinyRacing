use serde::{Deserialize, Serialize};

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
    // all skills are 0.0 to 1.0
    pub skill_level: f32,       // how good the driver is at the car
    pub stamina: f32, // how well the driver can handle the physical demands of the car, stamina decreases with race time
    pub weather_tolerance: f32, // how well the driver can handle the wet weather
    pub experience: f32, // how experienced the driver is, more experience means less mistakes
    pub consistency: f32, // how consistent the driver is, less variance in performance
    pub focus: f32,   // how focused the driver is, how much they are susceptible to be in the zone
}

impl Driver {
    pub fn new(json: &str) -> Self {
        let driver: Driver = serde_json::from_str(json).unwrap();
        driver
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CarStats {
    // all skills are 0.0 to 1.0
    pub handling: f32,         // how well the car manage turns
    pub acceleration: f32,     // how well the car can accelerate
    pub top_speed: f32,        // how fast the car can go
    pub reliability: f32,      // how reliable the car is, reliability decreases with race time
    pub fuel_consumption: f32, // how much fuel the car consumes
    pub tire_wear: f32,        // how much tire wear the car gets
    pub pit_stop_time: f32,    // how long the pit stop takes
    pub pit_stop_fuel: f32,    // how much fuel the car consumes during a pit stop
    pub pit_stop_tire: f32,    // how much tire wear the car gets during a pit stop
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamStats {
    pub pit_stop_time_fuel: f32, // how much time it takes to refuel the car
    pub pit_stop_time_tire: f32, // how much time it takes to change the tires
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
