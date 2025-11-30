// Standalone program to ensure all migrations have been run on the database
// Run with: cargo run --example run_migrations [up|down [version]]
//
// Examples:
//   cargo run --example run_migrations        # Run all pending migrations up
//   cargo run --example run_migrations up     # Run all pending migrations up
//   cargo run --example run_migrations down   # Revert the last migration
//   cargo run --example run_migrations down 20251130  # Revert to before version 20251130
//
// This program will:
// 1. Connect to the database using DATABASE_URL environment variable
// 2. Check which migrations have already been applied
// 3. Run migrations up or down based on command
// 4. Report the status

use std::process;
use tiny_racing::database::{discover_migrations, get_applied_migrations, Database};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("up");
    let target_version = args.get(2).and_then(|s| s.parse::<i64>().ok());

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing".to_string()
    });

    println!("🔌 Connecting to database...");
    let db = match Database::new(&database_url).await {
        Ok(db) => {
            println!("✅ Successfully connected to database");
            db
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            process::exit(1);
        }
    };

    // Discover all migrations
    // Use CARGO_MANIFEST_DIR to find migrations relative to crate root
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let migrations_dir = std::path::Path::new(manifest_dir).join("migrations");

    if !migrations_dir.exists() {
        eprintln!(
            "❌ Migrations directory not found: {}",
            migrations_dir.display()
        );
        process::exit(1);
    }

    println!("   Looking for migrations in: {}", migrations_dir.display());

    let all_migrations = match discover_migrations(&migrations_dir) {
        Ok(migrations) => migrations,
        Err(e) => {
            eprintln!("❌ Failed to discover migrations: {}", e);
            eprintln!("   Searched in: {}", migrations_dir.display());
            process::exit(1);
        }
    };

    // Check current migration status
    println!("\n📊 Checking migration status...");
    match get_applied_migrations(db.pool()).await {
        Ok(applied) => {
            println!("   Total migrations found: {}", all_migrations.len());
            println!("   Applied migrations: {}", applied.len());
            let pending = all_migrations.len().saturating_sub(applied.len());
            println!("   Pending migrations: {}", pending);

            if !applied.is_empty() {
                println!("\n   Applied migration versions:");
                for version in &applied {
                    if let Some(migration) = all_migrations.iter().find(|m| m.version == *version) {
                        println!("     - {}: {}", version, migration.name);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  Warning: Could not check migration status: {}", e);
            eprintln!("   This might be normal if this is the first run.");
        }
    }

    // Run migrations based on command
    match command {
        "up" | "migrate" => {
            println!("\n🚀 Running migrations up...");
            match db.migrate_up().await {
                Ok(applied) => {
                    if applied.is_empty() {
                        println!("✅ No pending migrations. Database is up to date!");
                    } else {
                        println!("✅ Applied {} migration(s):", applied.len());
                        for migration in &applied {
                            println!("   - {}: {}", migration.version, migration.name);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Migration failed: {}", e);
                    process::exit(1);
                }
            }
        }
        "down" | "revert" => {
            println!("\n⬇️  Reverting migrations...");
            if let Some(version) = target_version {
                println!("   Target version: {}", version);
            } else {
                println!("   Reverting last migration");
            }

            match db.migrate_down(target_version).await {
                Ok(reverted) => {
                    if reverted.is_empty() {
                        println!("✅ No migrations to revert.");
                    } else {
                        println!("✅ Reverted {} migration(s):", reverted.len());
                        for migration in &reverted {
                            println!("   - {}: {}", migration.version, migration.name);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Migration revert failed: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("❌ Unknown command: {}", command);
            eprintln!("Usage: cargo run --example run_migrations [up|down [version]]");
            eprintln!("  up          - Run all pending migrations (default)");
            eprintln!("  down        - Revert the last migration");
            eprintln!("  down <ver>   - Revert to before the specified version");
            process::exit(1);
        }
    }

    // Show final status
    if let Ok(applied) = get_applied_migrations(db.pool()).await {
        println!("\n📊 Final migration status:");
        println!("   Applied migrations: {}", applied.len());
        let pending = all_migrations.len().saturating_sub(applied.len());
        println!("   Pending migrations: {}", pending);
    }
}
