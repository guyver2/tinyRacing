use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self};

use crate::models::car::{Car, CarClientData, CarStats, CarStatus};
use crate::models::driver::{Driver, DrivingStyle};
use crate::models::event::{Event, EventData, EventType};
use crate::models::team::Team;
use crate::models::tire::{ClientTireData, Tire, TireType};
use crate::models::track::Track;
use crate::models::track::TrackClientData;

pub const MAX_PARTICIPANTS: i64 = 5;

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
        team_name: car.map(|c| c.team.name.clone()),
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

impl RaceState {
    /// Register a new event in the race state
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
            team_name: car.map(|c| c.team.name.clone()),
            driver_name: car.map(|c| c.driver.name.clone()),
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

        self.events.push(event);
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

                    // Create event data manually to avoid borrowing conflict
                    let event_id = self.events.len() as u16;
                    let event = create_event(
                        event_id,
                        self.tick_count as f32 * self.tick_duration_seconds,
                        EventType::PitRequest,
                        description,
                        Some(&car),
                    );
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
                }

                // Check for pit stop request at lap boundary
                if car.pit_request && car.lap < self.track.laps {
                    car.status = CarStatus::Pit;
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

                    // Create event data manually to avoid borrowing conflict
                    let event_id = self.events.len() as u16;
                    let event = create_event(
                        event_id,
                        self.tick_count as f32 * self.tick_duration_seconds,
                        EventType::PitStop,
                        description,
                        Some(&car),
                    );
                    self.events.push(event);

                    // println!("Car {} entering pits (Duration: {} ticks).", car.number, car.pit_time_remaining);
                    continue;
                }
            }

            // Update fuel consumption (example rate)
            let fuel_consumption_rate = 0.001 * car.max_speed(); // Higher performance uses more fuel
            car.fuel -= fuel_consumption_rate * self.tick_duration_seconds as f32;
            car.fuel = car.fuel.max(0.0);
            if car.fuel == 0.0 && car.status == CarStatus::Racing {
                // println!("Car {} ran out of fuel!", car.number);
                car.status = CarStatus::Dnf;
                car.finished_time = self.tick_count;
            }

            // Update tire wear (example rate)
            let tire_wear_rate = 0.0005 * car.max_speed(); // Higher performance wears tires faster
                                                           // TODO: Different wear rates for different tire types
            car.tire.wear += tire_wear_rate * self.tick_duration_seconds as f32;
            car.tire.wear = car.tire.wear.min(100.0); // Cap at 100%?
                                                      // TODO: Consider tire failure above certain wear

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
