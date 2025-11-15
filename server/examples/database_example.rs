// Example usage of the database module
// Run with: cargo run --example database_example

use tiny_racing::database::*;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing".to_string());

    println!("Connecting to database...");
    let db = Database::new(&database_url).await?;

    println!("Running migrations...");
    db.migrate().await?;
    println!("Migrations completed!");

    // Create a team
    println!("\nCreating a team...");
    let team = create_team(
        db.pool(),
        CreateTeamRequest {
            number: 1,
            name: "Red Bull Racing".to_string(),
            logo: "redbull.svg".to_string(),
            color: "#1E41FF".to_string(),
            pit_efficiency: 0.95,
        },
    )
    .await?;
    println!("Created team: {} (ID: {})", team.name, team.id);

    // Create a driver
    println!("\nCreating a driver...");
    let driver = create_driver(
        db.pool(),
        CreateDriverRequest {
            name: "Max Verstappen".to_string(),
            skill_level: 0.95,
            stamina: 0.90,
            weather_tolerance: 0.85,
            experience: 0.92,
            consistency: 0.88,
            focus: 0.93,
        },
    )
    .await?;
    println!("Created driver: {} (ID: {})", driver.name, driver.id);

    // Create a car
    println!("\nCreating a car...");
    let car = create_car(
        db.pool(),
        CreateCarRequest {
            number: 33,
            team_id: team.id,
            driver_id: driver.id,
            handling: 0.92,
            acceleration: 0.94,
            top_speed: 0.96,
            reliability: 0.90,
            fuel_consumption: 0.88,
            tire_wear: 0.85,
            base_performance: 1.05,
        },
    )
    .await?;
    println!("Created car #{} (ID: {})", car.number, car.id);

    // Create a track
    println!("\nCreating a track...");
    let track = create_track(
        db.pool(),
        CreateTrackRequest {
            track_id: "bahrain".to_string(),
            name: "Bahrain International Circuit".to_string(),
            description: Some("A challenging desert circuit".to_string()),
            laps: 57,
            lap_length_km: 5.412,
            svg_start_offset: 0.0,
        },
    )
    .await?;
    println!("Created track: {} (ID: {})", track.name, track.id);

    // Create a player
    println!("\nCreating a player...");
    let player = create_player(
        db.pool(),
        CreatePlayerRequest {
            username: "racing_fan".to_string(),
            email: Some("fan@example.com".to_string()),
        },
    )
    .await?;
    println!("Created player: {} (ID: {})", player.username, player.id);

    // Query examples
    println!("\n=== Query Examples ===");

    // Get team by number
    if let Some(team) = get_team_by_number(db.pool(), 1).await? {
        println!("Found team by number: {}", team.name);
    }

    // List all teams
    let teams = list_teams(db.pool()).await?;
    println!("Total teams: {}", teams.len());

    // List all cars
    let cars = list_cars(db.pool()).await?;
    println!("Total cars: {}", cars.len());

    // List cars for a team
    let team_cars = list_cars_by_team(db.pool(), team.id).await?;
    println!("Cars for team '{}': {}", team.name, team_cars.len());

    // Get driver by name
    if let Some(driver) = get_driver_by_name(db.pool(), "Max Verstappen").await? {
        println!("Found driver: {} (skill: {})", driver.name, driver.skill_level);
    }

    // List all tracks
    let tracks = list_tracks(db.pool()).await?;
    println!("Total tracks: {}", tracks.len());

    // List all players
    let players = list_players(db.pool()).await?;
    println!("Total players: {}", players.len());

    println!("\nExample completed successfully!");

    Ok(())
}

