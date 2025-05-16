use crate::models::*;
use crate::race_state::*;
use std::sync::{Arc, Mutex};

/// Command handling module for the race simulation
///
/// This module provides functionality for processing and executing commands that control
/// the race simulation. Commands can modify race state, car behavior, and race flow.
///
/// # Command Types
///
/// - Race control commands: start, pause, stop
/// - Car control commands: order [car_num] [style]
/// - Pit stop commands: pit [car_num]
/// - Status commands: status [car_num]
///
/// # Command Format
///
/// Commands are received as space-separated strings with the following format:
/// `command [arguments...]`
///
/// # Examples
///
/// ```text
/// start           // Start or resume the race
/// pause           // Pause the race
/// stop            // Stop/finish the race
/// order 44 relax  // Set car 44's driving style to relax
/// pit 77 soft refuel 50  // Order car 77 to pit, change to soft tires and refuel 50%
/// ```
///
/// Commands return status messages indicating success or failure of the operation.

// Type alias for the shared state used across threads/tasks
type SharedRaceState = Arc<Mutex<RaceState>>;

// Modified handle_command to return a String result
pub fn handle_command(command_str: String, state: SharedRaceState) -> String {
    let mut state_guard = state.lock().unwrap();
    let parts: Vec<&str> = command_str.trim().split_whitespace().collect();
    let mut result_messages = Vec::new();

    match parts.as_slice() {
        ["start"] => {
            if state_guard.run_state == RaceRunState::Paused {
                state_guard.run_state = RaceRunState::Running;
                result_messages.push("Race started!".to_string());
            } else {
                result_messages.push("Race is already running or finished.".to_string());
            }
        }
        ["pause"] => {
            if state_guard.run_state == RaceRunState::Running {
                state_guard.run_state = RaceRunState::Paused;
                result_messages.push("Race paused.".to_string());
            } else {
                result_messages.push("Race is not running.".to_string());
            }
        }
        ["stop"] => {
            state_guard.run_state = RaceRunState::Finished;
            result_messages.push("Race stopped/finished manually.".to_string());
        }
        ["order", car_num_str, style_str] => {
            if let Ok(car_num) = car_num_str.parse::<u32>() {
                if let Some(car) = state_guard.cars.get_mut(&car_num) {
                    match style_str.to_lowercase().as_str() {
                        "dnf" => {
                            car.status = CarStatus::Dnf;
                            result_messages.push(format!("Car {} set to DNF.", car_num));
                        }
                        "relax" => {
                            car.driving_style = DrivingStyle::Relax;
                            result_messages
                                .push(format!("Car {} driving style set to Relax.", car_num));
                        }
                        "normal" => {
                            car.driving_style = DrivingStyle::Normal;
                            result_messages
                                .push(format!("Car {} driving style set to Normal.", car_num));
                        }
                        "aggressive" => {
                            car.driving_style = DrivingStyle::Aggressive;
                            result_messages
                                .push(format!("Car {} driving style set to Aggressive.", car_num));
                        }
                        _ => result_messages.push(format!(
                            "Invalid driving style: {}. Use relax, normal, or aggressive.",
                            style_str
                        )),
                    }
                } else {
                    result_messages.push(format!("Car number {} not found.", car_num));
                }
            } else {
                result_messages.push(format!("Invalid car number: {}", car_num_str));
            }
        }
        ["pit", car_num_str, tire_str, "refuel", fuel_str] => {
            if let Ok(car_num) = car_num_str.parse::<u32>() {
                if let Some(car) = state_guard.cars.get_mut(&car_num) {
                    let target_tire = match tire_str.to_lowercase().as_str() {
                        "soft" => Some(TireType::Soft),
                        "medium" => Some(TireType::Medium),
                        "hard" => Some(TireType::Hard),
                        "intermediate" | "inter" => Some(TireType::Intermediate),
                        "wet" => Some(TireType::Wet),
                        _ => None,
                    };
                    let target_fuel = fuel_str.parse::<f32>().ok();

                    if target_tire.is_none() {
                        result_messages.push(format!("Invalid target tire type: {}", tire_str));
                        return result_messages.join("\\n");
                    }
                    if target_fuel.is_none()
                        || target_fuel.unwrap() < 0.0
                        || target_fuel.unwrap() > 100.0
                    {
                        result_messages.push(format!(
                            "Invalid target fuel level: {}. Must be 0-100.",
                            fuel_str
                        ));
                        return result_messages.join("\\n");
                    }

                    car.pit_request = true;
                    car.target_tire = target_tire;
                    car.target_fuel = target_fuel;
                    result_messages.push(format!(
                        "Car {} queued for pit stop: Tire -> {:?}, Fuel -> {:?}%",
                        car_num,
                        car.target_tire.as_ref().unwrap(),
                        car.target_fuel.unwrap()
                    ));
                } else {
                    result_messages.push(format!("Car number {} not found.", car_num));
                }
            } else {
                result_messages.push(format!("Invalid car number: {}", car_num_str));
            }
        }
        _ => result_messages.push(format!("Unknown command: {}", command_str.trim())),
    }
    result_messages.join("\\n")
}
