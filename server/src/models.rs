use std::fs;
use std::io::{self, Read};

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
    // pub car_stats: CarStats,
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

impl Car {
    // todo replace this car stats
    pub fn acceleration(&self) -> f32 {
        10.0
    }

    pub fn max_speed(&self) -> f32 {
        if self.status == CarStatus::Pit {
            return 30.0;
        } else if self.status == CarStatus::Finished || self.status == CarStatus::Dnf {
            return 0.0;
        }
        let tire_type_factor = match self.tire.type_ {
            TireType::Soft => 1.05,
            TireType::Medium => 1.0,
            TireType::Hard => 0.95,
            TireType::Intermediate => 0.9,
            TireType::Wet => 0.8,
        };
        // Map tire wear: 0% wear -> 1.0, 100% wear -> 0.9
        let tire_wear_factor = 1.0 - self.tire.wear / 1000.0;
        let tire = tire_type_factor * tire_wear_factor;
        // Map fuel: 0% fuel -> 1.0, 100% fuel -> 0.9
        let fuel_factor = 1.0 - self.fuel / 1000.0;

        let driving_style_factor = match self.driving_style {
            DrivingStyle::Relax => 0.95,
            DrivingStyle::Normal => 1.0,
            DrivingStyle::Aggressive => 1.05,
        };

        // TODO:
        // - add car stats
        // - add weather factor
        // - add track curvature factor
        // - add driver skill factor
        let max_speed = 300.0 * self.base_performance * tire * fuel_factor * driving_style_factor;
        max_speed
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team {
    pub number: u32,
    pub name: String,
    // Could add team-specific attributes later
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TrackPoint {
    pub x: f32,
    pub y: f32,
    pub curvature: f32,
}

//maps the track.json file definition
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub laps: u32,
    pub lap_length_km: f32,
    pub svg_start_offset: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub laps: u32,
    pub lap_length_km: f32,
    pub sampled_track: Vec<TrackPoint>,
}

impl Track {
    pub fn get_track_point_at_distance(&self, lap_ratio: f32) -> TrackPoint {
        let index = (lap_ratio * self.sampled_track.len() as f32).round() as usize;
        self.sampled_track[index % self.sampled_track.len()]
    }

    pub fn load_track_config(path: &str) -> Result<Track, io::Error> {
        let data = fs::read_to_string(format!("{}/track.json", path))?;
        let track_config: TrackConfig = serde_json::from_str(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Create Track from TrackConfig and initialize sampled_track
        let mut track = Track {
            id: track_config.id,
            name: track_config.name,
            laps: track_config.laps,
            lap_length_km: track_config.lap_length_km,
            sampled_track: Vec::new(), // Initialize empty, to be computed later
        };
        track.sampled_track =
            Self::load_track_curvature(format!("{}/curvature.bin", path).as_str())?;

        Ok(track)
    }

    fn load_track_curvature(path: &str) -> Result<Vec<TrackPoint>, io::Error> {
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File too small"));
        }

        // Read number of points (4 bytes, little-endian)
        let count = i32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;

        // Check if we have enough data for all points
        let expected_size = 4 + (count * 12); // 4 bytes for count + 12 bytes per point (3 floats)
        if buffer.len() < expected_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "File too small for expected data",
            ));
        }

        let mut points = Vec::with_capacity(count);
        let mut offset = 4; // Start after the count

        for _ in 0..count {
            // Read x coordinate (4 bytes)
            let x = f32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            offset += 4;

            // Read y coordinate (4 bytes)
            let y = f32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            offset += 4;

            // Read curvature (4 bytes)
            let curvature = f32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            offset += 4;

            points.push(TrackPoint { x, y, curvature });
        }

        Ok(points)
    }
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
    pub track: Track,
    pub cars: Vec<CarClientData>,
    pub current_lap: u32, // Max lap across all cars? Or based on leader?
    pub total_laps: u32,
    pub race_status: RaceRunState,
}
