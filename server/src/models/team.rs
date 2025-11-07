use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team {
    pub number: u32,
    pub name: String,
    pub logo: String,
    pub color: String,
    pub pit_efficiency: f32,
}
