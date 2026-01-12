// Standalone program to seed the database with initial data
// Run with: cargo run --example seed_db
// Run with randomization: cargo run --example seed_db -- randomize
//
// This program seeds the database with static hardcoded data for:
// - Teams, drivers, and cars (decoupled - can add unassigned drivers/cars)
// - Tracks
// When "randomize" parameter is passed, also generates random unassigned cars and drivers

use chrono::NaiveDate;
use fake::faker::name::raw::*;
use fake::locales::*;
use fake::Fake;
use rand::Rng;
use tiny_racing::database::*;

// Static data structures - decoupled
struct TeamSeedData {
    number: i32,
    name: &'static str,
    color: &'static str,
    logo: &'static str,
    pit_efficiency: f32,
}

struct DriverSeedData {
    first_name: &'static str,
    last_name: &'static str,
    date_of_birth: NaiveDate,
    nationality: &'static str,
    gender: &'static str,
    team_number: Option<i32>, // Optional reference to team
    car_number: Option<i32>,  // Optional reference to car
    skill_level: f32,
    stamina: f32,
    weather_tolerance: f32,
    experience: f32,
    consistency: f32,
    focus: f32,
}

struct CarSeedData {
    number: i32,
    team_number: Option<i32>, // Optional reference to team
    handling: f32,
    acceleration: f32,
    top_speed: f32,
    reliability: f32,
    fuel_consumption: f32,
    tire_wear: f32,
}

struct TrackSeedData {
    track_id: &'static str,
    name: &'static str,
    description: Option<&'static str>,
    laps: i32,
    lap_length_km: f32,
}

struct PlayerSeedData {
    username: &'static str,
    email: Option<&'static str>,
    password: &'static str,
}

// Static seed data - decoupled arrays
const TEAMS: &[TeamSeedData] = &[
    TeamSeedData {
        number: 1,
        name: "Red Bull Racing",
        color: "#1E41FF",
        logo: "red_bull_logo.png",
        pit_efficiency: 0.95,
    },
    TeamSeedData {
        number: 2,
        name: "Ferrari",
        color: "#DC143C",
        logo: "ferrari_logo.png",
        pit_efficiency: 0.88,
    },
    TeamSeedData {
        number: 3,
        name: "Mercedes",
        color: "#00D2BE",
        logo: "mercedes_logo.png",
        pit_efficiency: 0.92,
    },
    TeamSeedData {
        number: 4,
        name: "McLaren",
        color: "#FF8700",
        logo: "mclaren_logo.png",
        pit_efficiency: 0.85,
    },
    TeamSeedData {
        number: 5,
        name: "Alpine",
        color: "#0090FF",
        logo: "alpine_logo.png",
        pit_efficiency: 0.80,
    },
];

const CARS: &[CarSeedData] = &[
    // Red Bull cars (team 1)
    CarSeedData {
        number: 11, // team 1 * 10 + 1
        team_number: Some(1),
        handling: 0.92,
        acceleration: 0.94,
        top_speed: 0.96,
        reliability: 0.90,
        fuel_consumption: 0.88,
        tire_wear: 0.85,
    },
    CarSeedData {
        number: 12, // team 1 * 10 + 2
        team_number: Some(1),
        handling: 0.92,
        acceleration: 0.94,
        top_speed: 0.96,
        reliability: 0.90,
        fuel_consumption: 0.88,
        tire_wear: 0.85,
    },
    // Ferrari cars (team 2)
    CarSeedData {
        number: 21, // team 2 * 10 + 1
        team_number: Some(2),
        handling: 0.90,
        acceleration: 0.88,
        top_speed: 0.92,
        reliability: 0.85,
        fuel_consumption: 0.82,
        tire_wear: 0.80,
    },
    CarSeedData {
        number: 22, // team 2 * 10 + 2
        team_number: Some(2),
        handling: 0.90,
        acceleration: 0.88,
        top_speed: 0.92,
        reliability: 0.85,
        fuel_consumption: 0.82,
        tire_wear: 0.80,
    },
    // Mercedes cars (team 3)
    CarSeedData {
        number: 31, // team 3 * 10 + 1
        team_number: Some(3),
        handling: 0.88,
        acceleration: 0.90,
        top_speed: 0.89,
        reliability: 0.93,
        fuel_consumption: 0.85,
        tire_wear: 0.82,
    },
    CarSeedData {
        number: 32, // team 3 * 10 + 2
        team_number: Some(3),
        handling: 0.88,
        acceleration: 0.90,
        top_speed: 0.89,
        reliability: 0.93,
        fuel_consumption: 0.85,
        tire_wear: 0.82,
    },
    // McLaren cars (team 4)
    CarSeedData {
        number: 41, // team 4 * 10 + 1
        team_number: Some(4),
        handling: 0.87,
        acceleration: 0.89,
        top_speed: 0.87,
        reliability: 0.88,
        fuel_consumption: 0.80,
        tire_wear: 0.78,
    },
    CarSeedData {
        number: 42, // team 4 * 10 + 2
        team_number: Some(4),
        handling: 0.87,
        acceleration: 0.89,
        top_speed: 0.87,
        reliability: 0.88,
        fuel_consumption: 0.80,
        tire_wear: 0.78,
    },
    // Alpine cars (team 5)
    CarSeedData {
        number: 51, // team 5 * 10 + 1
        team_number: Some(5),
        handling: 0.80,
        acceleration: 0.82,
        top_speed: 0.81,
        reliability: 0.85,
        fuel_consumption: 0.78,
        tire_wear: 0.75,
    },
    CarSeedData {
        number: 52, // team 5 * 10 + 2
        team_number: Some(5),
        handling: 0.80,
        acceleration: 0.82,
        top_speed: 0.81,
        reliability: 0.85,
        fuel_consumption: 0.78,
        tire_wear: 0.75,
    },
    // Example: Unassigned car (no team)
    CarSeedData {
        number: 99,
        team_number: None,
        handling: 0.75,
        acceleration: 0.75,
        top_speed: 0.75,
        reliability: 0.80,
        fuel_consumption: 0.70,
        tire_wear: 0.70,
    },
];

const DRIVERS: &[DriverSeedData] = &[
    // Red Bull drivers (team 1)
    DriverSeedData {
        first_name: "Max",
        last_name: "Verstappen",
        team_number: Some(1),
        car_number: Some(11),
        date_of_birth: NaiveDate::from_ymd_opt(1997, 9, 30).unwrap(),
        nationality: "Dutch",
        gender: "Male",
        skill_level: 0.95,
        stamina: 0.92,
        weather_tolerance: 0.90,
        experience: 0.88,
        consistency: 0.93,
        focus: 0.94,
    },
    DriverSeedData {
        first_name: "Sergio",
        last_name: "Perez",
        team_number: Some(1),
        car_number: Some(12),
        date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 26).unwrap(),
        nationality: "Mexican",
        gender: "Male",
        skill_level: 0.88,
        stamina: 0.85,
        weather_tolerance: 0.82,
        experience: 0.90,
        consistency: 0.85,
        focus: 0.87,
    },
    // Ferrari drivers (team 2)
    DriverSeedData {
        first_name: "Charles",
        last_name: "Leclerc",
        team_number: Some(2),
        car_number: Some(21),
        date_of_birth: NaiveDate::from_ymd_opt(1997, 10, 16).unwrap(),
        nationality: "Italian",
        gender: "Male",
        skill_level: 0.93,
        stamina: 0.88,
        weather_tolerance: 0.85,
        experience: 0.86,
        consistency: 0.84,
        focus: 0.90,
    },
    DriverSeedData {
        first_name: "Carlos",
        last_name: "Sainz",
        team_number: Some(2),
        car_number: Some(22),
        date_of_birth: NaiveDate::from_ymd_opt(1994, 9, 1).unwrap(),
        nationality: "Spanish",
        gender: "Male",
        skill_level: 0.90,
        stamina: 0.87,
        weather_tolerance: 0.88,
        experience: 0.89,
        consistency: 0.86,
        focus: 0.88,
    },
    // Mercedes drivers (team 3)
    DriverSeedData {
        first_name: "Lewis",
        last_name: "Hamilton",
        team_number: Some(3),
        car_number: Some(31),
        date_of_birth: NaiveDate::from_ymd_opt(1985, 1, 7).unwrap(),
        nationality: "British",
        gender: "Male",
        skill_level: 0.94,
        stamina: 0.90,
        weather_tolerance: 0.92,
        experience: 0.95,
        consistency: 0.91,
        focus: 0.93,
    },
    DriverSeedData {
        first_name: "George",
        last_name: "Russell",
        team_number: Some(3),
        car_number: Some(32),
        date_of_birth: NaiveDate::from_ymd_opt(1998, 2, 15).unwrap(),
        nationality: "British",
        gender: "Male",
        skill_level: 0.89,
        stamina: 0.87,
        weather_tolerance: 0.84,
        experience: 0.82,
        consistency: 0.85,
        focus: 0.88,
    },
    // McLaren drivers (team 4)
    DriverSeedData {
        first_name: "Lando",
        last_name: "Norris",
        team_number: Some(4),
        car_number: Some(41),
        date_of_birth: NaiveDate::from_ymd_opt(1999, 11, 13).unwrap(),
        nationality: "British",
        gender: "Male",
        skill_level: 0.90,
        stamina: 0.88,
        weather_tolerance: 0.85,
        experience: 0.84,
        consistency: 0.87,
        focus: 0.89,
    },
    DriverSeedData {
        first_name: "Oscar",
        last_name: "Piastri",
        team_number: Some(4),
        car_number: Some(42),
        date_of_birth: NaiveDate::from_ymd_opt(2001, 4, 6).unwrap(),
        nationality: "Australian",
        gender: "Male",
        skill_level: 0.85,
        stamina: 0.84,
        weather_tolerance: 0.80,
        experience: 0.75,
        consistency: 0.82,
        focus: 0.86,
    },
    // Alpine drivers (team 5)
    DriverSeedData {
        first_name: "Pierre",
        last_name: "Gasly",
        team_number: Some(5),
        car_number: Some(51),
        date_of_birth: NaiveDate::from_ymd_opt(1996, 2, 7).unwrap(),
        nationality: "French",
        gender: "Male",
        skill_level: 0.86,
        stamina: 0.85,
        weather_tolerance: 0.83,
        experience: 0.84,
        consistency: 0.82,
        focus: 0.85,
    },
    DriverSeedData {
        first_name: "Esteban",
        last_name: "Ocon",
        team_number: Some(5),
        car_number: Some(52),
        date_of_birth: NaiveDate::from_ymd_opt(1996, 9, 17).unwrap(),
        nationality: "French",
        gender: "Male",
        skill_level: 0.84,
        stamina: 0.83,
        weather_tolerance: 0.81,
        experience: 0.83,
        consistency: 0.80,
        focus: 0.83,
    },
    // Example: Unassigned driver (no team, no car)
    DriverSeedData {
        first_name: "Test",
        last_name: "Driver",
        team_number: None,
        car_number: None,
        date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
        nationality: "Unknown",
        gender: "Unknown",
        skill_level: 0.70,
        stamina: 0.70,
        weather_tolerance: 0.70,
        experience: 0.70,
        consistency: 0.70,
        focus: 0.70,
    },
];

const TRACKS: &[TrackSeedData] = &[
    TrackSeedData {
        track_id: "monaco",
        name: "Monaco Grand Prix",
        description: Some("The Monaco Grand Prix is a 3.34 km permanent race track located in Monte Carlo, Monaco known for its narrow streets and high speeds."),
        laps: 78,
        lap_length_km: 3.34,
    },
    TrackSeedData {
        track_id: "bahrain",
        name: "Bahrain International Circuit",
        description: Some("A challenging desert circuit with multiple layout configurations."),
        laps: 57,
        lap_length_km: 5.41,
    },
    TrackSeedData {
        track_id: "bugatti",
        name: "Bugatti Circuit",
        description: Some("The Bugatti Circuit is a 4.14 km permanent race track located in Le Mans, France known for its technical corners and long straights."),
        laps: 5,
        lap_length_km: 4.14,
    },
    TrackSeedData {
        track_id: "le_mans",
        name: "Le Mans Circuit",
        description: Some("The Le Mans Circuit is a 13.629 km long circuit that is part of the 24 Hours of Le Mans race. It is a 10-turn circuit that is known for its high speeds and technical challenges."),
        laps: 5,
        lap_length_km: 13.629,
    },
    TrackSeedData {
        track_id: "circle",
        name: "Circle Track",
        description: Some("The Circle Track is a 1.0 km long circuit that is part of the British Grand Prix race. It is a 10-turn circuit that is known for its high speeds and technical challenges."),
        laps: 10,
        lap_length_km: 1.0,
    },
    TrackSeedData {
        track_id: "test",
        name: "Test Track",
        description: Some("A test track for testing car physics."),
        laps: 50,
        lap_length_km: 2.5,
    },
];

const PLAYERS: &[PlayerSeedData] = &[PlayerSeedData {
    username: "antoine",
    email: Some("antoine@example.com"),
    password: "antoine",
}];

const NATIONALITIES: &[&str] = &[
    "American",
    "British",
    "French",
    "German",
    "Italian",
    "Spanish",
    "Dutch",
    "Belgian",
    "Australian",
    "Canadian",
    "Brazilian",
    "Mexican",
    "Japanese",
    "Chinese",
    "Korean",
    "Swedish",
    "Finnish",
    "Norwegian",
    "Danish",
    "Swiss",
    "Austrian",
];

const GENDERS: &[&str] = &["Male", "Female", "Non-binary"];

fn generate_random_car(rng: &mut impl Rng, start_number: i32) -> CarSeedData {
    CarSeedData {
        number: start_number,
        team_number: None, // Always unassigned
        handling: rng.random_range(0.5..=0.95),
        acceleration: rng.random_range(0.5..=0.95),
        top_speed: rng.random_range(0.5..=0.95),
        reliability: rng.random_range(0.5..=0.95),
        fuel_consumption: rng.random_range(0.5..=0.95),
        tire_wear: rng.random_range(0.5..=0.95),
    }
}

// Helper struct for generated driver data (uses owned Strings)
struct GeneratedDriverData {
    first_name: String,
    last_name: String,
    date_of_birth: NaiveDate,
    nationality: String,
    gender: String,
    skill_level: f32,
    stamina: f32,
    weather_tolerance: f32,
    experience: f32,
    consistency: f32,
    focus: f32,
}

fn generate_random_driver(rng: &mut impl Rng) -> GeneratedDriverData {
    // Select random nationality and gender first
    let nationality = NATIONALITIES[rng.random_range(0..NATIONALITIES.len())].to_string();
    let gender = GENDERS[rng.random_range(0..GENDERS.len())].to_string();

    // Generate names based on nationality locale
    // Using only locales available in fake crate 4.4.0:
    // AR_SA, CY_GB, DE_DE, EN, FR_FR, IT_IT, JA_JP, PT_BR, PT_PT, ZH_CN, ZH_TW
    // See: https://docs.rs/fake/latest/fake/locales/index.html
    // Note: Each locale is a different type, so we generate names directly in each match arm
    let (first_name, last_name) = match nationality.as_str() {
        "French" | "Belgian" => (
            FirstName(FR_FR).fake::<String>(),
            LastName(FR_FR).fake::<String>(),
        ),
        "German" | "Austrian" | "Swiss" => (
            FirstName(DE_DE).fake::<String>(),
            LastName(DE_DE).fake::<String>(),
        ),
        "Italian" => (
            FirstName(IT_IT).fake::<String>(),
            LastName(IT_IT).fake::<String>(),
        ),
        "Japanese" => (
            FirstName(JA_JP).fake::<String>(),
            LastName(JA_JP).fake::<String>(),
        ),
        "Chinese" => (
            FirstName(ZH_CN).fake::<String>(),
            LastName(ZH_CN).fake::<String>(),
        ),
        "Brazilian" => (
            FirstName(PT_BR).fake::<String>(),
            LastName(PT_BR).fake::<String>(),
        ),
        "Portuguese" => (
            FirstName(PT_PT).fake::<String>(),
            LastName(PT_PT).fake::<String>(),
        ),
        // For nationalities without matching locales, fall back to EN
        "Spanish" | "Dutch" | "Korean" | "Swedish" | "Finnish" | "Norwegian" | "Danish" | _ => (
            FirstName(EN).fake::<String>(),
            LastName(EN).fake::<String>(),
        ),
    };

    // Generate a random date of birth between 1985 and 2005
    let year = rng.random_range(1985..=2005);
    let month = rng.random_range(1..=12);
    let day = rng.random_range(1..=28); // Use 28 to avoid month-specific day issues

    GeneratedDriverData {
        first_name,
        last_name,
        date_of_birth: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
        nationality,
        gender,
        skill_level: rng.random_range(0.5..=0.95),
        stamina: rng.random_range(0.5..=0.95),
        weather_tolerance: rng.random_range(0.5..=0.95),
        experience: rng.random_range(0.5..=0.95),
        consistency: rng.random_range(0.5..=0.95),
        focus: rng.random_range(0.5..=0.95),
    }
}

async fn seed_random_cars_and_drivers(
    db: &Database,
    num_cars: usize,
    num_drivers: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    // Get the maximum car number to start from
    let existing_cars = list_cars(db.pool(), 10000, 0).await?;
    let max_car_number = existing_cars.iter().map(|c| c.number).max().unwrap_or(0);
    let mut next_car_number = max_car_number + 1;

    println!("\n=== Seeding Random Unassigned Cars ===");
    for _ in 0..num_cars {
        // Find the next available car number
        while get_car_by_number(db.pool(), next_car_number)
            .await?
            .is_some()
        {
            next_car_number += 1;
        }

        let car_data = generate_random_car(&mut rng, next_car_number);

        let car = create_car(
            db.pool(),
            CreateCarRequest {
                number: car_data.number,
                team_id: None, // Always unassigned
                handling: car_data.handling,
                acceleration: car_data.acceleration,
                top_speed: car_data.top_speed,
                reliability: car_data.reliability,
                fuel_consumption: car_data.fuel_consumption,
                tire_wear: car_data.tire_wear,
                base_performance: 1.0,
            },
        )
        .await?;

        println!(
            "Created random unassigned car #{} (ID: {}) - handling: {:.2}, acceleration: {:.2}, top_speed: {:.2}",
            car.number, car.id, car.handling, car.acceleration, car.top_speed
        );

        next_car_number += 1;
    }

    println!("\n=== Seeding Random Unassigned Drivers ===");
    for _ in 0..num_drivers {
        let driver_data = generate_random_driver(&mut rng);

        // Check if driver already exists (unlikely but possible with random generation)
        if get_driver_by_first_and_last_name(
            db.pool(),
            driver_data.first_name.clone(),
            driver_data.last_name.clone(),
        )
        .await?
        .is_some()
        {
            println!(
                "Driver '{} {}' already exists, skipping",
                driver_data.first_name, driver_data.last_name
            );
            continue;
        }

        let driver = create_driver(
            db.pool(),
            CreateDriverRequest {
                first_name: driver_data.first_name.clone(),
                last_name: driver_data.last_name.clone(),
                date_of_birth: driver_data.date_of_birth,
                nationality: driver_data.nationality.clone(),
                gender: driver_data.gender.clone(),
                skill_level: driver_data.skill_level,
                stamina: driver_data.stamina,
                weather_tolerance: driver_data.weather_tolerance,
                experience: driver_data.experience,
                consistency: driver_data.consistency,
                focus: driver_data.focus,
                team_id: None, // Always unassigned
                car_id: None,  // Always unassigned
            },
        )
        .await?;

        println!(
            "Created random unassigned driver: {} {} ({}, {}) (ID: {}) - skill: {:.2}, stamina: {:.2}",
            driver.first_name,
            driver.last_name,
            driver.nationality,
            driver.gender,
            driver.id,
            driver.skill_level,
            driver.stamina
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for randomize parameter
    let args: Vec<String> = std::env::args().collect();
    let randomize = args.iter().any(|arg| arg == "randomize");

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://tiny_racing:tiny_racing_password@localhost:5432/tiny_racing".to_string()
    });

    println!("Connecting to database...");
    let db = Database::new(&database_url).await?;

    println!("Running migrations...");
    db.migrate().await?;
    println!("Migrations completed!");

    // Seed players
    println!("\n=== Seeding Players ===");
    for player_data in PLAYERS {
        // Check if player already exists
        if let Some(existing) = get_player_by_username(db.pool(), player_data.username).await? {
            println!("Player '{}' already exists, skipping", existing.username);
            continue;
        }
        let player = create_player(
            db.pool(),
            CreatePlayerRequest {
                username: player_data.username.to_string(),
                password: player_data.password.to_string(),
                email: player_data.email.map(|s| s.to_string()),
            },
        )
        .await
        .map_err(|e| {
            eprintln!("Error creating player: {}", e);
            e
        })?;
        println!("Created player: {} (ID: {})", player.username, player.id);
    }

    // Seed tracks first (no dependencies)
    println!("\n=== Seeding Tracks ===");
    for track in TRACKS {
        // Check if track already exists
        if let Some(_existing) = get_track_by_track_id(db.pool(), track.track_id).await? {
            println!("Track '{}' already exists, skipping", track.name);
            continue;
        }

        let created = create_track(
            db.pool(),
            CreateTrackRequest {
                track_id: track.track_id.to_string(),
                name: track.name.to_string(),
                description: track.description.map(|s| s.to_string()),
                laps: track.laps,
                lap_length_km: track.lap_length_km,
            },
        )
        .await?;
        println!("Created track: {} (ID: {})", created.name, created.id);
    }

    // Seed teams
    println!("\n=== Seeding Teams ===");
    let mut team_map: std::collections::HashMap<i32, uuid::Uuid> = std::collections::HashMap::new();

    for team_data in TEAMS {
        // Check if team already exists
        if let Some(existing) = get_team_by_number(db.pool(), team_data.number).await? {
            println!(
                "Team '{}' (#{}) already exists, skipping",
                existing.name, existing.number
            );
            team_map.insert(team_data.number, existing.id);
            continue;
        }

        let team = create_team(
            db.pool(),
            CreateTeamRequest {
                number: Some(team_data.number),
                player_id: None,
                name: team_data.name.to_string(),
                logo: Some(team_data.logo.to_string()),
                color: team_data.color.to_string(),
                pit_efficiency: Some(team_data.pit_efficiency),
            },
        )
        .await?;
        println!(
            "Created team: {} (#{}) (ID: {})",
            team.name, team.number, team.id
        );
        team_map.insert(team_data.number, team.id);
    }

    // Seed cars (can be assigned to teams or unassigned)
    println!("\n=== Seeding Cars ===");
    let mut car_map: std::collections::HashMap<i32, uuid::Uuid> = std::collections::HashMap::new();

    for car_data in CARS {
        // Check if car already exists
        if let Some(existing) = get_car_by_number(db.pool(), car_data.number).await? {
            println!("Car #{} already exists, skipping", car_data.number);
            car_map.insert(car_data.number, existing.id);
            continue;
        }

        // Get team_id if car is assigned to a team
        let team_id = car_data
            .team_number
            .and_then(|num| team_map.get(&num).copied());

        let car = create_car(
            db.pool(),
            CreateCarRequest {
                number: car_data.number,
                team_id,
                handling: car_data.handling,
                acceleration: car_data.acceleration,
                top_speed: car_data.top_speed,
                reliability: car_data.reliability,
                fuel_consumption: car_data.fuel_consumption,
                tire_wear: car_data.tire_wear,
                base_performance: 1.0,
            },
        )
        .await?;
        let team_info = if let Some(team_num) = car_data.team_number {
            format!(" for team #{}", team_num)
        } else {
            " (unassigned)".to_string()
        };
        println!("Created car #{}{} (ID: {})", car.number, team_info, car.id);
        car_map.insert(car_data.number, car.id);
    }

    // Seed drivers (can be assigned to teams/cars or unassigned)
    println!("\n=== Seeding Drivers ===");

    for driver_data in DRIVERS {
        // Check if driver already exists
        if let Some(existing) = get_driver_by_first_and_last_name(
            db.pool(),
            driver_data.first_name.to_string(),
            driver_data.last_name.to_string(),
        )
        .await?
        {
            println!(
                "Driver '{} {}' already exists, skipping",
                existing.first_name, existing.last_name
            );
            continue;
        }

        // Get team_id if driver is assigned to a team
        let team_id = driver_data
            .team_number
            .and_then(|num| team_map.get(&num).copied());

        // Get car_id if driver is assigned to a car
        let car_id = driver_data
            .car_number
            .and_then(|num| car_map.get(&num).copied());

        let driver = create_driver(
            db.pool(),
            CreateDriverRequest {
                first_name: driver_data.first_name.to_string(),
                last_name: driver_data.last_name.to_string(),
                date_of_birth: driver_data.date_of_birth,
                nationality: driver_data.nationality.to_string(),
                gender: driver_data.gender.to_string(),
                skill_level: driver_data.skill_level,
                stamina: driver_data.stamina,
                weather_tolerance: driver_data.weather_tolerance,
                experience: driver_data.experience,
                consistency: driver_data.consistency,
                focus: driver_data.focus,
                team_id,
                car_id,
            },
        )
        .await?;

        let assignment_info = match (driver_data.team_number, driver_data.car_number) {
            (Some(team_num), Some(car_num)) => format!(" (team #{}, car #{})", team_num, car_num),
            (Some(team_num), None) => format!(" (team #{}, no car)", team_num),
            (None, Some(car_num)) => format!(" (no team, car #{})", car_num),
            (None, None) => " (unassigned)".to_string(),
        };
        println!(
            "Created driver: {} {}{} (ID: {})",
            driver.first_name, driver.last_name, assignment_info, driver.id
        );
    }

    // Seed random cars and drivers if randomize parameter is passed
    if randomize {
        println!("\n=== Randomization Mode Enabled ===");
        seed_random_cars_and_drivers(&db, 20, 30).await?;
    }

    // Print summary
    println!("\n=== Seeding Summary ===");
    let teams = list_teams(db.pool(), 10000, 0).await?;
    let cars = list_cars(db.pool(), 10000, 0).await?;
    let drivers = list_drivers(db.pool(), 10000, 0).await?;
    let tracks = list_tracks(db.pool(), 10000, 0).await?;

    let unassigned_cars = list_unassigned_cars(db.pool(), 10000, 0).await?;
    let unassigned_drivers = list_unassigned_drivers(db.pool(), 10000, 0).await?;

    println!("Total teams in database: {}", teams.len());
    println!(
        "Total cars in database: {} ({} unassigned)",
        cars.len(),
        unassigned_cars.len()
    );
    println!(
        "Total drivers in database: {} ({} unassigned)",
        drivers.len(),
        unassigned_drivers.len()
    );
    println!("Total tracks in database: {}", tracks.len());

    println!("\nSeeding completed successfully!");

    Ok(())
}
