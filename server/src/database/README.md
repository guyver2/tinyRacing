# Database Module

This module provides database persistence for the Tiny Racing simulator using SQLX with PostgreSQL.

## Setup

1. Start the PostgreSQL database using Docker Compose:
   ```bash
   docker-compose up -d
   ```

2. Set the `DATABASE_URL` environment variable:
   ```bash
   export DATABASE_URL=postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing
   ```

3. Initialize the database and run migrations:
   ```rust
   use tiny_racing::database::Database;

   let db = Database::new(&std::env::var("DATABASE_URL")?).await?;
   db.migrate().await?;
   ```

## Usage Examples

### Creating a Team

```rust
use tiny_racing::database::{Database, CreateTeamRequest, create_team};

let db = Database::new(&database_url).await?;
let team = create_team(
    db.pool(),
    CreateTeamRequest {
        number: 1,
        name: "Red Bull Racing".to_string(),
        logo: "redbull.svg".to_string(),
        color: "#1E41FF".to_string(),
        pit_efficiency: 0.95,
    },
).await?;
```

### Creating a Driver

```rust
use tiny_racing::database::{CreateDriverRequest, create_driver};

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
).await?;
```

### Creating a Car

```rust
use tiny_racing::database::{CreateCarRequest, create_car};

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
).await?;
```

### Querying Data

```rust
use tiny_racing::database::{get_team_by_id, list_cars, list_cars_by_team};

// Get a team by ID
let team = get_team_by_id(db.pool(), team_id).await?;

// List all cars
let cars = list_cars(db.pool()).await?;

// List cars for a specific team
let team_cars = list_cars_by_team(db.pool(), team_id).await?;
```

## Database Schema

The database includes the following tables:

- **teams**: Racing teams with pit efficiency
- **drivers**: Driver profiles with skill attributes
- **cars**: Cars with stats and references to teams and drivers
- **tracks**: Race track configurations
- **players**: Player/user accounts

All tables include `id` (UUID), `created_at`, and `updated_at` timestamps.

## Migrations

Migrations are stored in the `migrations/` directory and are automatically applied when `Database::migrate()` is called. To add a new migration:

1. Create a new SQL file: `migrations/YYYYMMDDHHMMSS_description.sql`
2. Write your migration SQL
3. The migration will be applied automatically on the next `migrate()` call

