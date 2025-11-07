use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeamStats {
    pub pit_stop_time_fuel: f32, // how much time it takes to refuel the car
    pub pit_stop_time_tire: f32, // how much time it takes to change the tires
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team {
    pub number: u32,
    pub name: String,
    // Could add team-specific attributes later
}
