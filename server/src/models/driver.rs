use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DrivingStyle {
    Relax,
    Normal,
    Aggressive,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Driver {
    pub uid: Option<Uuid>,
    pub name: String,
    // all skills are 0.0 to 1.0
    pub skill_level: f32,       // how good the driver is at the car
    pub stamina: f32, // how well the driver can handle the physical demands of the car, stamina decreases with race time
    pub weather_tolerance: f32, // how well the driver can handle the wet weather
    pub experience: f32, // how experienced the driver is, more experience means less mistakes
    pub consistency: f32, // how consistent the driver is, less variance in performance
    pub focus: f32,   // how focused the driver is, how much they are susceptible to be in the zone and how much they resist and recover from stress
    pub stress_level: f32, // how stressed the driver is (0.0 to 1.0). Increases with time when aggressive, decreases slowly when normal, decreases faster when relaxed
}

impl Driver {
    pub fn new(json: &str) -> Self {
        let driver: Driver = serde_json::from_str(json).unwrap();
        driver
    }
}
