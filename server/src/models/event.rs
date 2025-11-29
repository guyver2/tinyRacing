use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: u16,
    pub description: String,
    pub event_type: EventType,
    pub data: EventData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventType {
    StartRace,
    EndRace,
    PitRequest,
    PitCancel,
    PitStop,
    WeatherChange,
    Accident,
    CarFinished,
    Dnf,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventData {
    pub car_number: Option<u32>,
    pub team_name: Option<String>,
    pub driver_name: Option<String>,
    pub tire: Option<String>,
    pub fuel: Option<f32>,
    pub weather: Option<String>,
    pub time_offset_seconds: f32,
}
