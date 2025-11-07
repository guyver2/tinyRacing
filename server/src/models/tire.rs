use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TireType {
    Soft,
    Medium,
    Hard,
    Intermediate,
    Wet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tire {
    pub type_: TireType,
    pub wear: f32, // 0.0 to 100.0 %
}

#[derive(Serialize, Debug, Clone)]
pub struct ClientTireData {
    #[serde(rename = "type")]
    pub type_: TireType,
    pub wear: f32,
}
