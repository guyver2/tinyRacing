use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};

use crate::models::weather::Weather;

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
    pub weather: Weather,
    pub wetness: f32, // 0.0 (dry) to 1.0 (wet)
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
            //weather: Weather::new_random(2.0 * 60.0 * 60.0), // 2 hours
            weather: Weather {
                state_change_time: vec![(0.0, 1.0), (60.0, 0.0)],
            },
            wetness: 0.0,
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

#[derive(Serialize, Debug, Clone)]
pub struct TrackClientData {
    pub id: String,
    pub name: String,
    pub laps: u32,
    pub lap_length_km: f32,
    pub sampled_track: Vec<TrackPoint>,
    pub current_weather: String,
    pub wetness: f32,      // 0.0 (dry) to 1.0 (wet)
    pub elapsed_time: f32, // seconds
}

impl TrackClientData {
    pub fn new(track: &Track, time: f32) -> Self {
        Self {
            id: track.id.clone(),
            name: track.name.clone(),
            laps: track.laps,
            lap_length_km: track.lap_length_km,
            sampled_track: track.sampled_track.clone(),
            current_weather: Weather::to_weather_string(track.weather.get_state_at_time(time)),
            wetness: track.wetness,
            elapsed_time: time,
        }
    }
}
