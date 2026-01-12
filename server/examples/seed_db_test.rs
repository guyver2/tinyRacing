// Test-specific seed script that creates a test user
// This is based on seed_db.rs but ensures testuser exists with known credentials
// Run with: cargo run --example seed_db_test

use tiny_racing::database::*;

const TEST_PLAYER: &str = "testuser";
const TEST_PASSWORD: &str = "testpass123";
const TEST_EMAIL: &str = "testuser@test.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get database URL from environment or use test default
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://tiny_racing_test:test_password@localhost:5433/tiny_racing_test".to_string()
    });

    println!("Connecting to test database...");
    let db = Database::new(&database_url).await?;

    println!("Running migrations...");
    db.migrate().await?;
    println!("Migrations completed!");

    // Ensure test user exists
    println!("\n=== Ensuring Test User Exists ===");
    let test_player = match get_player_by_username(db.pool(), TEST_PLAYER).await? {
        Some(player) => {
            println!("Test user '{}' already exists (ID: {})", player.username, player.id);
            player
        }
        None => {
            println!("Creating test user '{}'...", TEST_PLAYER);
            let player = create_player(
                db.pool(),
                CreatePlayerRequest {
                    username: TEST_PLAYER.to_string(),
                    password: TEST_PASSWORD.to_string(),
                    email: Some(TEST_EMAIL.to_string()),
                },
            )
            .await?;
            println!("Created test user: {} (ID: {})", player.username, player.id);
            player
        }
    };

    // Assign team #1 to test user if not already assigned
    println!("\n=== Assigning Team to Test User ===");
    let team = get_team_by_number(db.pool(), 1).await?;
    if let Some(team) = team {
        if team.player_id.is_none() {
            // Update team to assign to test user
            // Note: You may need to add an update_team function to the database module
            println!("Team #1 ({}) is not assigned to any player", team.name);
            println!("Note: Team assignment may need to be done via API or database update");
        } else if team.player_id == Some(test_player.id) {
            println!("Team #1 ({}) is already assigned to test user", team.name);
        } else {
            println!("Team #1 ({}) is assigned to a different player", team.name);
        }
    } else {
        println!("Team #1 not found - seed_db.rs should be run first");
    }

    println!("\nTest database setup complete!");
    println!("Test user credentials:");
    println!("  Username: {}", TEST_PLAYER);
    println!("  Password: {}", TEST_PASSWORD);

    Ok(())
}

