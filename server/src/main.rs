use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;


use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use uuid::Uuid;
use warp::ws::Message;
use warp::Filter; // For generating unique client IDsuse std::convert::Infallible;

// Added for ncurses UI
use pancurses::*;
use std::sync::mpsc as std_mpsc;
use std::thread;

mod models;
use crate::models::*;

mod race_state;
use crate::race_state::*;

mod commands;
use crate::commands::*;

fn load_track_config(path: &str) -> Result<Track, io::Error> {
    let data = fs::read_to_string(path)?;
    let track: Track =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(track)
}

// Type alias for the shared state used across threads/tasks
type SharedRaceState = Arc<Mutex<RaceState>>;

// Type alias for WebSocket client sender channels
type Clients = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;


// Function to handle new WebSocket connections
async fn handle_websocket_connection(
    ws: warp::ws::WebSocket,
    clients: Clients,
    state: SharedRaceState,
) {
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
        let _ = clients
            .lock()
            .unwrap()
            .get(&client_id)
            .unwrap()
            .send(Ok(Message::text(state_json)));
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
                    println!(
                        "Error receiving message in client channel {}: {}",
                        client_id_fwd, e
                    );
                    break;
                }
            }
        }
        // If the loop ends, the client channel was closed or there was an error
        println!(
            "Forwarding task ended for client: {}. Cleaning up.",
            client_id_fwd
        );
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
        if self.log_messages.len() > 5 {
            // Keep only last 5 logs for example
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
    window.mvprintw(
        0,
        0,
        &format!(
            "Race: {} Time elapsed: {:.1}s",
            track_name, time_elapsed_secs
        ),
    );
    window.attroff(A_BOLD);

    // Line 1: Header for cars - Make it bold
    window.attron(A_BOLD);
    window.mvprintw(1,0, &format!("{:<3} {:<5} {:<16} {:<10} {:<10} {:<6} {:<5} {:<5} {:<6} {:<8} {:<10} {:<7} {:<10} {:<10}",
        "Pos", "Car#", "Driver", "Team", "Tire", "Wear", "Fuel", "Lap#", "Lap%%", "Status", "Style", "Speed", "Finished Time", "Total Distance"));
    window.attroff(A_BOLD);

    if let Some(race_view) = race_view_opt {
        // Lines for cars
        for (i, car_data) in race_view.cars.iter().enumerate().take(max_y as usize - 5) {
            // Avoid overflow
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
            } else {
                7
            }; // 7 means white-on-black if no team-specific colors

            if has_color_support {
                window.attron(COLOR_PAIR(color_pair_num));
                // Add bold for better visibility if needed
                if car_data.race_position <= 3 {
                    // Highlight top 3
                    window.attron(A_BOLD);
                }
            }

            // First print position, car number in default color
            window.attron(COLOR_PAIR(7)); // Default color
            window.mvprintw(line, 0, &format!("{:<3} ", car_data.race_position));

            // Print car number and driver name with team color
            if has_color_support {
                window.attron(COLOR_PAIR(color_pair_num));
                if car_data.race_position <= 3 {
                    // Highlight top 3
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
                ),
            );

            // Print team name with team color
            window.mvprintw(
                line,
                27, // Position after driver
                &format!(
                    "{:<10} ",
                    &car_data.team_name[..std::cmp::min(car_data.team_name.len(), 10)]
                ),
            );

            // Display tire with appropriate color
            let tire_type_str = format!("{:?}", car_data.tire.type_).to_lowercase();
            let tire_color = if has_color_support {
                match car_data.tire.type_ {
                    TireType::Soft => 8,          // Red for Soft
                    TireType::Medium => 9,        // Yellow for Medium
                    TireType::Hard => 10,         // White for Hard
                    TireType::Intermediate => 11, // Green for Intermediate
                    TireType::Wet => 12,          // Blue for Wet
                }
            } else {
                7
            }; // Default color if no color support

            window.attron(COLOR_PAIR(tire_color));
            window.mvprintw(line, 38, &format!("{:<10} ", tire_type_str));

            // Rest of data in default color
            window.attron(COLOR_PAIR(7)); // Back to default color
            window.mvprintw(
                line,
                49, // Position after tire type
                &format!(
                    "{:<5.1} {:<5.1} {:<5} {:<5.1}%% {:<8} {:<10} {:<7.1} {:<10} {:<10}",
                    car_data.tire.wear,
                    car_data.fuel,
                    car_data.track_position.trunc() as u32,
                    car_data.track_position.fract() * 100.0,
                    format!("{:?}", car_data.status),
                    format!("{:?}", car_data.driving_style),
                    car_data.speed,
                    car_data.finished_time,
                    car_data.track_position
                ),
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
        window.mvprintw(
            0,
            max_x - 20,
            &format!("Status: {:?}", race_view.race_status),
        );
        window.mvprintw(
            1,
            max_x - 20,
            &format!("Lap: {}/{}", race_view.current_lap, race_view.total_laps),
        );
        window.attroff(A_BOLD);
    } else {
        window.mvprintw(2, 0, "Waiting for race data...");
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
    let result_display = format!(
        "> {} < {}",
        ui_state.last_command, ui_state.last_command_result
    );
    window.mvprintw(
        cmd_area_start_y + 1,
        0,
        &result_display[..std::cmp::min(result_display.len(), max_x as usize)],
    );

    // Log messages (above command input)
    let log_start_y = cmd_area_start_y - ui_state.log_messages.len() as i32 - 1;
    for (i, log_msg) in ui_state.log_messages.iter().enumerate() {
        if log_start_y + i as i32 >= 2 {
            // Ensure not overwriting car list header
            window.mvprintw(
                log_start_y + i as i32,
                0,
                &log_msg[..std::cmp::min(log_msg.len(), max_x as usize)],
            );
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
        for x in 1..max_x - 2 {
            window.mvaddch(track_line_y, x, ACS_HLINE());
        }

        // Plot each car on the track line
        for car in &race_view.cars {
            // Calculate car's position on the track line - only for current lap progress
            // Extract just the fractional part (lap percentage) for visualization
            let lap_progress = car.track_position.fract();
            let track_x_pos = 1 + ((max_x - 3) as f32 * lap_progress) as i32;

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
            window.mvaddch(
                track_line_y,
                clamped_x,
                car.car_number.to_string().chars().next().unwrap() as chtype,
            );
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
        init_pair(1, COLOR_CYAN, -1); // Team 1 (Red Bull)
        init_pair(2, COLOR_RED, -1); // Team 2 (Ferrari)
        init_pair(3, COLOR_GREEN, -1); // Team 3 (Mercedes)
        init_pair(4, COLOR_YELLOW, -1); // Team 4 (McLaren)
        init_pair(5, COLOR_BLUE, -1); // Team 5 (Alpine)
        init_pair(6, COLOR_WHITE, -1); // Default/Other

        // Use this pair for headers and other UI elements
        init_pair(7, COLOR_WHITE, -1); // Default UI elements

        // Add tire color schemes (8-12)
        init_pair(8, COLOR_RED, -1); // Soft tires
        init_pair(9, COLOR_YELLOW, -1); // Medium tires
        init_pair(10, COLOR_WHITE, -1); // Hard tires
        init_pair(11, COLOR_GREEN, -1); // Intermediate tires
        init_pair(12, COLOR_BLUE, -1); // Wet tires
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
                    ui_state.last_command_result =
                        log_msg.trim_start_matches("CMD_RESULT:").to_string();
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
        draw_ui(
            &window,
            &current_race_view,
            &ui_state,
            &initial_track_name,
            tick_count,
        );

        // 4. Handle Input
        match window.getch() {
            Some(Input::Character(c)) => {
                if c == '\n' {
                    // Enter key
                    if !ui_state.current_input.is_empty() {
                        let command_to_send = ui_state.current_input.clone();
                        ui_state.last_command = command_to_send.clone();
                        ui_state.add_to_history(command_to_send.clone());
                        if cmd_tx.send(command_to_send).is_err() {
                            ui_state.add_log(
                                "Failed to send command. Processor disconnected.".to_string(),
                            );
                            // Potentially break loop or handle error
                        }
                        ui_state.current_input.clear();
                        ui_state.last_command_result = "Executing...".to_string();
                        // Intermediate feedback
                    }
                } else if c == '\x7f' || c == '\u{8}' {
                    // ASCII DEL or Backspace char
                    ui_state.current_input.pop();
                } else if c.is_control() {
                    // Handle other control characters if needed, e.g. arrow keys for history
                    // Ignore other control characters for now
                } else {
                    ui_state.current_input.push(c);
                }
            }
            Some(Input::KeyBackspace) => {
                ui_state.current_input.pop();
            }
            Some(Input::KeyResize) => {
                // Handle terminal resize
                // Redraw or reinitialize if necessary, pancurses might handle some aspects
                window.clearok(true);
            }
            Some(Input::KeyUp) => {
                if !ui_state.input_history.is_empty() {
                    let new_idx = ui_state
                        .history_index
                        .map_or(ui_state.input_history.len() - 1, |idx| {
                            idx.saturating_sub(1)
                        });
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

        if ui_state.current_input == "quit" || ui_state.current_input == "exit" {
            // Exit condition
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <track_config.json>", args[0]);
        std::process::exit(1);
    }
    match load_track_config(&args[1]) {
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
                ui_thread_main(
                    view_rx,
                    cmd_tx,
                    log_rx,
                    initial_track_name_for_ui,
                    shared_state_for_ui,
                );
            });

            ui_log_tx.send("UI thread started.".to_string()).ok();
            ui_log_tx
                .send(format!("Track loaded: {}", track_config.name))
                .ok();

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

                        if state_guard.run_state == RaceRunState::Finished
                            && previous_run_state != RaceRunState::Finished
                        {
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
                            game_log_tx
                                .send("Failed to send view to UI. UI might be closed.".to_string())
                                .ok();
                            break;
                        }
                    }
                    broadcast_state(
                        Arc::clone(&game_state_clone_loop),
                        Arc::clone(&clients_clone_loop),
                    ); // Keep if websockets are active
                }
            });
            ui_log_tx.send("Game loop started.".to_string()).ok();

            // --- Spawn Command Processor Task ---
            let cmd_proc_state = Arc::clone(&shared_state);
            let cmd_proc_log_tx = log_tx.clone();
            tokio::task::spawn_blocking(move || {
                // Use spawn_blocking for std_mpsc::Receiver
                while let Ok(command_str) = cmd_rx_ui.recv() {
                    // Blocks here until command
                    let result_str = handle_command(command_str, Arc::clone(&cmd_proc_state));
                    if cmd_proc_log_tx
                        .send(format!("CMD_RESULT:{}", result_str))
                        .is_err()
                    {
                        // UI log channel closed
                        break;
                    }
                }
                // Log that command processor is shutting down if necessary
                let _ = cmd_proc_log_tx.send("Command processor shutting down.".to_string());
            });
            ui_log_tx
                .send("Command processor started.".to_string())
                .ok();

            // --- Setup WebSocket Server (Optional - can run in parallel) ---
            let state_filter = warp::any().map(move || Arc::clone(&shared_state));
            let clients_filter = warp::any().map(move || Arc::clone(&clients));
            let ws_log_tx = log_tx.clone();

            let websocket_route = warp::path("ws")
                .and(warp::ws())
                .and(clients_filter)
                .and(state_filter)
                .map(
                    move |ws: warp::ws::Ws, ws_clients: Clients, ws_state: SharedRaceState| {
                        let client_log_tx = ws_log_tx.clone();
                        ws.on_upgrade(move |socket| {
                            client_log_tx
                                .send("New WebSocket client connecting...".to_string())
                                .ok();
                            async move {
                                handle_websocket_connection(socket, ws_clients, ws_state).await;
                                // client_log_tx.send("WebSocket client disconnected.".to_string()).ok(); // Already handled in handle_websocket
                            }
                        })
                    },
                );

            let routes = websocket_route;
            let addr = ([127, 0, 0, 1], 3030);
            let server_log_tx = log_tx.clone();

            tokio::spawn(async move {
                server_log_tx
                    .send(format!(
                        "WebSocket server starting on ws://{}:{}",
                        addr.0
                            .iter()
                            .map(|b| b.to_string())
                            .collect::<Vec<_>>()
                            .join("."),
                        addr.1
                    ))
                    .ok();
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
