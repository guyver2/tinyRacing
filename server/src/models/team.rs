use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Team {
    pub uid: Uuid,
    pub number: u32,
    pub name: String,
    pub logo: String,
    pub color: String,
    pub pit_efficiency: f32,
}
