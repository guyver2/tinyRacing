#![allow(dead_code)]
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use uuid::Uuid;
use warp::ws::Message;
use warp::Filter; // For generating unique client IDsuse std::convert::Infallible;

use std::sync::mpsc as std_mpsc;
use std::thread;

mod models;
use crate::models::race::{RaceRunState, RaceState, RaceStateClientView};

mod commands;
use crate::commands::*;

mod ncurses_ui;
use crate::ncurses_ui::*;

mod database;
use crate::database::{finish_race, init_from_env, Database};
mod api;
mod auth;
mod auth_middleware;

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

#[tokio::main]
async fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} [track folder]", args[0]);
        std::process::exit(1);
    }

    //match Track::load_track_config(args[1].as_str()) {
    //    Ok(track_config) => {
    //        let initial_state = RaceState::new(track_config.clone());
    match RaceState::load_race_config(args[1].as_str()) {
        Ok(initial_state) => {
            let track_id = initial_state.track.id.clone();
            let track_name = initial_state.track.name.clone();
            let shared_state = Arc::new(Mutex::new(initial_state));
            let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

            // Create channels for UI communication
            let (view_tx, view_rx) = std_mpsc::channel::<RaceStateClientView>();
            let (cmd_tx, cmd_rx_ui) = std_mpsc::channel::<String>(); // Renamed for clarity in UI thread
            let (log_tx, log_rx) = std_mpsc::channel::<String>();

            // Clone the race state for the API server
            let api_race_state = shared_state.clone();

            // Initialize database connection and run migrations
            let db_pool = if let Some(database_url) = std::env::var("DATABASE_URL").ok() {
                tracing::info!("Connecting to database and running migrations...");
                match Database::new(&database_url).await {
                    Ok(db) => {
                        // Run migrations
                        if let Err(e) = db.migrate().await {
                            tracing::warn!("Failed to run migrations: {}. Continuing anyway...", e);
                        } else {
                            tracing::info!("Database migrations completed successfully");
                        }
                        Some(db.pool().clone())
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to database: {}. API database endpoints will not work.", e);
                        None
                    }
                }
            } else {
                init_from_env().await
            };

            // Clone db_pool for the game loop
            let game_loop_db_pool = db_pool.clone();

            // Start the API server in a separate task
            tokio::spawn(async move {
                let app = api::create_api_router(api_race_state, db_pool);

                let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
                tracing::info!("API server listening on http://localhost:3000");

                axum::serve(listener, app).await.unwrap();
            });

            // --- Spawn UI Thread (only if not disabled) ---
            // In Docker/headless mode, disable UI to avoid terminal errors
            let disable_ui =
                std::env::var("DISABLE_UI").unwrap_or_else(|_| "true".to_string()) == "true";

            if !disable_ui {
                let ui_log_tx = log_tx.clone(); // Clone for initial messages
                let initial_track_name_for_ui = track_name.clone();
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
            } else {
                // In headless mode, consume messages from channels to prevent blocking
                // Consume view updates
                thread::spawn(move || {
                    while let Ok(_) = view_rx.recv() {
                        // Consume view updates in headless mode
                    }
                });
                // Consume log messages
                thread::spawn(move || {
                    while let Ok(_) = log_rx.recv() {
                        // Consume log messages in headless mode
                    }
                });
                // Commands are not processed in headless mode (cmd_tx will just drop messages)
                log_tx
                    .send("Running in headless mode (UI disabled).".to_string())
                    .ok();
            }

            let ui_log_tx = log_tx.clone();
            ui_log_tx
                .send(format!("Track loaded: {}:{}", track_id, track_name))
                .ok();

            // Log that the API is available
            ui_log_tx
                .send("REST API server available at http://localhost:3000".to_string())
                .ok();

            // --- Spawn Game Loop Task ---
            let game_state_clone_loop = Arc::clone(&shared_state);
            let clients_clone_loop = Arc::clone(&clients); // Keep for WebSocket if still used
            let game_view_tx = view_tx.clone();
            let game_log_tx = log_tx.clone();
            let config_path = args[1].clone(); // Store config path for potential restart
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_millis(100));
                loop {
                    interval.tick().await;
                    let client_view_opt: Option<RaceStateClientView>;
                    let race_id_opt: Option<Uuid>;
                    let race_just_finished: bool;
                    let should_restart = {
                        let mut state_guard = game_state_clone_loop.lock().unwrap();
                        let previous_run_state = state_guard.run_state.clone();
                        race_id_opt = state_guard.race_id;
                        state_guard.update();
                        client_view_opt = Some(state_guard.get_client_view());

                        race_just_finished = state_guard.run_state == RaceRunState::Finished
                            && previous_run_state != RaceRunState::Finished;

                        let should_restart = if race_just_finished {
                            game_log_tx.send("Race Finished!".to_string()).ok();
                            // Check if auto restart is enabled
                            crate::models::race::is_auto_race_restart_enabled()
                        } else {
                            false
                        };
                        should_restart
                    };

                    // Update database status to FINISHED if this is a scheduled race that just finished
                    // Do this outside the mutex guard to avoid holding it across await
                    if race_just_finished {
                        if let Some(race_id) = race_id_opt {
                            if let Some(pool) = &game_loop_db_pool {
                                if let Err(e) = finish_race(pool, race_id).await {
                                    game_log_tx
                                        .send(format!(
                                            "Failed to update race status to FINISHED: {:?}",
                                            e
                                        ))
                                        .ok();
                                }
                            }
                        }
                    }

                    // Restart race if needed
                    if should_restart {
                        match RaceState::load_race_config(&config_path) {
                            Ok(new_state) => {
                                *game_state_clone_loop.lock().unwrap() = new_state;
                                game_log_tx
                                    .send("Race automatically restarted!".to_string())
                                    .ok();
                            }
                            Err(e) => {
                                game_log_tx
                                    .send(format!(
                                        "Failed to restart race: {}. Race will remain finished.",
                                        e
                                    ))
                                    .ok();
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

            // --- Spawn Command Processor Task (only if UI enabled) ---
            if !disable_ui {
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
            }

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
                            }
                        })
                    },
                );

            let routes = websocket_route;
            let addr = ([0, 0, 0, 0], 3030);
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

            // Example: Wait for a shutdown signal (not fully implemented here)
            let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

            // This keeps the main alive until a shutdown signal is received or tasks end.
            let _ = shutdown_rx.await;
        }
        Err(e) => {
            eprintln!("Failed to load track configuration: {}", e);
            return;
        }
    }
}
