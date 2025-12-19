use crate::models::driver::{Driver, DrivingStyle};
use crate::models::team::Team;
use crate::models::tire::{ClientTireData, Tire, TireType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum CarStatus {
    Racing,
    Pit,
    Finished,
    Dnf, // Did Not Finish
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
}

impl CarStats {
    pub fn new() -> Self {
        CarStats {
            handling: 0.5,
            acceleration: 0.5,
            top_speed: 0.5,
            reliability: 0.5,
            fuel_consumption: 0.5,
            tire_wear: 0.5,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Car {
    pub uid: Uuid,
    pub number: u32,
    pub team: Team,
    pub driver: Driver,
    pub stats: CarStats,
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
    pub player_uuid: Option<String>,
}

impl Car {
    pub fn acceleration(&self) -> f32 {
        // Base acceleration from car stats (0.0 to 1.0, mapped to 5.0 to 15.0 km/h per tick)
        let base_accel = 5.0 + (self.stats.acceleration * 10.0);

        // Driver skill affects acceleration (0.0 to 1.0, adds 0% to 10% bonus)
        let driver_skill_factor = 1.0 + (self.driver.skill_level * 0.1);

        // Driving style affects acceleration
        let driving_style_factor = match self.driving_style {
            DrivingStyle::Relax => 0.95,
            DrivingStyle::Normal => 1.0,
            DrivingStyle::Aggressive => 1.05,
        };

        base_accel * driver_skill_factor * driving_style_factor
    }

    pub fn max_speed(&self) -> f32 {
        if self.status == CarStatus::Pit {
            return 30.0;
        } else if self.status == CarStatus::Finished || self.status == CarStatus::Dnf {
            return 0.0;
        }

        // Base top speed from car stats (0.0 to 1.0, mapped to 200.0 to 400.0 km/h)
        let base_top_speed = 200.0 + (self.stats.top_speed * 200.0);

        // Tire type factor
        let tire_type_factor = match self.tire.type_ {
            TireType::Soft => 1.05,
            TireType::Medium => 1.0,
            TireType::Hard => 0.95,
            TireType::Intermediate => 0.9,
            TireType::Wet => 0.8,
        };

        // Map tire wear: 0% wear -> 1.0, 100% wear -> 0.9
        let tire_wear_factor = 1.0 - (self.tire.wear / 1000.0);
        let tire_factor = tire_type_factor * tire_wear_factor;

        // Map fuel: 0% fuel -> 1.0, 100% fuel -> 0.9
        let fuel_factor = 1.0 - (self.fuel / 1000.0);

        // Driving style factor
        let driving_style_factor = match self.driving_style {
            DrivingStyle::Relax => 0.95,
            DrivingStyle::Normal => 1.0,
            DrivingStyle::Aggressive => 1.05,
        };

        // Driver skill affects top speed (0.0 to 1.0, adds 0% to 5% bonus)
        let driver_skill_factor = 1.0 + (self.driver.skill_level * 0.05);

        // Handling affects top speed in corners (better handling = less speed loss)
        // This is already handled in the race update loop via curvature_factor,
        // but we can add a small base bonus here
        let handling_factor = 0.98 + (self.stats.handling * 0.04); // 0.98 to 1.02

        let max_speed = base_top_speed
            * self.base_performance
            * tire_factor
            * fuel_factor
            * driving_style_factor
            * driver_skill_factor
            * handling_factor;

        max_speed
    }
}

#[derive(Serialize, Debug, Clone)] // Only Serialize for sending to clients
pub struct CarClientData {
    pub car_number: u32,
    pub driver: Driver, // Just the name for the client
    pub carstats: CarStats,
    pub team: Team,
    pub race_position: u32,
    pub track_position: f32, // Combined lap.percentage
    pub status: CarStatus,
    pub tire: ClientTireData,
    pub fuel: f32,
    pub driving_style: DrivingStyle,
    pub speed: f32,         // Current speed in km/h
    pub finished_time: u64, // Ticks taken to finish
    pub player_uuid: Option<String>,
    pub pit_requested: Option<bool>,
}
