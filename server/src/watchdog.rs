use crate::database::queries as tdb;
use crate::models::race::RaceState;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Watchdog service that monitors and manages scheduled races
///
/// Responsibilities:
/// - Start races when their scheduled time arrives
/// - Cancel races that passed their start time without running
/// - Ensure only one race runs at a time
pub struct RaceWatchdog {
    db_pool: Arc<PgPool>,
    race_state: Arc<Mutex<crate::models::race::RaceState>>,
}

impl RaceWatchdog {
    pub fn new(
        db_pool: Arc<PgPool>,
        race_state: Arc<Mutex<crate::models::race::RaceState>>,
    ) -> Self {
        Self {
            db_pool,
            race_state,
        }
    }

    /// Check and process races that need attention
    /// Returns the number of races started, marked as upcoming, and canceled
    pub async fn check_races(&self) -> Result<(usize, usize, usize), sqlx::Error> {
        let mut started_count = 0;
        let mut upcoming_count = 0;
        let mut canceled_count = 0;

        // First, cancel races that passed their start time without running
        let races_to_cancel = tdb::get_races_to_cancel(&self.db_pool).await?;
        for race in races_to_cancel {
            tracing::info!(
                "Canceling race {} (start_datetime: {:?}, status: {})",
                race.id,
                race.start_datetime,
                race.status
            );
            if let Err(e) = tdb::update_race_status(&self.db_pool, race.id, "CANCELED").await {
                tracing::error!("Failed to cancel race {}: {:?}", race.id, e);
            } else {
                canceled_count += 1;
            }
        }

        // Mark races as UPCOMING 5 minutes before start
        let races_to_mark_upcoming = tdb::get_races_to_mark_upcoming(&self.db_pool).await?;
        if !races_to_mark_upcoming.is_empty() {
            tracing::debug!(
                "Found {} races to mark as UPCOMING",
                races_to_mark_upcoming.len()
            );
        }
        for race in races_to_mark_upcoming {
            use chrono::Utc;
            let now = Utc::now();
            let time_until_start = race.start_datetime.map(|dt| dt - now);
            tracing::info!(
                "Marking race {} as UPCOMING (start_datetime: {:?}, time until start: {:?}, current status: {})",
                race.id,
                race.start_datetime,
                time_until_start,
                race.status
            );
            if let Err(e) = tdb::update_race_status(&self.db_pool, race.id, "UPCOMING").await {
                tracing::error!("Failed to mark race {} as UPCOMING: {:?}", race.id, e);
            } else {
                upcoming_count += 1;
                // Load the race into the game loop (paused) so it's visible
                if let Err(e) = self.load_upcoming_race(race.id).await {
                    tracing::error!("Failed to load upcoming race {}: {:?}", race.id, e);
                }
            }
        }

        // Check if there's already an ongoing race
        let has_ongoing = tdb::has_ongoing_race(&self.db_pool).await?;

        // Also check if the current race state is running
        let race_state_is_running = {
            let state_guard = self.race_state.lock().unwrap();
            matches!(
                state_guard.run_state,
                crate::models::race::RaceRunState::Running
                    | crate::models::race::RaceRunState::LastLap
            )
        };

        // Only start a race if no race is currently running
        if !has_ongoing && !race_state_is_running {
            let races_to_start = tdb::get_races_to_start(&self.db_pool).await?;

            // Start the first race that needs to start
            if let Some(race) = races_to_start.first() {
                tracing::info!(
                    "Starting race {} (start_datetime: {:?}, status: {})",
                    race.id,
                    race.start_datetime,
                    race.status
                );

                match self.start_race(race.id).await {
                    Ok(_) => {
                        started_count += 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to start race {}: {:?}", race.id, e);
                    }
                }
            }
        } else {
            if has_ongoing {
                tracing::debug!(
                    "Skipping race start: there is already an ongoing race in the database"
                );
            }
            if race_state_is_running {
                tracing::debug!("Skipping race start: there is already a race running in memory");
            }
        }

        // Also check if we need to load an UPCOMING race that isn't already loaded
        // (in case the server restarted and there's an UPCOMING race)
        if !race_state_is_running {
            let upcoming_races = tdb::get_upcoming_races(&self.db_pool).await?;
            if let Some(upcoming_race) = upcoming_races.first() {
                let current_race_id = {
                    let state_guard = self.race_state.lock().unwrap();
                    state_guard.race_id
                };

                // Only load if it's not already loaded
                if current_race_id != Some(upcoming_race.id) {
                    if let Err(e) = self.load_upcoming_race(upcoming_race.id).await {
                        tracing::error!(
                            "Failed to load upcoming race {}: {:?}",
                            upcoming_race.id,
                            e
                        );
                    }
                }
            }
        }

        Ok((started_count, upcoming_count, canceled_count))
    }

    /// Load an upcoming race into the game loop (paused) so it's visible
    async fn load_upcoming_race(&self, race_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        // Load the race from the database
        let mut new_race_state = RaceState::load_scheduled_race(&self.db_pool, race_id)
            .await
            .map_err(|e| format!("Failed to load race: {}", e))?;

        // Set the database pool for saving events
        new_race_state.set_db_pool(Arc::new(self.db_pool.as_ref().clone()));

        // Ensure the race starts paused
        new_race_state.run_state = crate::models::race::RaceRunState::Paused;

        // Replace the current race state
        {
            let mut race_state_guard = self.race_state.lock().unwrap();
            *race_state_guard = new_race_state;
        }

        tracing::info!("Race {} loaded as UPCOMING (paused)", race_id);
        Ok(())
    }

    /// Start a race by loading it and updating the race state
    async fn start_race(&self, race_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        // Check if the race is already loaded (might be UPCOMING)
        let race_already_loaded = {
            let state_guard = self.race_state.lock().unwrap();
            state_guard.race_id == Some(race_id)
        };

        if !race_already_loaded {
            // Load the race from the database
            let mut new_race_state = RaceState::load_scheduled_race(&self.db_pool, race_id)
                .await
                .map_err(|e| format!("Failed to load race: {}", e))?;

            // Set the database pool for saving events
            new_race_state.set_db_pool(Arc::new(self.db_pool.as_ref().clone()));

            // Replace the current race state
            {
                let mut race_state_guard = self.race_state.lock().unwrap();
                *race_state_guard = new_race_state;
            }
        }

        // Update race status to ONGOING and set start_datetime
        tdb::start_race(&self.db_pool, race_id)
            .await
            .map_err(|e| format!("Failed to start race in database: {:?}", e))?;

        // Start the race simulation (this will change Paused to Running)
        crate::commands::handle_command("start".to_string(), self.race_state.clone());

        tracing::info!("Race {} started successfully", race_id);
        Ok(())
    }
}

/// Spawn the watchdog task that runs every minute
pub fn spawn_watchdog(
    db_pool: Arc<PgPool>,
    race_state: Arc<Mutex<crate::models::race::RaceState>>,
) {
    let watchdog = RaceWatchdog::new(db_pool, race_state);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

        // Run immediately on startup
        interval.tick().await;

        loop {
            interval.tick().await;

            match watchdog.check_races().await {
                Ok((started, upcoming, canceled)) => {
                    if started > 0 || upcoming > 0 || canceled > 0 {
                        tracing::info!(
                            "Watchdog check completed: {} races started, {} races marked as upcoming, {} races canceled",
                            started,
                            upcoming,
                            canceled
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Watchdog check failed: {:?}", e);
                }
            }
        }
    });
}
