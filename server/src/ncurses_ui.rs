use pancurses::*;
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::time::Duration;

use crate::models::race::{RaceState, RaceStateClientView};
use crate::models::tire::TireType;

/// UI module for the race simulation using ncurses
///
/// This module provides the terminal user interface for the race simulation using ncurses.
/// It handles displaying race state information, car positions, track visualization,
/// command input, and logging in a terminal-based interface.
///
/// The UI shows:
/// - Race status and track information
/// - Car positions, lap times, and status
/// - Track visualization with car positions
/// - Command input area
/// - Log messages
///
/// Key features:
/// - Color-coded display for teams and tire compounds
/// - Real-time race state updates
/// - Command history navigation
/// - Scrolling log messages
/// - Track position visualization
///
/// The UI runs in its own thread and communicates with the main game loop through channels.

// Type alias for the shared state used across threads/tasks
type SharedRaceState = Arc<Mutex<RaceState>>;

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

fn manual_or_auto_player(player_uuid: Option<&String>) -> String {
    if let Some(_uuid) = player_uuid {
        return "Human".to_string();
    } else {
        return "AI".to_string();
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
    window.mvprintw(1,0, &format!("{:<3} {:<5} {:<16} {:<10} {:<10} {:<6} {:<5} {:<5} {:<6} {:<8} {:<10} {:<7} {:<10} {:<10} {:<6}",
        "Pos", "Car#", "Driver", "Team", "Tire", "Wear", "Fuel", "Lap#", "Lap%%", "Status", "Style", "Speed", "Finished T", "Distance", "Player"));
    window.attroff(A_BOLD);

    if let Some(race_view) = race_view_opt {
        // Lines for cars
        for (i, car_data) in race_view.cars.iter().enumerate().take(max_y as usize - 5) {
            // Avoid overflow
            let line = 2 + i as i32;

            // Determine color pair based on team number (1-based)
            let color_pair_num = if has_color_support {
                match car_data.team.number {
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
                    &car_data.driver.name[..std::cmp::min(car_data.driver.name.len(), 16)]
                ),
            );

            // Print team name with team color
            window.mvprintw(
                line,
                27, // Position after driver
                &format!(
                    "{:<10} ",
                    &car_data.team.name[..std::cmp::min(car_data.team.name.len(), 10)]
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
                    "{:<5.1} {:<5.1} {:<5} {:<5.1}%% {:<8} {:<10} {:<7.1} {:<10} {:<1.8} {:<6}",
                    car_data.tire.wear,
                    car_data.fuel,
                    car_data.track_position.trunc() as u32,
                    car_data.track_position.fract() * 100.0,
                    format!("{:?}", car_data.status),
                    format!("{:?}", car_data.driving_style),
                    car_data.speed,
                    car_data.finished_time,
                    car_data.track_position,
                    manual_or_auto_player(car_data.player_uuid.as_ref()).clone(),
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
            let team_color = match car.team.number {
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

pub fn ui_thread_main(
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
            _ => {}       // No input
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
