# Database Setup Guide

This document describes the database persistence layer added to the Tiny Racing simulator.

## Overview

The database module provides a complete persistence layer using:
- **SQLX** for async database operations
- **PostgreSQL** as the database (running in Docker)
- **Sequential migrations** for schema management
- **Type-safe queries** with compile-time checking

## Quick Start

### 1. Start PostgreSQL Database

```bash
# From the project root
docker-compose up -d
```

This starts a PostgreSQL 16 container with:
- Database: `tiny_racing`
- User: `tiny_racing`
- Password: `tiny_racing_password`
- Port: `5432`

### 2. Set Environment Variable

```bash
export DATABASE_URL=postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing
```

Or create a `.env` file in the `server/` directory:
```
DATABASE_URL=postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing
```

### 3. Initialize Database

```rust
use tiny_racing::database::Database;

let db = Database::new(&std::env::var("DATABASE_URL")?).await?;
db.migrate().await?; // Runs all pending migrations
```

### 4. Run Example

```bash
cd server
cargo run --example database_example
```

## Database Schema

### Tables

1. **teams** - Racing teams
   - `id` (UUID), `number`, `name`, `logo`, `color`, `pit_efficiency`
   - Timestamps: `created_at`, `updated_at`

2. **drivers** - Driver profiles
   - `id` (UUID), `name`, `skill_level`, `stamina`, `weather_tolerance`, `experience`, `consistency`, `focus`
   - Timestamps: `created_at`, `updated_at`

3. **cars** - Car configurations
   - `id` (UUID), `number`, `team_id`, `driver_id`
   - Stats: `handling`, `acceleration`, `top_speed`, `reliability`, `fuel_consumption`, `tire_wear`, `base_performance`
   - Timestamps: `created_at`, `updated_at`

4. **tracks** - Race track configurations
   - `id` (UUID), `track_id`, `name`, `description`, `laps`, `lap_length_km`, `svg_start_offset`
   - Timestamps: `created_at`, `updated_at`

5. **players** - Player/user accounts
   - `id` (UUID), `username`, `email`
   - Timestamps: `created_at`, `updated_at`

### Features

- **UUID primary keys** for all entities
- **Automatic timestamps** with triggers
- **Foreign key constraints** (cars reference teams and drivers)
- **Indexes** on frequently queried columns
- **Data validation** with CHECK constraints

## Available Operations

### Teams
- `create_team()` - Create a new team
- `get_team_by_id()` - Get team by UUID
- `get_team_by_number()` - Get team by number
- `list_teams()` - List all teams
- `update_team()` - Update team details
- `delete_team()` - Delete a team

### Drivers
- `create_driver()` - Create a new driver
- `get_driver_by_id()` - Get driver by UUID
- `get_driver_by_name()` - Get driver by name
- `list_drivers()` - List all drivers
- `update_driver()` - Update driver stats
- `delete_driver()` - Delete a driver

### Cars
- `create_car()` - Create a new car
- `get_car_by_id()` - Get car by UUID
- `get_car_by_number()` - Get car by number
- `list_cars()` - List all cars
- `list_cars_by_team()` - List cars for a team
- `update_car()` - Update car stats
- `delete_car()` - Delete a car

### Tracks
- `create_track()` - Create a new track
- `get_track_by_id()` - Get track by UUID
- `get_track_by_track_id()` - Get track by track_id string
- `list_tracks()` - List all tracks
- `update_track()` - Update track details
- `delete_track()` - Delete a track

### Players
- `create_player()` - Create a new player
- `get_player_by_id()` - Get player by UUID
- `get_player_by_username()` - Get player by username
- `list_players()` - List all players
- `update_player()` - Update player details
- `delete_player()` - Delete a player

## Migrations

Migrations are stored in `server/migrations/` and are automatically applied when `Database::migrate()` is called.

### Creating a New Migration

1. Create a new SQL file: `migrations/YYYYMMDDHHMMSS_description.sql`
2. Write your migration SQL (CREATE TABLE, ALTER TABLE, etc.)
3. Migrations are applied in chronological order

Example migration filename: `20240115120000_add_race_history.sql`

### Migration Features

- **Sequential execution** - Migrations run in timestamp order
- **Idempotent** - SQLX tracks applied migrations
- **Automatic** - Run `db.migrate().await?` to apply all pending migrations

## Module Structure

```
server/src/database/
├── mod.rs           # Module exports
├── connection.rs    # Database connection and migration management
├── models.rs        # Database models and DTOs
├── queries.rs       # All database query functions
└── README.md        # Detailed usage documentation
```

## Integration with Existing Code

The database models (`TeamDb`, `DriverDb`, etc.) are separate from the runtime models (`Team`, `Driver`, etc.) in `models/`. You'll need to convert between them when loading/saving data:

```rust
// Convert from database model to runtime model
let runtime_team = Team {
    number: db_team.number as u32,
    name: db_team.name,
    logo: db_team.logo,
    color: db_team.color,
    pit_efficiency: db_team.pit_efficiency,
};
```

## Error Handling

All database operations return `Result<T, sqlx::Error>`. The `DatabaseError` enum is used for connection and migration errors.

## Next Steps

1. Integrate database loading into race initialization
2. Add race history/result persistence
3. Add player-team associations
4. Add race session management
5. Implement data synchronization between runtime and database

