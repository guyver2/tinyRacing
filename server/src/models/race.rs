use crate::database::models::{CreateEventRequest, DriverDb, TeamDb};
use crate::database::queries as tdb;
use crate::models::car::{Car, CarClientData, CarStats, CarStatus};
use crate::models::driver::{Driver, DrivingStyle};
use crate::models::event::{Event, EventData, EventType};
use crate::models::team::Team;
use crate::models::tire::{ClientTireData, Tire, TireType};
use crate::models::track::Track;
use crate::models::track::TrackClientData;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::io::{self};
use std::sync::Arc;
use uuid::Uuid;

pub const MAX_PARTICIPANTS: i64 = 5;
const AUTO_RACE_RESTART: bool = false;

/// Check if auto race restart is enabled via environment variable
pub fn is_auto_race_restart_enabled() -> bool {
    AUTO_RACE_RESTART
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RaceRunState {
    Paused,
    Running,
    LastLap,
    Finished,
}

#[derive(Serialize, Debug, Clone)] // Only Serialize for sending to clients
pub struct RaceStateClientView {
    pub track: TrackClientData,
    pub cars: Vec<CarClientData>,
    pub current_lap: u32, // Max lap across all cars? Or based on leader?
    pub total_laps: u32,
    pub race_status: RaceRunState,
}

/// Race state module for managing the race simulation
///
/// This module handles the core race simulation state and logic, including:
/// - Track configuration and properties
/// - Car and driver state management
/// - Race progression and timing
/// - Pit stops and tire management
/// - Weather conditions and their effects
/// - Race events and incidents
///
/// The race state is updated on each tick of the simulation loop and provides
/// a client view for the UI and network interfaces.
///
/// Key features:
/// - Real-time race position tracking
/// - Tire wear and fuel consumption simulation
/// - Driver skill and consistency effects
/// - Weather impact on performance
/// - Pit stop strategy handling
/// - Race control and flagging system
///
/// The state can be modified through commands from the UI or network clients
/// while maintaining consistency and race regulations.

#[derive(Debug, Clone)]
pub struct RaceState {
    pub track: Track,
    pub cars: HashMap<u32, Car>, // Keyed by car number
    pub run_state: RaceRunState,
    pub tick_count: u64,
    pub tick_duration_seconds: f32,
    pub events: Vec<Event>,
    pub race_id: Option<Uuid>, // ID of the race in the database (None for races loaded from config files)
    pub db_pool: Option<Arc<PgPool>>, // Optional database pool for saving events
}

pub struct PitDecision {
    pub pit: bool,
    pub tire: Option<TireType>,
    pub fuel: Option<f32>,
}

fn is_ai_player(player_uuid: &Option<String>) -> bool {
    player_uuid.is_none()
}

pub fn ai_pit_decision(car: Car, track_wetness: f32, total_laps: u32) -> PitDecision {
    // skip if not AI, already pitting or pitted
    if !is_ai_player(&car.player_uuid) || car.pit_request || car.status == CarStatus::Pit {
        return PitDecision {
            pit: false,
            tire: None,
            fuel: None,
        };
    }
    let mut needs_pit = false;
    if car.fuel < 99.0 {
        needs_pit = true;
    }
    let laps_remaining = total_laps.saturating_sub(car.lap);

    // Decide tire type based on track wetness and laps remaining
    let best_tire = if track_wetness > 0.65 {
        TireType::Wet
    } else if track_wetness > 0.2 {
        TireType::Intermediate
    } else {
        // Dry tire strategy: harder compounds for more laps left, softer for fewer laps left
        if laps_remaining > 12 {
            TireType::Hard
        } else if laps_remaining > 6 {
            TireType::Medium
        } else {
            TireType::Soft
        }
    };
    // check if we need to change tire because of track condition change
    if (matches!(best_tire, TireType::Intermediate | TireType::Wet)
        && matches!(
            car.tire.type_,
            TireType::Soft | TireType::Medium | TireType::Hard
        ))
        || (matches!(car.tire.type_, TireType::Intermediate | TireType::Wet)
            && matches!(
                best_tire,
                TireType::Soft | TireType::Medium | TireType::Hard
            ))
    {
        needs_pit = true;
    }
    if needs_pit {
        return PitDecision {
            pit: true,
            tire: Some(best_tire.clone()),
            fuel: Some(100.0),
        };
    } else {
        return PitDecision {
            pit: false,
            tire: None,
            fuel: None,
        };
    }
}

pub fn create_event(
    event_id: u16,
    time: f32,
    event_type: EventType,
    description: String,
    car: Option<&Car>,
) -> Event {
    let event_data = EventData {
        car_number: car.map(|c| c.number),
        car_id: car.map(|c| c.uid),
        team_name: car.map(|c| c.team.name.clone()),
        team_id: car.map(|c| c.team.uid),
        driver_id: car.map(|c| c.driver.uid),
        driver_name: car.map(|c| c.driver.name.clone()),
        tire: car.and_then(|c| Some(format!("{:?}", c.tire.type_))),
        fuel: car.and_then(|c| Some(c.fuel)),
        weather: None,
        time_offset_seconds: time,
    };

    Event {
        id: event_id,
        description,
        event_type,
        data: event_data,
    }
}

/// Convert EventType enum to database string representation
fn event_type_to_db_string(event_type: &EventType) -> String {
    match event_type {
        EventType::StartRace => "START_RACE".to_string(),
        EventType::EndRace => "END_RACE".to_string(),
        EventType::PitRequest => "PIT_REQUEST".to_string(),
        EventType::PitCancel => "PIT_CANCEL".to_string(),
        EventType::PitStop => "PIT_STOP".to_string(),
        EventType::WeatherChange => "WEATHER_CHANGE".to_string(),
        EventType::Accident => "ACCIDENT".to_string(),
        EventType::CarFinished => "CAR_FINISHED".to_string(),
        EventType::Dnf => "DNF".to_string(),
        EventType::Other => "OTHER".to_string(),
    }
}

/// Convert an Event to CreateEventRequest for database storage
fn event_to_create_request(event: &Event, race_id: Uuid) -> CreateEventRequest {
    CreateEventRequest {
        race_id,
        event_type: event_type_to_db_string(&event.event_type),
        description: event.description.clone(),
        time_offset_seconds: event.data.time_offset_seconds,
        car_number: event.data.car_number.map(|n| n as i32),
        car_id: event.data.car_id,
        team_id: event.data.team_id,
        driver_id: event.data.driver_id,
        tire: event.data.tire.clone(),
        fuel: event.data.fuel,
    }
}

/// Save an event to the database asynchronously
/// This function spawns a tokio task to save the event without blocking
fn save_event_to_db(pool: Arc<PgPool>, event: Event, race_id: Uuid) {
    let request = event_to_create_request(&event, race_id);
    tokio::spawn(async move {
        if let Err(e) = tdb::create_event(&pool, request).await {
            eprintln!("Failed to save event to database: {}", e);
        }
    });
}

impl RaceState {
    /// Set the database pool for saving events
    pub fn set_db_pool(&mut self, pool: Arc<PgPool>) {
        self.db_pool = Some(pool);
    }

    /// Register a new event in the race state and optionally save it to the database
    /// This method adds the event to the in-memory events vector and saves it to DB if:
    /// - A database pool is configured
    /// - A race_id is set (race is from database)
    pub fn register_event(
        &mut self,
        event_type: EventType,
        description: String,
        car: Option<&Car>,
    ) {
        let event_id = self.events.len() as u16;

        let tire_str = if let Some(c) = car {
            // For pit requests, use target_tire if available, otherwise current tire
            c.target_tire
                .as_ref()
                .map(|t| format!("{:?}", t))
                .or_else(|| Some(format!("{:?}", c.tire.type_)))
        } else {
            None
        };

        let event_data = EventData {
            car_number: car.map(|c| c.number),
            car_id: car.map(|c| c.uid),
            team_name: car.map(|c| c.team.name.clone()),
            team_id: car.map(|c| c.team.uid),
            driver_name: car.map(|c| c.driver.name.clone()),
            driver_id: car.map(|c| c.driver.uid),
            tire: tire_str,
            fuel: car.and_then(|c| c.target_fuel),
            weather: None,
            time_offset_seconds: self.tick_count as f32 * self.tick_duration_seconds,
        };

        let event = Event {
            id: event_id,
            description,
            event_type,
            data: event_data,
        };

        // Clone data needed for DB save before pushing (to avoid borrow issues)
        let db_pool = self.db_pool.clone();
        let race_id = self.race_id;

        // Push event first
        self.events.push(event.clone());

        // Save to database if pool and race_id are available (after push to avoid borrow conflicts)
        if let (Some(pool), Some(race_id)) = (db_pool, race_id) {
            save_event_to_db(pool, event, race_id);
        }
    }

    /// Create an empty race state (no race loaded)
    /// This is used when the server starts without a pre-loaded race.
    /// Races should be started from scheduled race items via the API.
    pub fn empty() -> Self {
        use crate::models::weather::Weather;
        Self {
            track: Track {
                uid: None,
                id: "".to_string(),
                name: "No race loaded".to_string(),
                laps: 0,
                lap_length_km: 0.0,
                sampled_track: Vec::new(),
                weather: Weather {
                    state_change_time: vec![(0.0, 0.0)],
                },
                wetness: 0.0,
            },
            cars: HashMap::new(),
            run_state: RaceRunState::Paused,
            tick_count: 0,
            tick_duration_seconds: 0.1,
            events: Vec::new(),
            race_id: None,
            db_pool: None,
        }
    }

    pub fn load_race_config(config_path: &str) -> Result<RaceState, io::Error> {
        let mut cars = HashMap::new();
        let mut rng = rand::rng();

        let config = read_race_config(config_path).unwrap();
        // Derive assets directory from config path
        // If config_path is /app/assets/race.json, assets_dir will be /app/assets
        let assets_dir = std::path::Path::new(config_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/app/assets");
        let track_folder = format!("{}/tracks/{}", assets_dir, config.track.name);
        let mut track = Track::load_track_config(&track_folder).unwrap();
        track.laps = config.track.laps;

        let mut car_number = 1;

        for team_data in config.teams.iter() {
            for (driver, car_stats) in vec![
                (&team_data.driver_1, &team_data.car_1),
                (&team_data.driver_2, &team_data.car_2),
            ] {
                let car = Car {
                    uid: Uuid::new_v4(),
                    number: car_number,
                    team: team_data.data.clone(),
                    driver: driver.clone(),
                    stats: car_stats.clone(),
                    tire: Tire {
                        type_: TireType::Medium,
                        wear: 0.0,
                    },
                    fuel: 100.0,
                    driving_style: DrivingStyle::Normal,
                    status: CarStatus::Racing,
                    race_position: car_number, // Initial placeholder
                    lap: 0,
                    lap_percentage: 0.0,
                    total_distance: 0.0,
                    finished_time: 0,
                    speed: 0.0,
                    base_performance: rng.random_range(0.9..1.1),
                    pit_request: false,
                    target_tire: None,
                    target_fuel: None,
                    pit_time_remaining: 0,
                    player_uuid: team_data.player_uuid.clone(),
                };
                cars.insert(car_number, car);
                car_number += 1;
            }
        }

        Ok(RaceState {
            track,
            cars,
            run_state: RaceRunState::Paused, // Start paused
            tick_count: 0,
            tick_duration_seconds: 0.1, // 100ms
            events: Vec::new(),
            race_id: None, // Races loaded from config don't have a database ID
            db_pool: None,
        })
    }

    // Helper function to process a team and add its cars to the race
    async fn process_team_for_race(
        pool: &PgPool,
        team_id: Uuid,
        cars: &mut HashMap<u32, Car>,
        mut car_number: u32,
    ) -> Result<u32, io::Error> {
        // Load the team
        let team_db = tdb::get_team_by_id(pool, team_id)
            .await
            .map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Failed to load team: {}", e))
            })?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Team not found"))?;

        // Convert TeamDb to Team
        let team = Team {
            uid: team_db.id,
            number: team_db.number as u32,
            name: team_db.name,
            logo: team_db.logo,
            color: team_db.color,
            pit_efficiency: team_db.pit_efficiency,
        };

        // Load cars for this team
        let cars_db = tdb::list_cars_by_team(pool, team_id).await.map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to load cars: {}", e))
        })?;

        // For each car, load its driver and create a Car
        for car_db in cars_db {
            // Load the driver for this car (query by car_id)
            let driver_db = sqlx::query_as::<_, DriverDb>("SELECT * FROM driver WHERE car_id = $1")
                .bind(car_db.id)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to load driver: {}", e),
                    )
                })?
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Driver not found for car {}", car_db.number),
                    )
                })?;

            // Convert DriverDb to Driver
            let driver = Driver {
                uid: driver_db.id,
                name: format!("{} {}", driver_db.first_name, driver_db.last_name),
                skill_level: driver_db.skill_level,
                stamina: driver_db.stamina,
                weather_tolerance: driver_db.weather_tolerance,
                experience: driver_db.experience,
                consistency: driver_db.consistency,
                focus: driver_db.focus,
                stress_level: 0.0, // Initialize stress level to 0 at race start
            };

            // Convert CarDb stats to CarStats
            let car_stats = CarStats {
                handling: car_db.handling,
                acceleration: car_db.acceleration,
                top_speed: car_db.top_speed,
                reliability: car_db.reliability,
                fuel_consumption: car_db.fuel_consumption,
                tire_wear: car_db.tire_wear,
            };

            // Create Car
            let car = Car {
                uid: car_db.id,
                number: car_number,
                team: team.clone(),
                driver,
                stats: car_stats,
                tire: Tire {
                    type_: TireType::Medium,
                    wear: 0.0,
                },
                fuel: 100.0,
                driving_style: DrivingStyle::Normal,
                status: CarStatus::Racing,
                race_position: car_number,
                lap: 0,
                lap_percentage: 0.0,
                total_distance: 0.0,
                finished_time: 0,
                speed: 0.0,
                base_performance: car_db.base_performance,
                pit_request: false,
                target_tire: None,
                target_fuel: None,
                pit_time_remaining: 0,
                player_uuid: team_db.player_id.map(|id| id.to_string()),
            };

            cars.insert(car_number, car);
            car_number += 1;
        }

        Ok(car_number)
    }

    // Load a scheduled race from the database
    // load the teams from the registration table for this race
    // load the track from the track table for this race
    // load the cars from the cars of the teams of this race
    // load the drivers from the drivers associated to the cars of this race
    // load the number of laps for this race.
    pub async fn load_scheduled_race(pool: &PgPool, race_id: Uuid) -> Result<RaceState, io::Error> {
        // Load the race from the database
        let race_db = tdb::get_race_by_id(pool, race_id)
            .await
            .map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Failed to load race: {}", e))
            })?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Race not found"))?;

        // Load the track from the database
        let track_db = tdb::get_track_by_id(pool, race_db.track_id)
            .await
            .map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Failed to load track: {}", e))
            })?
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Track not found"))?;

        // Load track configuration from files (using track_id)
        // Try multiple possible asset directory locations
        let assets_dir = if let Ok(dir) = std::env::var("ASSETS_DIR") {
            dir
        } else {
            // Try relative path first (for development), then absolute path (for production)
            if std::path::Path::new("./assets").exists() {
                "./assets".to_string()
            } else if std::path::Path::new("../assets").exists() {
                "../assets".to_string()
            } else {
                "/app/assets".to_string()
            }
        };

        let track_folder = format!("{}/tracks/{}", assets_dir, track_db.track_id);

        // Try to find the track folder by checking multiple possible locations
        let track_folder_path = if std::path::Path::new(&track_folder).exists() {
            track_folder
        } else {
            // Try alternative paths
            let alternatives = vec![
                format!("./assets/tracks/{}", track_db.track_id),
                format!("../assets/tracks/{}", track_db.track_id),
                format!("assets/tracks/{}", track_db.track_id),
            ];

            alternatives
                .iter()
                .find(|path| std::path::Path::new(path).exists())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!(
                            "Track folder not found for track_id '{}'. Tried: {}, {}, {}, and {}. Please ensure track files exist.",
                            track_db.track_id,
                            track_folder,
                            alternatives[0],
                            alternatives[1],
                            alternatives[2]
                        ),
                    )
                })?
                .clone()
        };

        let mut track = Track::load_track_config(&track_folder_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to load track configuration from {}: {}",
                    track_folder_path, e
                ),
            )
        })?;
        track.laps = race_db.laps as u32;

        // Load registrations for this race
        let registrations = tdb::list_registrations_by_race(pool, race_id)
            .await
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to load registrations: {}", e),
                )
            })?;

        // Collect registered team IDs to exclude them when filling with AI teams
        let registered_team_ids: HashSet<Uuid> = registrations.iter().map(|r| r.team_id).collect();

        let mut cars = HashMap::new();
        let mut car_number = 1;

        // Process registered teams
        for registration in &registrations {
            car_number =
                Self::process_team_for_race(pool, registration.team_id, &mut cars, car_number)
                    .await?;
        }

        // If we have fewer than MAX_PARTICIPANTS teams, fill with AI teams (player_id IS NULL)
        let registered_count = registered_team_ids.len() as i64;
        if registered_count < MAX_PARTICIPANTS {
            let needed = MAX_PARTICIPANTS - registered_count;

            // Query for teams where player_id IS NULL and not already registered
            let ai_teams = tdb::list_ai_teams_not_registered_for_race(pool, race_id, needed)
                .await
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to load AI teams: {}", e),
                    )
                })?;

            // Process AI teams
            for ai_team in ai_teams {
                car_number =
                    Self::process_team_for_race(pool, ai_team.id, &mut cars, car_number).await?;
            }
        }

        Ok(RaceState {
            track,
            cars,
            run_state: RaceRunState::Paused, // Start paused
            tick_count: 0,
            tick_duration_seconds: 0.1, // 100ms
            events: Vec::new(),
            race_id: Some(race_id), // Store the race ID for scheduled races
            db_pool: None,
        })
    }

    pub fn new(track: Track) -> Self {
        let mut cars = HashMap::new();
        let mut teams = HashMap::new();
        let mut rng = rand::rng();

        let team_names = ["Red Bull", "Ferrari", "Mercedes", "McLaren", "Alpine"];
        let drivers = load_drivers_from_json("./assets/drivers.json");
        if drivers.len() < team_names.len() * 2 {
            panic!(
                "Not enough drivers in the JSON file to fill the teams ({} < {}*2)",
                drivers.len(),
                team_names.len()
            );
        }

        for i in 0..5 {
            let team = Team {
                uid: Uuid::new_v4(),
                number: (i + 1) as u32,
                name: team_names[i as usize].to_string(),
                logo: format!("team_{}.png", i + 1),
                color: "#ea02a4ff".to_string(),
                pit_efficiency: 0.5,
            };
            teams.insert(team.number, team.clone());

            for j in 0..2 {
                let car_index = i * 2 + j;
                let car_number = (car_index + 1) as u32;

                // Use Driver::new to create the driver
                let driver = drivers[car_index as usize].clone();

                let car = Car {
                    uid: Uuid::new_v4(),
                    number: car_number,
                    team: team.clone(),
                    driver,
                    stats: CarStats::new(),
                    tire: Tire {
                        type_: TireType::Medium,
                        wear: 0.0,
                    },
                    fuel: 100.0,
                    driving_style: DrivingStyle::Normal,
                    status: CarStatus::Racing,
                    race_position: car_number, // Initial placeholder
                    lap: 0,
                    lap_percentage: 0.0,
                    total_distance: 0.0,
                    finished_time: 0,
                    speed: 0.0,
                    base_performance: rng.random_range(0.9..1.1),
                    pit_request: false,
                    target_tire: None,
                    target_fuel: None,
                    pit_time_remaining: 0,
                    player_uuid: None,
                };
                cars.insert(car_number, car);
            }
        }

        RaceState {
            track,
            cars,
            run_state: RaceRunState::Paused, // Start paused
            tick_count: 0,
            tick_duration_seconds: 0.1, // 100ms
            events: Vec::new(),
            race_id: None, // Races created with new() don't have a database ID
            db_pool: None,
        }
    }

    // Placeholder function to get client view (will be refined later)
    pub fn get_client_view(&self) -> RaceStateClientView {
        let mut car_data: Vec<CarClientData> = self
            .cars
            .values()
            .map(|car| {
                let track_position = car.lap as f32 + car.lap_percentage;
                CarClientData {
                    car_number: car.number,
                    driver: car.driver.clone(),
                    carstats: car.stats.clone(),
                    team: car.team.clone(),
                    race_position: car.race_position,
                    track_position: track_position, // Combined lap.percentage
                    status: car.status.clone(),
                    tire: ClientTireData {
                        type_: car.tire.type_.clone(),
                        wear: car.tire.wear,
                    },
                    fuel: car.fuel,
                    driving_style: car.driving_style.clone(),
                    speed: car.speed, // Use the speed from Car struct
                    finished_time: car.finished_time,
                    player_uuid: car.player_uuid.clone(),
                    pit_requested: Some(car.pit_request),
                }
            })
            .collect();

        // Sort cars by race position (ascending)
        car_data.sort_by_key(|c| c.race_position);

        RaceStateClientView {
            track: TrackClientData::new(
                &self.track,
                self.tick_count as f32 * self.tick_duration_seconds,
            ),
            cars: car_data,
            current_lap: self.cars.values().map(|c| c.lap).max().unwrap_or(0), // Leader's lap
            total_laps: self.track.laps,
            race_status: self.run_state.clone(),
        }
    }

    pub fn update_weather(&mut self) {
        // update weather
        let rain_chance = self
            .track
            .weather
            .get_state_at_time(self.tick_count as f32 * self.tick_duration_seconds);

        let wetness_change = if rain_chance > 0.66 {
            // Increasing wetness (raining)
            // At 1.0: increase by 1 in 3 minutes (180 seconds)
            // At 0.66: increase by 1 in 10 minutes (600 seconds)
            // Linear interpolation between 0.66 and 1.0
            let rate_at_100 = 1.0 / 180.0; // per second
            let rate_at_66 = 1.0 / 600.0; // per second
            let interpolation_factor = (rain_chance - 0.66) / (1.0 - 0.66);
            let rate = rate_at_66 + (rate_at_100 - rate_at_66) * interpolation_factor;
            rate * self.tick_duration_seconds
        } else if rain_chance < 0.5 {
            // Decreasing wetness (drying)
            // At 0.5: decrease by 1 in 10 minutes (600 seconds)
            // At 0.0: decrease by 1 in 1 minute (60 seconds)
            // Linear interpolation between 0.0 and 0.5
            let rate_at_0 = -1.0 / 60.0; // per second (negative for decrease)
            let rate_at_50 = -1.0 / 600.0; // per second (negative for decrease)
            let interpolation_factor = rain_chance / 0.5;
            let rate = rate_at_0 + (rate_at_50 - rate_at_0) * interpolation_factor;
            rate * self.tick_duration_seconds
        } else {
            // Between 0.5 and 0.66: stable, no change
            0.0
        };

        // Apply wetness change and clamp between 0.0 and 1.0
        self.track.wetness += wetness_change;
        self.track.wetness = self.track.wetness.clamp(0.0, 1.0);
    }

    pub fn update(&mut self) {
        if self.run_state != RaceRunState::Running && self.run_state != RaceRunState::LastLap {
            return; // Don't update if paused or finished
        }

        self.tick_count += 1;

        self.update_weather();

        let mut positions: Vec<&Car> = Vec::new(); // vector of references to cars
        let number_finished = self
            .cars
            .values()
            .filter(|c| c.status == CarStatus::Finished)
            .count();

        for car in self.cars.values_mut() {
            if car.status == CarStatus::Dnf || car.status == CarStatus::Finished {
                if car.status == CarStatus::Finished {
                    car.total_distance = car.lap as f32 * self.track.lap_length_km;
                }
                positions.push(car);
                continue; // Skip DNF cars entirely
            }

            // --- Handle Pit Stop Logic ---
            if car.status == CarStatus::Pit {
                car.speed = 30.0;
                if car.pit_time_remaining > 0 {
                    car.pit_time_remaining -= 1;
                } else {
                    // Pit stop complete: Apply changes
                    if let Some(new_tire_type) = car.target_tire.take() {
                        car.tire.type_ = new_tire_type;
                        car.tire.wear = 0.0; // Fresh tires
                                             // println!("Car {} fitted with {:?} tires.", car.number, car.tire.type_);
                    }
                    if let Some(new_fuel_level) = car.target_fuel.take() {
                        car.fuel = new_fuel_level.min(100.0).max(car.fuel); // Clamp fuel level [previous level-100]
                    }
                    car.status = CarStatus::Racing; // Back to racing
                                                    // println!("Car {} exits the pits.", car.number);
                }
                positions.push(car);
                continue; // Skip normal updates while pitting
            }

            // --- Handle AI input ---
            let decision = ai_pit_decision(car.clone(), self.track.wetness, self.track.laps);
            if decision.pit == true {
                let was_requested = car.pit_request;
                car.pit_request = true;
                car.target_fuel = decision.fuel;
                car.target_tire = decision.tire.clone();

                // Register PitRequest event if this is a new request
                if !was_requested {
                    let tire_str = decision
                        .tire
                        .as_ref()
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|| "No change".to_string());
                    let fuel_str = decision
                        .fuel
                        .map(|f| format!("{:.0}", f))
                        .unwrap_or_else(|| "No refuel".to_string());
                    let description = format!(
                        "Car {} (AI) requests pit stop: {} tires, {} fuel",
                        car.number, tire_str, fuel_str
                    );

                    let event = create_event(
                        self.events.len() as u16,
                        self.tick_count as f32 * self.tick_duration_seconds,
                        EventType::PitRequest,
                        description,
                        Some(&car),
                    );
                    // Save to database if pool and race_id are available
                    if let (Some(pool), Some(race_id)) = (self.db_pool.clone(), self.race_id) {
                        save_event_to_db(pool, event.clone(), race_id);
                    }
                    self.events.push(event);
                }
            }

            // --- Calculate Performance Factors (Only if Racing) ---
            let mut max_speed = car.max_speed();
            // Base performance
            //let mut performance_multiplier = car.base_performance;

            // Driver skill
            //performance_multiplier *= car.driver.skill_level;

            // Driving style
            //match car.driving_style {
            //    DrivingStyle::Aggressive => performance_multiplier *= 1.05,
            //    DrivingStyle::Relax => performance_multiplier *= 0.95,
            //    DrivingStyle::Normal => (), // No change
            //}

            // Tire grip/wear (simple model: linear drop-off)
            // TODO: More sophisticated tire model based on type
            //let grip_factor = 1.0 - (car.tire.wear / 150.0); // Example: Grip drops linearly, fully gone at 150% wear
            //performance_multiplier *= grip_factor.max(0.1); // Ensure minimum grip

            // Fuel weight (simple model: linear effect)
            //let fuel_weight_penalty = car.fuel / 2000.0; // Example: 0.05 penalty at 100% fuel
            //performance_multiplier *= 1.0 - fuel_weight_penalty;

            // slow down on corners
            let track_point = self.track.get_track_point_at_distance(car.lap_percentage);
            let curvature = track_point.curvature;
            // curvature goes from 0 to ~0.3 (angle in radians).
            // we want to map 0.3 and above to a speed decrease of 75%, 0.0 is a speed decrease of 0%, between is exponential
            let curvature_factor = (-4.62 * curvature).exp().max(0.15);
            if car.number == 1 {
                //println!("at lap_percentage: {}, curvature: {}, curvature_factor: {}", car.lap_percentage, curvature, curvature_factor);
            }
            max_speed *= curvature_factor;

            // ramp up to max speed
            car.speed = max_speed.min(car.speed + car.acceleration());

            // Random events (placeholder)
            // TODO: Implement failure risk, especially for aggressive style

            // --- Update State (Only if Racing) ---
            // Calculate distance covered this tick (adjust speed based on time_scale)

            let distance_km = (car.speed / 3600.0) * self.tick_duration_seconds;

            let distance_laps = distance_km / self.track.lap_length_km;
            car.lap_percentage += distance_laps;

            // Calculate live speed in km/h (converting from distance per tick to km/h)
            // car.speed = (base_speed_kph * performance_multiplier)
            //     * grip_factor
            //     * (1.0 - fuel_weight_penalty);

            // Handle lap completion
            while car.lap_percentage >= 1.0 {
                car.lap += 1;
                car.lap_percentage -= 1.0;
                if self.run_state == RaceRunState::LastLap {
                    car.lap_percentage = 0.0;
                    car.status = CarStatus::Finished;
                    car.finished_time = self.tick_count;
                    let event = create_event(
                        self.events.len() as u16,
                        self.tick_count as f32 * self.tick_duration_seconds,
                        EventType::CarFinished,
                        format!(
                            "Car {} finished the race in position {}.",
                            car.number,
                            number_finished + 1
                        ),
                        Some(&car),
                    );
                    // Save to database if pool and race_id are available
                    if let (Some(pool), Some(race_id)) = (self.db_pool.clone(), self.race_id) {
                        save_event_to_db(pool, event.clone(), race_id);
                    }
                    self.events.push(event);
                }

                // Check for pit stop request at lap boundary
                if car.pit_request && car.lap < self.track.laps {
                    car.status = CarStatus::Pit;
                    car.lap_percentage = 0.0001; // 1% of the next lap, prevent passing in pit
                    car.pit_request = false;
                    car.pit_time_remaining = 50;

                    // Register PitStop event
                    let tire_str = car
                        .target_tire
                        .as_ref()
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|| "No change".to_string());
                    let fuel_str = car
                        .target_fuel
                        .map(|f| format!("{:.0}", f))
                        .unwrap_or_else(|| "No refuel".to_string());
                    let description = format!(
                        "Car {} enters pit stop: {} tires, {} fuel",
                        car.number, tire_str, fuel_str
                    );

                    let event = create_event(
                        self.events.len() as u16,
                        self.tick_count as f32 * self.tick_duration_seconds,
                        EventType::PitStop,
                        description,
                        Some(&car),
                    );
                    // Save to database if pool and race_id are available
                    if let (Some(pool), Some(race_id)) = (self.db_pool.clone(), self.race_id) {
                        save_event_to_db(pool, event.clone(), race_id);
                    }
                    self.events.push(event);

                    // println!("Car {} entering pits (Duration: {} ticks).", car.number, car.pit_time_remaining);
                    continue;
                }
            }

            // Update fuel consumption based on car stats
            // Base consumption rate (0.0 to 1.0 fuel_consumption stat maps to 0.0005 to 0.002 per second at max speed)
            let base_fuel_rate = 0.0005 + (car.stats.fuel_consumption * 0.15);
            // Scale by current speed relative to max speed
            let max_speed = car.max_speed();
            let speed_factor = if max_speed > 0.0 {
                car.speed / max_speed
            } else {
                0.0
            };
            let fuel_consumption_rate = base_fuel_rate * speed_factor.max(0.0);
            car.fuel -= fuel_consumption_rate * self.tick_duration_seconds as f32;
            car.fuel = car.fuel.max(0.0);
            if car.fuel == 0.0 && car.status == CarStatus::Racing {
                // println!("Car {} ran out of fuel!", car.number);
                car.status = CarStatus::Dnf;
                car.finished_time = self.tick_count;
                let event = create_event(
                    self.events.len() as u16,
                    self.tick_count as f32 * self.tick_duration_seconds,
                    EventType::Dnf,
                    format!("Car {} ran out of fuel!", car.number),
                    Some(&car),
                );
                // Save to database if pool and race_id are available
                if let (Some(pool), Some(race_id)) = (self.db_pool.clone(), self.race_id) {
                    save_event_to_db(pool, event.clone(), race_id);
                }
                self.events.push(event);
            }

            // Update tire wear based on car stats
            // Base wear rate (0.0 to 1.0 tire_wear stat maps to 0.0002 to 0.001 per second at max speed)
            let base_tire_wear_rate = 0.0002 + (car.stats.tire_wear * 0.08);
            // Scale by current speed relative to max speed (reuse speed_factor from above)
            let tire_wear_rate = base_tire_wear_rate * speed_factor.max(0.0);
            // Different wear rates for different tire types
            let tire_type_wear_multiplier = match car.tire.type_ {
                TireType::Soft => 1.5,   // Soft tires wear faster
                TireType::Medium => 1.0, // Medium is baseline
                TireType::Hard => 0.7,   // Hard tires wear slower
                TireType::Intermediate => 1.2,
                TireType::Wet => 1.3,
            };
            car.tire.wear +=
                tire_wear_rate * tire_type_wear_multiplier * self.tick_duration_seconds as f32;
            car.tire.wear = car.tire.wear.min(100.0); // Cap at 100%?
                                                      // TODO: Consider tire failure above certain wear

            // Update driver stress level based on time and driving style
            // Base stress increase from race time (increases over time)
            let stress_change = match car.driving_style {
                DrivingStyle::Aggressive => {
                    // Aggressive driving increases stress faster
                    0.03 * (1.0 - car.driver.focus) * self.tick_duration_seconds
                }
                DrivingStyle::Normal => {
                    // Normal driving: stress increases slowly, but can decrease slightly
                    // Net effect: slow decrease (stress increase - small decrease)
                    -0.005 * car.driver.focus * self.tick_duration_seconds
                }
                DrivingStyle::Relax => {
                    // Relaxed driving: stress decreases faster
                    // Net effect: faster decrease (stress increase - larger decrease)
                    -0.015 * car.driver.focus * self.tick_duration_seconds
                }
            };
            car.driver.stress_level += stress_change;
            // Clamp stress level between 0.0 and 1.0
            car.driver.stress_level = car.driver.stress_level.clamp(0.0, 1.0);

            // Store total distance for position calculation
            if car.status == CarStatus::Racing || car.status == CarStatus::Pit {
                // Only include racing cars for positions
                let total_distance =
                    (car.lap as f32 + car.lap_percentage) * self.track.lap_length_km;
                car.total_distance = total_distance;
            }
            positions.push(car);
        }

        // --- Update Race Positions ---
        // Sort cars by:
        // - Finished cars by finished_time ascending
        // - Racing cars by total distance descending
        // - DNF cars by finished_time descending
        positions.sort_by(|a, b| compare_cars(a, b));

        // Assign race positions
        let car_numbers: Vec<u32> = positions.iter().map(|car| car.number).collect();
        for (index, car_number) in car_numbers.iter().enumerate() {
            if let Some(car) = self.cars.get_mut(car_number) {
                car.race_position = (index + 1) as u32;
            }
        }

        // Check if all cars are finished or DNF
        update_race_finished(self);
    }
}

fn compare_cars(a: &Car, b: &Car) -> std::cmp::Ordering {
    match (a.status, b.status) {
        // Both finished - sort by finished time ascending
        (CarStatus::Finished, CarStatus::Finished) => {
            if a.lap > b.lap {
                std::cmp::Ordering::Less
            } else if a.lap < b.lap {
                std::cmp::Ordering::Greater
            } else {
                a.finished_time.cmp(&b.finished_time)
            }
        }
        // Both DNF - sort by total distance descending
        (CarStatus::Dnf, CarStatus::Dnf) => b
            .total_distance
            .partial_cmp(&a.total_distance)
            .unwrap_or(std::cmp::Ordering::Equal),
        // Finished cars come before others
        //(CarStatus::Finished, _) => std::cmp::Ordering::Less,
        //(_, CarStatus::Finished) => std::cmp::Ordering::Greater,
        // Non-DNF cars come before DNF
        (_, CarStatus::Dnf) => std::cmp::Ordering::Less,
        (CarStatus::Dnf, _) => std::cmp::Ordering::Greater,
        // Both racing or pit - sort by total distance descending
        (_, _) => b
            .total_distance
            .partial_cmp(&a.total_distance)
            .unwrap_or(std::cmp::Ordering::Equal),
    }
}

/// Updates the race state to check if the race is finished or in its final lap
///
/// # Arguments
/// * `state` - Mutable reference to the current RaceState
///
/// This function checks all cars to determine if:
/// - Any cars have completed all laps (sets their status to Finished)
/// - All cars have either finished or DNF'd (sets race state to Finished)
/// - Some cars finished but others still racing (sets race state to LastLap)
fn update_race_finished(state: &mut RaceState) {
    let mut race_finished = true;
    let mut someone_finished = false;
    let mut tot_done = 0;
    let number_finished = state
        .cars
        .values()
        .filter(|c| c.status == CarStatus::Finished)
        .count();
    for car in state.cars.values_mut() {
        if car.status == CarStatus::Finished {
            someone_finished = true;
            tot_done += 1;
        } else if car.lap >= state.track.laps {
            car.status = CarStatus::Finished;
            car.total_distance = car.lap as f32 * state.track.lap_length_km;
            car.finished_time = state.tick_count;
            someone_finished = true;
            tot_done += 1;
            let event = create_event(
                state.events.len() as u16,
                state.tick_count as f32 * state.tick_duration_seconds,
                EventType::CarFinished,
                format!(
                    "Car {} finished the race in position {}.",
                    car.number,
                    number_finished + 1
                ),
                Some(&car),
            );
            // Save to database if pool and race_id are available
            if let (Some(pool), Some(race_id)) = (state.db_pool.clone(), state.race_id) {
                save_event_to_db(pool, event.clone(), race_id);
            }
            state.events.push(event);
        } else if car.status == CarStatus::Racing || car.status == CarStatus::Pit {
            race_finished = false;
        } else if car.status == CarStatus::Dnf {
            tot_done += 1;
        }
    }
    if tot_done == state.cars.len() {
        state.run_state = RaceRunState::Finished;
    } else if someone_finished {
        if race_finished {
            state.run_state = RaceRunState::Finished;
        } else {
            state.run_state = RaceRunState::LastLap;
        }
    }
}

fn load_drivers_from_json(file_path: &str) -> Vec<Driver> {
    let drivers_json = std::fs::read_to_string(file_path).expect("Failed to read drivers.json");
    let drivers_data: serde_json::Value =
        serde_json::from_str(&drivers_json).expect("Failed to parse drivers.json");
    let drivers = drivers_data
        .as_array()
        .expect("Expected drivers to be an array");
    drivers
        .iter()
        .map(|v| Driver::new(&serde_json::to_string(v).unwrap()))
        .collect::<Vec<_>>()
}

#[derive(Debug, Deserialize)]
struct TrackConfig {
    name: String,
    laps: u32,
}

#[derive(Debug, Deserialize)]
struct TeamConfig {
    data: Team,
    driver_1: Driver,
    driver_2: Driver,
    car_1: CarStats,
    car_2: CarStats,
    player_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RaceConfig {
    track: TrackConfig,
    teams: Vec<TeamConfig>,
}

fn read_race_config(file_path: &str) -> Result<RaceConfig, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(file_path)
        .expect(format!("Failed to read config file {file_path}").as_str());
    let config: RaceConfig = serde_json::from_str(&data)
        .expect(format!("Failed to parse config file {file_path}").as_str());
    Ok(config)
}
