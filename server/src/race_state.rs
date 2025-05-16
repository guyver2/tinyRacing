use rand::Rng;
use std::collections::HashMap;

use crate::models::*;

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
}

impl RaceState {
    pub fn new(track: Track) -> Self {
        let mut cars = HashMap::new();
        let mut teams = HashMap::new();
        let mut rng = rand::thread_rng();

        let team_names = ["Red Bull", "Ferrari", "Mercedes", "McLaren", "Alpine"];
        let drivers = load_drivers_from_json("./drivers.json");
        if drivers.len() < team_names.len() * 2 {
            panic!(
                "Not enough drivers in the JSON file to fill the teams ({} < {}*2)",
                drivers.len(),
                team_names.len()
            );
        }

        for i in 0..5 {
            let team_number = (i + 1) as u32;
            let team_name_str = team_names[i as usize].to_string();
            teams.insert(
                team_number,
                Team {
                    number: team_number,
                    name: team_name_str.clone(),
                },
            );

            for j in 0..2 {
                let car_index = i * 2 + j;
                let car_number = (car_index + 1) as u32;

                // Use Driver::new to create the driver
                let driver = drivers[car_index as usize].clone();

                let car = Car {
                    number: car_number,
                    team_number,
                    team_name: team_name_str.clone(),
                    driver,
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
                    base_performance: rng.gen_range(0.9..1.1),
                    pit_request: false,
                    target_tire: None,
                    target_fuel: None,
                    pit_time_remaining: 0,
                };
                cars.insert(car_number, car);
            }
        }

        RaceState {
            track,
            cars,
            // teams,
            run_state: RaceRunState::Paused, // Start paused
            tick_count: 0,
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
                    driver: car.driver.name.clone(),
                    team_number: car.team_number,
                    team_name: car.team_name.clone(), // Use stored team_name
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
                }
            })
            .collect();

        // Sort cars by race position (ascending)
        car_data.sort_by_key(|c| c.race_position);

        RaceStateClientView {
            cars: car_data,
            current_lap: self.cars.values().map(|c| c.lap).max().unwrap_or(0), // Leader's lap
            total_laps: self.track.laps,
            race_status: self.run_state.clone(),
        }
    }

    pub fn update(&mut self) {
        if self.run_state != RaceRunState::Running && self.run_state != RaceRunState::LastLap {
            return; // Don't update if paused or finished
        }

        self.tick_count += 1;
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

            // --- Calculate Performance Factors (Only if Racing) ---
            // Base performance
            let mut performance_multiplier = car.base_performance;

            // Driver skill
            performance_multiplier *= car.driver.skill_level;

            // Driving style
            match car.driving_style {
                DrivingStyle::Aggressive => performance_multiplier *= 1.05,
                DrivingStyle::Relax => performance_multiplier *= 0.95,
                DrivingStyle::Normal => (), // No change
            }

            // Tire grip/wear (simple model: linear drop-off)
            // TODO: More sophisticated tire model based on type
            let grip_factor = 1.0 - (car.tire.wear / 150.0); // Example: Grip drops linearly, fully gone at 150% wear
            performance_multiplier *= grip_factor.max(0.1); // Ensure minimum grip

            // Fuel weight (simple model: linear effect)
            let fuel_weight_penalty = car.fuel / 2000.0; // Example: 0.05 penalty at 100% fuel
            performance_multiplier *= 1.0 - fuel_weight_penalty;

            // Random events (placeholder)
            // TODO: Implement failure risk, especially for aggressive style

            // --- Update State (Only if Racing) ---
            // Calculate distance covered this tick (adjust speed based on time_scale)
            let base_speed_kph = 200.0; // Base speed in km/h
            let tick_duration_seconds = 0.1; // 100ms
            let distance_km =
                (base_speed_kph * performance_multiplier / 3600.0) * tick_duration_seconds;

            let distance_laps = distance_km / self.track.lap_length_km;
            car.lap_percentage += distance_laps;

            // Calculate live speed in km/h (converting from distance per tick to km/h)
            car.speed = (base_speed_kph * performance_multiplier)
                * grip_factor
                * (1.0 - fuel_weight_penalty);

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
                    // println!("Car {} entering pits (Duration: {} ticks).", car.number, car.pit_time_remaining);
                    continue;
                }
            }

            // Update fuel consumption (example rate)
            let fuel_consumption_rate = 0.1 * performance_multiplier; // Higher performance uses more fuel
            car.fuel -= fuel_consumption_rate * tick_duration_seconds as f32;
            car.fuel = car.fuel.max(0.0);
            if car.fuel == 0.0 && car.status == CarStatus::Racing {
                // println!("Car {} ran out of fuel!", car.number);
                car.status = CarStatus::Dnf;
                car.finished_time = self.tick_count;
            }

            // Update tire wear (example rate)
            let tire_wear_rate = 0.05 * performance_multiplier; // Higher performance wears tires faster
                                                                // TODO: Different wear rates for different tire types
            car.tire.wear += tire_wear_rate * tick_duration_seconds as f32;
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
