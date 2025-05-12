use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use warp::ws::Message;
use std::fs;
use std::io;
use rand::Rng;
use tokio::time::{self, Duration};
use futures_util::{SinkExt, StreamExt};
use warp::Filter;
use uuid::Uuid; // For generating unique client IDsuse std::convert::Infallible;

// Added for ncurses UI
use pancurses::*;
use std::thread;
use std::sync::mpsc as std_mpsc;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum TireType {
    Soft,
    Medium,
    Hard,
    Intermediate,
    Wet,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum DrivingStyle {
    Relax,
    Normal,
    Aggressive,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum CarStatus {
    Racing,
    Pit,
    Dnf, // Did Not Finish
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Tire {
    type_: TireType,
    wear: f32, // 0.0 to 100.0 %
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Driver {
    name: String,
    skill_level: f32, // 0.5 to 1.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Car {
    number: u32,
    team_number: u32,
    team_name: String,
    driver: Driver,
    tire: Tire,
    fuel: f32, // 0.0 to 100.0 %
    driving_style: DrivingStyle,
    status: CarStatus,
    race_position: u32, // 1st, 2nd, etc.
    lap: u32,
    lap_percentage: f32, // 0.0 to 1.0
    base_performance: f32, // 0.9 to 1.1 multiplier
    speed: f64, // Current speed in km/h
    // Fields for pit stop planning
    pit_request: bool,
    target_tire: Option<TireType>,
    target_fuel: Option<f32>,
    pit_time_remaining: u32, // Ticks remaining in pit stop
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Team {
    number: u32,
    name: String,
    // Could add team-specific attributes later
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Track {
    name: String,
    laps: u32,
    lap_length_km: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum RaceRunState {
    Paused,
    Running,
    Finished,
}

#[derive(Serialize, Debug, Clone)] // Only Serialize for sending to clients
struct CarClientData {
    car_number: u32,
    driver: String, // Just the name for the client
    team_number: u32,    // Team number
    team_name: String, // Added for UI
    race_position: u32,
    track_position: f64, // Combined lap.percentage
    status: CarStatus,
    tire: ClientTireData,
    fuel: f32,
    driving_style: DrivingStyle,
    speed: f64, // Current speed in km/h
}

#[derive(Serialize, Debug, Clone)]
struct ClientTireData {
    #[serde(rename = "type")]
    type_: TireType,
    wear: f32,
}

#[derive(Serialize, Debug, Clone)] // Only Serialize for sending to clients
struct RaceStateClientView {
    cars: Vec<CarClientData>,
    current_lap: u32, // Max lap across all cars? Or based on leader?
    total_laps: u32,
    race_status: RaceRunState,
}

#[derive(Debug, Clone)]
struct RaceState {
    track: Track,
    cars: HashMap<u32, Car>, // Keyed by car number
    // teams: HashMap<u32, Team>, // Keyed by team number
    run_state: RaceRunState,
    tick_count: u64,
}

impl RaceState {
    fn new(track: Track) -> Self {
        let mut cars = HashMap::new();
        let mut teams = HashMap::new();
        let mut rng = rand::thread_rng();

        let team_names = ["Red Bull", "Ferrari", "Mercedes", "McLaren", "Alpine"];
        let driver_names = [
            "Max Verstappen", "Yuki Tsunoda", // Red Bull
            "Charles Leclerc", "Lewis Hamilton", // Ferrari
            "George Russell", "Kimi Antonelli", // Mercedes
            "Lando Norris", "Oscar Piastri", // McLaren
            "Pierre Gasly", "Isack Hadjar", // Alpine
        ];

        for i in 0..5 {
            let team_number = (i + 1) as u32;
            let team_name_str = team_names[i as usize].to_string();
            teams.insert(team_number, Team { number: team_number, name: team_name_str.clone() });

            for j in 0..2 {
                let car_index = i * 2 + j;
                let car_number = (car_index + 1) as u32;
                let driver_name = driver_names[car_index as usize].to_string();

                let car = Car {
                    number: car_number,
                    team_number,
                    team_name: team_name_str.clone(), // Store team_name directly in Car for easier access
                    driver: Driver {
                        name: driver_name,
                        skill_level: rng.gen_range(0.5..1.0),
                    },
                    tire: Tire { type_: TireType::Medium, wear: 0.0 },
                    fuel: 100.0,
                    driving_style: DrivingStyle::Normal,
                    status: CarStatus::Racing,
                    race_position: car_number, // Initial placeholder
                    lap: 0,
                    lap_percentage: 0.0,
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
    fn get_client_view(&self) -> RaceStateClientView {
        let mut car_data: Vec<CarClientData> = self.cars.values().map(|car| {
             let track_position = car.lap as f64 + car.lap_percentage as f64;
             // Ensure team_name is sourced correctly, here from car.team_name
             // If car.team_name wasn't added, you'd fetch from self.teams based on car.team_number
             CarClientData {
                 car_number: car.number,
                 driver: car.driver.name.clone(),
                 team_number: car.team_number,
                 team_name: car.team_name.clone(), // Use stored team_name
                 race_position: car.race_position,
                 track_position: track_position, // Combined lap.percentage
                 status: car.status.clone(),
                 tire: ClientTireData { type_: car.tire.type_.clone(), wear: car.tire.wear },
                 fuel: car.fuel,
                 driving_style: car.driving_style.clone(),
                 speed: car.speed, // Use the speed from Car struct
             }
         }).collect();

        // Sort cars by race position (ascending)
        car_data.sort_by_key(|c| c.race_position);

        RaceStateClientView {
            cars: car_data,
            current_lap: self.cars.values().map(|c| c.lap).max().unwrap_or(0), // Example: Leader's lap
            total_laps: self.track.laps,
            race_status: self.run_state.clone(),
        }
    }

    fn update(&mut self) {
        if self.run_state != RaceRunState::Running {
            return; // Don't update if paused or finished
        }

        self.tick_count += 1;
        let mut positions: Vec<(u32, f64)> = Vec::new(); // (car_number, total_distance)

        for car in self.cars.values_mut() {
            if car.status == CarStatus::Dnf {
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
                        car.fuel = new_fuel_level.min(100.0).max(0.0); // Clamp fuel level 0-100
                         // println!("Car {} refueled to {:.1}%.", car.number, car.fuel);
                    }
                    car.status = CarStatus::Racing; // Back to racing
                    // println!("Car {} exits the pits.", car.number);
                }
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
            let distance_km = (base_speed_kph * (performance_multiplier as f64) / 3600.0) * tick_duration_seconds;

            let distance_laps_f64 = distance_km / self.track.lap_length_km;
            car.lap_percentage += distance_laps_f64 as f32;
            
            // Calculate live speed in km/h (converting from distance per tick to km/h)
            car.speed = (base_speed_kph * performance_multiplier as f64) * grip_factor as f64 * (1.0 - fuel_weight_penalty as f64);

            // Handle lap completion
            while car.lap_percentage >= 1.0 {
                car.lap += 1;
                car.lap_percentage -= 1.0;

                // Check for race finish
                if car.lap >= self.track.laps {
                    car.status = CarStatus::Racing; 
                    // println!("Car {} finished the race!", car.number);
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
            }

            // Update tire wear (example rate)
            let tire_wear_rate = 0.05 * performance_multiplier; // Higher performance wears tires faster
            // TODO: Different wear rates for different tire types
            car.tire.wear += tire_wear_rate * tick_duration_seconds as f32;
            car.tire.wear = car.tire.wear.min(100.0); // Cap at 100%?
            // TODO: Consider tire failure above certain wear

            // Store total distance for position calculation
            if car.status == CarStatus::Racing { // Only include racing cars for positions
                 let total_distance = car.lap as f64 * self.track.lap_length_km + car.lap_percentage as f64 * self.track.lap_length_km;
                 positions.push((car.number, total_distance));
             }
        }

        // --- Update Race Positions --- 
        // Sort cars by total distance descending
        positions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Assign race positions
        for (index, (car_number, _)) in positions.iter().enumerate() {
            if let Some(car) = self.cars.get_mut(car_number) {
                car.race_position = (index + 1) as u32;
            }
        }

        // Check if all cars are finished or DNF
        let active_cars = self.cars.values().filter(|c| c.status == CarStatus::Racing || c.status == CarStatus::Pit).count();
        if active_cars == 0 && self.tick_count > 1 { // Avoid finishing instantly
             // println!("Race Finished!");
             self.run_state = RaceRunState::Finished;
        }
    }
}

fn load_track_config(path: &str) -> Result<Track, io::Error> {
    let data = fs::read_to_string(path)?;
    let track: Track = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(track)
}

// Type alias for the shared state used across threads/tasks
type SharedRaceState = Arc<Mutex<RaceState>>;

// Type alias for WebSocket client sender channels
type Clients = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

// Modified handle_command to return a String result
fn handle_command(command_str: String, state: SharedRaceState) -> String {
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
                        "relax" => {
                            car.driving_style = DrivingStyle::Relax;
                            result_messages.push(format!("Car {} driving style set to Relax.", car_num));
                        }
                        "normal" => {
                            car.driving_style = DrivingStyle::Normal;
                             result_messages.push(format!("Car {} driving style set to Normal.", car_num));
                        }
                        "aggressive" => {
                            car.driving_style = DrivingStyle::Aggressive;
                            result_messages.push(format!("Car {} driving style set to Aggressive.", car_num));
                        }
                        _ => result_messages.push(format!("Invalid driving style: {}. Use relax, normal, or aggressive.", style_str)),
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
                         "intermediate" => Some(TireType::Intermediate),
                         "wet" => Some(TireType::Wet),
                         _ => None,
                     };
                     let target_fuel = fuel_str.parse::<f32>().ok();

                     if target_tire.is_none() {
                         result_messages.push(format!("Invalid target tire type: {}", tire_str));
                         return result_messages.join("\\n");
                     }
                     if target_fuel.is_none() || target_fuel.unwrap() < 0.0 || target_fuel.unwrap() > 100.0 {
                          result_messages.push(format!("Invalid target fuel level: {}. Must be 0-100.", fuel_str));
                          return result_messages.join("\\n");
                     }

                     car.pit_request = true;
                     car.target_tire = target_tire;
                     car.target_fuel = target_fuel;
                     result_messages.push(format!("Car {} queued for pit stop: Tire -> {:?}, Fuel -> {:?}%", car_num, car.target_tire.as_ref().unwrap(), car.target_fuel.unwrap()));
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

// Function to handle new WebSocket connections
async fn handle_websocket_connection(ws: warp::ws::WebSocket, clients: Clients, state: SharedRaceState) {
    let client_id = Uuid::new_v4().to_string();
    // println!("New WebSocket client connected: {}", client_id); // Output to UI log instead

    // Create a channel for sending messages to this specific client
    let (client_tx, mut client_rx) = mpsc::unbounded_channel::<Result<Message, warp::Error>>();

    // Add the client's sender channel to the shared list
    clients.lock().unwrap().insert(client_id.clone(), client_tx);

    // Send the current race state immediately upon connection
    {
        let current_state = state.lock().unwrap().get_client_view();
        let state_json = serde_json::to_string(&current_state).unwrap_or_else(|_| "{}".to_string());
        let _ = clients.lock().unwrap().get(&client_id).unwrap().send(Ok(Message::text(state_json)));
    }

    // Split the WebSocket into a sender and receiver
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Task to forward messages from the client-specific channel to the WebSocket sender
    let client_id_fwd = client_id.clone();
    let clients_fwd = Arc::clone(&clients);
    tokio::spawn(async move {
        while let Some(result) = client_rx.recv().await {
            match result {
                Ok(msg) => {
                    if ws_tx.send(msg).await.is_err() {
                        println!("WebSocket send error for client: {}", client_id_fwd);
                        break; // Exit loop on send error
                    }
                }
                Err(e) => {
                     println!("Error receiving message in client channel {}: {}", client_id_fwd, e);
                     break;
                 }
            }
        }
        // If the loop ends, the client channel was closed or there was an error
        println!("Forwarding task ended for client: {}. Cleaning up.", client_id_fwd);
        clients_fwd.lock().unwrap().remove(&client_id_fwd);
    });

    // Handle messages received from the client (optional, e.g., ping/pong)
    while let Some(result) = ws_rx.next().await {
        let _msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("WebSocket receive error for client {}: {}", client_id, e);
                break;
            }
        };
        // We don't expect commands from the client in this version, maybe just handle close/ping
        // println!("Received message from {}: {:?}", client_id, msg);
    }

    // Client disconnected
    // println!("WebSocket client disconnected: {}", client_id); // Output to UI log
    // Removal is handled by the forwarding task when the channel closes or errors
}

// Function to broadcast the current state to all connected clients
fn broadcast_state(state: SharedRaceState, clients: Clients) {
    let clients_map = clients.lock().unwrap();
    if clients_map.is_empty() {
        return; // No clients connected
    }

    let current_state_view = state.lock().unwrap().get_client_view();
    match serde_json::to_string(&current_state_view) {
        Ok(state_json) => {
            // Iterate over a clone of keys to avoid holding the lock during sends
            let client_ids: Vec<String> = clients_map.keys().cloned().collect();

            for client_id in client_ids {
                if let Some(sender) = clients_map.get(&client_id) {
                    // Clone the String for each message, then create the Ok(Message)
                    let message_to_send = Message::text(state_json.clone());
                    if sender.send(Ok(message_to_send)).is_err() {
                        // Error sending means client is likely disconnected
                        // The forwarding task will handle cleanup, but we can log it here too
                        // println!("Failed to send state to disconnected client: {}", client_id);
                        // No need to remove here, handled by the client's task
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize race state for broadcast: {}", e);
        }
    }
}

// --- NCurses UI Section ---
struct UiState {
    last_command: String,
    last_command_result: String,
    current_input: String,
    log_messages: Vec<String>, // For general logs
    input_history: Vec<String>,
    history_index: Option<usize>,
}

impl UiState {
    fn new() -> Self {
        UiState {
            last_command: String::new(),
            last_command_result: String::new(),
            current_input: String::new(),
            log_messages: Vec::new(),
            input_history: Vec::new(),
            history_index: None,
        }
    }

    fn add_log(&mut self, message: String) {
        self.log_messages.push(message);
        if self.log_messages.len() > 5 { // Keep only last 5 logs for example
            self.log_messages.remove(0);
        }
    }
     fn add_to_history(&mut self, command: String) {
        if command.trim().is_empty() || self.input_history.last().map_or(false, |c| c == &command) {
            return;
        }
        self.input_history.push(command);
        self.history_index = None;
    }
}

fn draw_ui(
    window: &Window,
    race_view_opt: &Option<RaceStateClientView>,
    ui_state: &UiState,
    track_name: &str,
    tick_count: u64,
) {
    window.erase();
    let (max_y, max_x) = window.get_max_yx();
    let has_color_support = has_colors();
    
    // Set default color for UI
    if has_color_support {
        window.attron(COLOR_PAIR(7));
    }

    // Line 0: Race Info
    let time_elapsed_secs = tick_count as f64 * 0.1; // Assuming 100ms tick
    window.attron(A_BOLD); // Make title bold
    window.mvprintw(0, 0, &format!("Race: {} Time elapsed: {:.1}s", track_name, time_elapsed_secs));
    window.attroff(A_BOLD);

    // Line 1: Header for cars - Make it bold
    window.attron(A_BOLD);
    window.mvprintw(1,0, &format!("{:<3} {:<5} {:<16} {:<10} {:<10} {:<6} {:<5} {:<5} {:<6} {:<8} {:<10} {:<7}",
        "Pos", "Car#", "Driver", "Team", "Tire", "Wear", "Fuel", "Lap#", "Lap%%", "Status", "Style", "Speed"));
    window.attroff(A_BOLD);
        

    if let Some(race_view) = race_view_opt {
        // Lines for cars
        for (i, car_data) in race_view.cars.iter().enumerate().take(max_y as usize - 5) { // Avoid overflow
            let line = 2 + i as i32;

            // Determine color pair based on team number (1-based)
            let color_pair_num = if has_color_support {
                 match car_data.team_number {
                     1 => 1,
                     2 => 2,
                     3 => 3,
                     4 => 4,
                     5 => 5,
                     _ => 6, // Default color
                 }
            } else { 7 }; // 7 means white-on-black if no team-specific colors

            if has_color_support {
                window.attron(COLOR_PAIR(color_pair_num));
                // Add bold for better visibility if needed
                if car_data.race_position <= 3 { // Highlight top 3
                    window.attron(A_BOLD);
                }
            }

            // First print position, car number in default color
            window.attron(COLOR_PAIR(7)); // Default color
            window.mvprintw(
                line,
                0,
                &format!(
                    "{:<3} ",
                    car_data.race_position
                )
            );
            
            // Print car number and driver name with team color
            if has_color_support {
                window.attron(COLOR_PAIR(color_pair_num));
                if car_data.race_position <= 3 { // Highlight top 3
                    window.attron(A_BOLD);
                }
            }
            window.mvprintw(
                line,
                4, // Position after position column
                &format!(
                    "{:<5} {:<16} ",
                    car_data.car_number,
                    &car_data.driver[..std::cmp::min(car_data.driver.len(), 16)]
                )
            );
            
            // Print team name with team color
            window.mvprintw(
                line,
                27, // Position after driver
                &format!(
                    "{:<10} ",
                    &car_data.team_name[..std::cmp::min(car_data.team_name.len(), 10)]
                )
            );
            
            // Display tire with appropriate color
            let tire_type_str = format!("{:?}", car_data.tire.type_).to_lowercase();
            let tire_color = if has_color_support {
                match car_data.tire.type_ {
                    TireType::Soft => 8,       // Red for Soft
                    TireType::Medium => 9,     // Yellow for Medium
                    TireType::Hard => 10,      // White for Hard
                    TireType::Intermediate => 11, // Green for Intermediate
                    TireType::Wet => 12,       // Blue for Wet
                }
            } else { 7 }; // Default color if no color support
            
            window.attron(COLOR_PAIR(tire_color));
            window.mvprintw(line, 38, &format!("{:<10} ", tire_type_str));
            
            // Rest of data in default color
            window.attron(COLOR_PAIR(7)); // Back to default color
            window.mvprintw(
                line,
                49, // Position after tire type
                &format!(
                    "{:<5.1} {:<5.1} {:<5} {:<5.1}%% {:<8} {:<10} {:<7.1}",
                    car_data.tire.wear,
                    car_data.fuel,
                    car_data.track_position.trunc() as u32,
                    car_data.track_position.fract() * 100.0,
                    format!("{:?}", car_data.status),
                    format!("{:?}", car_data.driving_style),
                    car_data.speed
                )
            );

            if has_color_support {
                if car_data.race_position <= 3 {
                    window.attroff(A_BOLD);
                }
                window.attroff(COLOR_PAIR(color_pair_num));
                window.attron(COLOR_PAIR(7)); // Restore default UI color
            }
        }
        
        // Display race status information with default colors and bold
        window.attron(A_BOLD);
        window.mvprintw(0, max_x - 20, &format!("Status: {:?}", race_view.race_status));
        window.mvprintw(1, max_x - 20, &format!("Lap: {}/{}", race_view.current_lap, race_view.total_laps));
        window.attroff(A_BOLD);
    } else {
        window.mvprintw(2,0, "Waiting for race data...");
    }

    // Rest of the UI (command area, etc.)
    // Use default color for UI elements
    if has_color_support {
        window.attron(COLOR_PAIR(7));
    }

    // Command Area (bottom of screen)
    let cmd_area_start_y = max_y - 4;
    window.attron(COLOR_PAIR(7)); // Ensure using default color pair
    window.mv(cmd_area_start_y, 0);
    window.hline('-', max_x);

    // Last command and result
    let result_display = format!("> {} < {}", ui_state.last_command, ui_state.last_command_result);
    window.mvprintw(cmd_area_start_y + 1, 0, &result_display[..std::cmp::min(result_display.len(), max_x as usize)]);


    // Log messages (above command input)
    let log_start_y = cmd_area_start_y - ui_state.log_messages.len() as i32 -1 ;
    for (i, log_msg) in ui_state.log_messages.iter().enumerate() {
        if log_start_y + i as i32 >= 2 { // Ensure not overwriting car list header
             window.mvprintw(log_start_y + i as i32, 0, &log_msg[..std::cmp::min(log_msg.len(), max_x as usize)]);
        }
    }
    
    // Input prompt with bold
    window.attron(A_BOLD);
    window.mvprintw(cmd_area_start_y + 2, 0, "$ ");
    window.attroff(A_BOLD);
    window.printw(&ui_state.current_input);


    window.mv(cmd_area_start_y + 3, 0);
    window.hline('-', max_x);
    
    // Restore normal attributes before refresh
    window.attroff(A_BOLD);
    if has_color_support {
        window.attroff(COLOR_PAIR(7));
    }
    
    // Track visualization at bottom
    if let Some(race_view) = race_view_opt {
        // Draw the track line under the car list
        let track_line_y = 4 + race_view.cars.len() as i32;
        
        // Draw track borders
        window.attron(COLOR_PAIR(7));
        window.mvprintw(track_line_y - 1, 0, "Track position:");
        window.mvprintw(track_line_y, 0, "[");
        window.mvprintw(track_line_y, max_x - 2, "]");
        
        // Draw track line
        for x in 1..max_x-2 {
            window.mvaddch(track_line_y, x, ACS_HLINE());
        }
        
        // Plot each car on the track
        for car in &race_view.cars {
            // Calculate car's position on the track line - only for current lap progress
            // Extract just the fractional part (lap percentage) for visualization
            let lap_progress = car.track_position.fract();
            let track_x_pos = 1 + ((max_x - 3) as f64 * lap_progress) as i32;
            
            // Ensure car is within track bounds
            let clamped_x = track_x_pos.max(1).min(max_x - 3);
            
            // Display car marker with team color
            let team_color = match car.team_number {
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                5 => 5,
                _ => 6,
            };
            
            // Use car number as the marker
            window.attron(COLOR_PAIR(team_color));
            window.attron(A_BOLD);
            window.mvaddch(track_line_y, clamped_x, car.car_number.to_string().chars().next().unwrap() as chtype);
            window.attroff(A_BOLD);
            window.attroff(COLOR_PAIR(team_color));
        }
    }
    
    window.mv(cmd_area_start_y + 3, 0);
    window.hline('-', max_x);
    
    window.refresh();
}


fn ui_thread_main(
    view_rx: std_mpsc::Receiver<RaceStateClientView>,
    cmd_tx: std_mpsc::Sender<String>,
    log_rx: std_mpsc::Receiver<String>, // For receiving command results and general logs
    initial_track_name: String,
    shared_state_ui: SharedRaceState, // To get tick_count directly
) {
    let window = initscr();
    window.keypad(true); // Enable keypad (arrow keys, etc.)
    noecho(); // Don't echo typed characters
    cbreak(); // Read chars one by one, not waiting for newline
    curs_set(1); // Show cursor
    window.nodelay(true); // Non-blocking input

    // Initialize Colors
    if has_colors() {
        start_color();
        use_default_colors(); // Try to use terminal's default colors
        
        // Define color pairs with explicit BLACK background for better visibility
        init_pair(1, COLOR_CYAN, -1);       // Team 1 (Red Bull)
        init_pair(2, COLOR_RED, -1);    // Team 2 (Ferrari)
        init_pair(3, COLOR_GREEN, -1);      // Team 3 (Mercedes)
        init_pair(4, COLOR_YELLOW, -1);     // Team 4 (McLaren)
        init_pair(5, COLOR_BLUE, -1);      // Team 5 (Alpine)
        init_pair(6, COLOR_WHITE, -1);     // Default/Other
        
        // Use this pair for headers and other UI elements
        init_pair(7, COLOR_WHITE, -1);     // Default UI elements
        
        // Add tire color schemes (8-12)
        init_pair(8, COLOR_RED, -1);       // Soft tires
        init_pair(9, COLOR_YELLOW, -1);    // Medium tires
        init_pair(10, COLOR_WHITE, -1);    // Hard tires
        init_pair(11, COLOR_GREEN, -1);    // Intermediate tires
        init_pair(12, COLOR_BLUE, -1);     // Wet tires
    } else {
        // Handle lack of color support
    }
    
    // Ensure the terminal is using default colors at startup
    window.attron(COLOR_PAIR(7));
    
    let mut ui_state = UiState::new();
    let mut current_race_view: Option<RaceStateClientView> = None;

    loop {
        // 1. Check for new race state view
        match view_rx.try_recv() {
            Ok(new_view) => current_race_view = Some(new_view),
            Err(std_mpsc::TryRecvError::Empty) => {} // No new view
            Err(std_mpsc::TryRecvError::Disconnected) => {
                ui_state.add_log("Race state channel disconnected. Exiting UI.".to_string());
                break;
            }
        }

        // 2. Check for new log messages (command results, system messages)
        match log_rx.try_recv() {
            Ok(log_msg) => {
                if log_msg.starts_with("CMD_RESULT:") {
                    ui_state.last_command_result = log_msg.trim_start_matches("CMD_RESULT:").to_string();
                } else {
                    ui_state.add_log(log_msg);
                }
            }
            Err(std_mpsc::TryRecvError::Empty) => {}
            Err(std_mpsc::TryRecvError::Disconnected) => {
                 ui_state.add_log("Log channel disconnected. Exiting UI.".to_string());
                break;
            }
        }
        
        let tick_count = shared_state_ui.lock().unwrap().tick_count;


        // 3. Draw UI
        draw_ui(&window, &current_race_view, &ui_state, &initial_track_name, tick_count);

        // 4. Handle Input
        match window.getch() {
            Some(Input::Character(c)) => {
                if c == '\n' { // Enter key
                    if !ui_state.current_input.is_empty() {
                        let command_to_send = ui_state.current_input.clone();
                        ui_state.last_command = command_to_send.clone();
                        ui_state.add_to_history(command_to_send.clone());
                        if cmd_tx.send(command_to_send).is_err() {
                            ui_state.add_log("Failed to send command. Processor disconnected.".to_string());
                            // Potentially break loop or handle error
                        }
                        ui_state.current_input.clear();
                        ui_state.last_command_result = "Executing...".to_string(); // Intermediate feedback
                    }
                } else if c == '\x7f' || c == '\u{8}' { // ASCII DEL or Backspace char
                    ui_state.current_input.pop();
                } else if c.is_control() {
                    // Handle other control characters if needed, e.g. arrow keys for history
                    // Ignore other control characters for now
                }
                else {
                    ui_state.current_input.push(c);
                }
            }
            Some(Input::KeyBackspace) => { 
                 ui_state.current_input.pop();
            }
            Some(Input::KeyResize) => { // Handle terminal resize
                // Redraw or reinitialize if necessary, pancurses might handle some aspects
                window.clearok(true);
            }
            Some(Input::KeyUp) => {
                if !ui_state.input_history.is_empty() {
                    let new_idx = ui_state.history_index.map_or(ui_state.input_history.len() - 1, |idx| idx.saturating_sub(1));
                    ui_state.current_input = ui_state.input_history[new_idx].clone();
                    ui_state.history_index = Some(new_idx);
                }
            }
            Some(Input::KeyDown) => {
                 if let Some(idx) = ui_state.history_index {
                    if idx < ui_state.input_history.len() - 1 {
                        let new_idx = idx + 1;
                        ui_state.current_input = ui_state.input_history[new_idx].clone();
                        ui_state.history_index = Some(new_idx);
                    } else {
                        ui_state.history_index = None;
                        ui_state.current_input.clear();
                    }
                }
            }
            Some(_) => {} // Other input types
            None => {}    // No input
        }
        
        if ui_state.current_input == "quit" || ui_state.current_input == "exit" { // Exit condition
             ui_state.add_log("Quit command received. Exiting UI.".to_string());
             // TODO: Signal main app to shut down if desired
             break;
        }

        thread::sleep(Duration::from_millis(30)); // UI refresh rate control
    }

    endwin(); // Cleanup ncurses
    println!("Ncurses UI ended."); // This will print after ncurses is shut down.
}


#[tokio::main]
async fn main() {
    match load_track_config("short.json") {
        Ok(track_config) => {
            let initial_state = RaceState::new(track_config.clone());
            let shared_state = Arc::new(Mutex::new(initial_state));
            let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

            // Create channels for UI communication
            let (view_tx, view_rx) = std_mpsc::channel::<RaceStateClientView>();
            let (cmd_tx, cmd_rx_ui) = std_mpsc::channel::<String>(); // Renamed for clarity in UI thread
            let (log_tx, log_rx) = std_mpsc::channel::<String>();


            // --- Spawn UI Thread ---
            let ui_log_tx = log_tx.clone(); // Clone for initial messages
            let initial_track_name_for_ui = track_config.name.clone();
            let shared_state_for_ui = Arc::clone(&shared_state);
            thread::spawn(move || {
                ui_thread_main(view_rx, cmd_tx, log_rx, initial_track_name_for_ui, shared_state_for_ui);
            });
            
            ui_log_tx.send("UI thread started.".to_string()).ok();
            ui_log_tx.send(format!("Track loaded: {}", track_config.name)).ok();


            // --- Spawn Game Loop Task ---
            let game_state_clone_loop = Arc::clone(&shared_state);
            let clients_clone_loop = Arc::clone(&clients); // Keep for WebSocket if still used
            let game_view_tx = view_tx.clone();
            let game_log_tx = log_tx.clone();
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_millis(100));
                loop {
                    interval.tick().await;
                    let client_view_opt: Option<RaceStateClientView>;
                    {
                        let mut state_guard = game_state_clone_loop.lock().unwrap();
                        let previous_run_state = state_guard.run_state.clone();
                        state_guard.update();
                        client_view_opt = Some(state_guard.get_client_view());

                        if state_guard.run_state == RaceRunState::Finished && previous_run_state != RaceRunState::Finished {
                            game_log_tx.send("Race Finished!".to_string()).ok();
                        }
                         for car in state_guard.cars.values() {
                            if car.status == CarStatus::Pit && car.pit_time_remaining == 0 {
                                // This logic is tricky here as update() handles exit.
                                // Consider adding specific log events inside update() or via return values.
                            }
                        }
                    } 

                    if let Some(cv) = client_view_opt {
                        if game_view_tx.send(cv).is_err() {
                            // UI thread likely closed, maybe log or stop game loop
                            game_log_tx.send("Failed to send view to UI. UI might be closed.".to_string()).ok();
                            break; 
                        }
                    }
                    broadcast_state(Arc::clone(&game_state_clone_loop), Arc::clone(&clients_clone_loop)); // Keep if websockets are active
                }
            });
            ui_log_tx.send("Game loop started.".to_string()).ok();


            // --- Spawn Command Processor Task ---
            let cmd_proc_state = Arc::clone(&shared_state);
            let cmd_proc_log_tx = log_tx.clone();
            tokio::task::spawn_blocking(move || { // Use spawn_blocking for std_mpsc::Receiver
                while let Ok(command_str) = cmd_rx_ui.recv() { // Blocks here until command
                    let result_str = handle_command(command_str, Arc::clone(&cmd_proc_state));
                    if cmd_proc_log_tx.send(format!("CMD_RESULT:{}",result_str)).is_err() {
                        // UI log channel closed
                        break;
                    }
                }
                 // Log that command processor is shutting down if necessary
                let _ = cmd_proc_log_tx.send("Command processor shutting down.".to_string());
            });
             ui_log_tx.send("Command processor started.".to_string()).ok();


            // --- Setup WebSocket Server (Optional - can run in parallel) ---
            let state_filter = warp::any().map(move || Arc::clone(&shared_state));
            let clients_filter = warp::any().map(move || Arc::clone(&clients));
            let ws_log_tx = log_tx.clone();

            let websocket_route = warp::path("ws")
                .and(warp::ws())
                .and(clients_filter)
                .and(state_filter)
                .map(move |ws: warp::ws::Ws, ws_clients: Clients, ws_state: SharedRaceState| {
                    let client_log_tx = ws_log_tx.clone();
                    ws.on_upgrade(move |socket| {
                        client_log_tx.send("New WebSocket client connecting...".to_string()).ok();
                        async move {
                             handle_websocket_connection(socket, ws_clients, ws_state).await;
                             // client_log_tx.send("WebSocket client disconnected.".to_string()).ok(); // Already handled in handle_websocket
                        }
                    })
                });

            let routes = websocket_route;
            let addr = ([127, 0, 0, 1], 3030);
            let server_log_tx = log_tx.clone();
            
            tokio::spawn(async move {
                server_log_tx.send(format!("WebSocket server starting on ws://{}:{}", addr.0.iter().map(|b| b.to_string()).collect::<Vec<_>>().join("."), addr.1)).ok();
                warp::serve(routes).run(addr).await;
            });


            // Keep the main function alive by awaiting something or looping if necessary
            // Since UI is in a separate thread and tokio tasks are spawned,
            // the main tokio runtime will stay alive as long as there are tasks.
            // If all tokio tasks complete and UI thread is still running, main might exit.
            // The warp server will keep its task alive.
            // For robustness, we can wait for a shutdown signal if implemented.
            // For now, the server_task.await (if it were awaited directly here) or just letting tokio run is okay.
            // The current structure with tokio::spawn for the server means main might finish if all other tasks finish.
            // Let's add a small loop to keep main alive, or await a signal.
            // For now, let the tokio runtime manage its lifetime based on spawned tasks.
            // If the UI thread is the primary interface, its termination could signal shutdown for others.

            // Example: Wait for a shutdown signal (not fully implemented here)
            let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
            // In ui_thread_main, on "quit", you could try to send to shutdown_tx.
            
            // This keeps the main alive until a shutdown signal is received or tasks end.
            // If the UI thread is the main controller, its exit should ideally trigger others.
            // For now, tokio will run as long as the warp server task is alive.
            let _ = shutdown_rx.await; // This will wait indefinitely if shutdown_tx is never used.
                                     // Or simply let main finish, tokio tasks will run.
                                     // For a cleaner shutdown, the UI thread would need to signal other tasks.


        }
        Err(e) => {
            // If ncurses is not initialized, println is fine.
            // If it might be, this error might not be visible.
            eprintln!("Failed to load track configuration: {}", e);
            // Consider logging to a file as a fallback if UI fails early.
            return;
        }
    }
}
// ... (rest of the comments are fine)