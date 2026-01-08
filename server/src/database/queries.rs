use super::models::*;
use crate::auth::hash_password;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// ========== Team Queries ==========

pub async fn create_team(pool: &PgPool, request: CreateTeamRequest) -> Result<TeamDb, sqlx::Error> {
    // If number is not provided, calculate the next available number
    let team_number = if let Some(number) = request.number {
        number
    } else {
        // Get the maximum team number and add 1
        let max_number: Option<i32> = sqlx::query_scalar("SELECT MAX(number) FROM team")
            .fetch_optional(pool)
            .await?;

        max_number.map(|n| n + 1).unwrap_or(1)
    };

    // If pit_efficiency is not provided, generate a random value between 0.4 and 0.8 (inclusive)
    let pit_efficiency = if let Some(efficiency) = request.pit_efficiency {
        efficiency
    } else {
        use rand::Rng;
        let mut rng = rand::rng();
        // Use 0.4..0.81 to ensure 0.8 can be generated (range is exclusive on upper bound)
        rng.random_range(0.4..0.81)
    };

    // Default cash value for new teams is 500
    let cash = 500;

    // Use empty string if logo is not provided
    let logo = request.logo.unwrap_or_default();

    let team = sqlx::query_as::<_, TeamDb>(
        // this query check against the database schema for the correct types at compile time query_as!
        r#"
        INSERT INTO team (number, name, logo, color, pit_efficiency, cash, player_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(team_number)
    .bind(request.name)
    .bind(logo)
    .bind(request.color)
    .bind(pit_efficiency)
    .bind(cash)
    .bind(request.player_id)
    .fetch_one(pool)
    .await?;

    Ok(team)
}

pub async fn get_team_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TeamDb>, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>("SELECT * FROM team WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(team)
}

pub async fn get_team_by_number(pool: &PgPool, number: i32) -> Result<Option<TeamDb>, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>("SELECT * FROM team WHERE number = $1")
        .bind(number)
        .fetch_optional(pool)
        .await?;

    Ok(team)
}

pub async fn list_teams(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<TeamDb>, sqlx::Error> {
    let teams =
        sqlx::query_as::<_, TeamDb>("SELECT * FROM team ORDER BY number LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

    Ok(teams)
}

pub async fn list_teams_by_player(
    pool: &PgPool,
    player_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<TeamDb>, sqlx::Error> {
    let teams = sqlx::query_as::<_, TeamDb>(
        "SELECT * FROM team WHERE player_id = $1 ORDER BY number LIMIT $2 OFFSET $3",
    )
    .bind(player_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(teams)
}

pub async fn get_team_by_player(
    pool: &PgPool,
    player_id: Uuid,
) -> Result<Option<TeamDb>, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>("SELECT * FROM team WHERE player_id = $1 LIMIT 1")
        .bind(player_id)
        .fetch_optional(pool)
        .await?;

    Ok(team)
}

pub async fn update_team(
    pool: &PgPool,
    id: Uuid,
    request: CreateTeamRequest,
) -> Result<TeamDb, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>(
        r#"
        UPDATE team
        SET number = $2, name = $3, logo = $4, color = $5, pit_efficiency = $6, player_id = $7, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.number)
    .bind(request.name)
    .bind(request.logo)
    .bind(request.color)
    .bind(request.pit_efficiency)
    .bind(request.player_id)
    .fetch_one(pool)
    .await?;

    Ok(team)
}

pub async fn delete_team(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM team WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn count_drivers_by_team(pool: &PgPool, team_id: Uuid) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM driver WHERE team_id = $1")
        .bind(team_id)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

pub async fn update_team_cash(
    pool: &PgPool,
    team_id: Uuid,
    new_cash: i32,
) -> Result<TeamDb, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>(
        r#"
        UPDATE team
        SET cash = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(team_id)
    .bind(new_cash)
    .fetch_one(pool)
    .await?;

    Ok(team)
}

pub async fn list_ai_teams_not_registered_for_race(
    pool: &PgPool,
    race_id: Uuid,
    limit: i64,
) -> Result<Vec<TeamDb>, sqlx::Error> {
    let teams = sqlx::query_as::<_, TeamDb>(
        r#"
        SELECT * FROM team 
        WHERE player_id IS NULL 
        AND id NOT IN (
            SELECT team_id FROM registration WHERE race_id = $1
        )
        ORDER BY number
        LIMIT $2
        "#,
    )
    .bind(race_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(teams)
}

// ========== Driver Queries ==========

pub async fn create_driver(
    pool: &PgPool,
    request: CreateDriverRequest,
) -> Result<DriverDb, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>(
        r#"
        INSERT INTO driver (first_name, last_name, date_of_birth, nationality, gender, skill_level, stamina, weather_tolerance, experience, consistency, focus, team_id, car_id, total_exp, spent_exp)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING *
        "#,
    )
    .bind(request.first_name)
    .bind(request.last_name)
    .bind(request.date_of_birth)
    .bind(request.nationality)
    .bind(request.gender)
    .bind(request.skill_level)
    .bind(request.stamina)
    .bind(request.weather_tolerance)
    .bind(request.experience)
    .bind(request.consistency)
    .bind(request.focus)
    .bind(request.team_id)
    .bind(request.car_id)
    .bind(0i32) // total_exp defaults to 0
    .bind(0i32) // spent_exp defaults to 0
    .fetch_one(pool)
    .await?;

    Ok(driver)
}

pub async fn get_driver_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DriverDb>, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>("SELECT * FROM driver WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(driver)
}

pub async fn get_driver_by_first_and_last_name(
    pool: &PgPool,
    first_name: String,
    last_name: String,
) -> Result<Option<DriverDb>, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>(
        "SELECT * FROM driver WHERE first_name = $1 AND last_name = $2",
    )
    .bind(first_name)
    .bind(last_name)
    .fetch_optional(pool)
    .await?;

    Ok(driver)
}

pub async fn list_drivers(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<DriverDb>, sqlx::Error> {
    let drivers = sqlx::query_as::<_, DriverDb>(
        "SELECT * FROM driver ORDER BY last_name, first_name LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(drivers)
}

pub async fn list_unassigned_drivers(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<DriverDb>, sqlx::Error> {
    let drivers = sqlx::query_as::<_, DriverDb>(
        "SELECT * FROM driver WHERE team_id IS NULL ORDER BY last_name, first_name LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(drivers)
}

pub async fn list_drivers_by_team(
    pool: &PgPool,
    team_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<DriverDb>, sqlx::Error> {
    let drivers = sqlx::query_as::<_, DriverDb>(
        "SELECT * FROM driver WHERE team_id = $1 ORDER BY last_name, first_name LIMIT $2 OFFSET $3",
    )
    .bind(team_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(drivers)
}

pub async fn update_driver(
    pool: &PgPool,
    id: Uuid,
    request: CreateDriverRequest,
) -> Result<DriverDb, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>(
        r#"
        UPDATE driver
        SET first_name = $2, last_name = $3, date_of_birth = $4, nationality = $5, gender = $6, skill_level = $7, stamina = $8, weather_tolerance = $9,
            experience = $10, consistency = $11, focus = $12, team_id = $13, car_id = $14, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.first_name)
    .bind(request.last_name)
    .bind(request.date_of_birth)
    .bind(request.nationality)
    .bind(request.gender)
    .bind(request.skill_level)
    .bind(request.stamina)
    .bind(request.weather_tolerance)
    .bind(request.experience)
    .bind(request.consistency)
    .bind(request.focus)
    .bind(request.team_id)
    .bind(request.car_id)
    .fetch_one(pool)
    .await?;

    Ok(driver)
}

pub async fn delete_driver(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM driver WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn assign_driver_to_team(
    pool: &PgPool,
    driver_id: Uuid,
    team_id: Uuid,
) -> Result<DriverDb, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>(
        r#"
        UPDATE driver
        SET team_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(driver_id)
    .bind(team_id)
    .fetch_one(pool)
    .await?;

    Ok(driver)
}

pub async fn assign_driver_to_car(
    pool: &PgPool,
    driver_id: Uuid,
    car_id: Option<Uuid>,
) -> Result<DriverDb, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>(
        r#"
        UPDATE driver
        SET car_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(driver_id)
    .bind(car_id)
    .fetch_one(pool)
    .await?;

    Ok(driver)
}

/// Level up a driver by spending experience points
/// Increases a stat by 0.1 and spends 100 experience points
/// Stats are capped at 1.0 using LEAST() in the SQL query
pub async fn level_up_driver(
    pool: &PgPool,
    driver_id: Uuid,
    stat_name: &str,
) -> Result<DriverDb, sqlx::Error> {
    // First, get the current driver to check current stat values
    let driver = get_driver_by_id(pool, driver_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    // Check if the stat is already at max (1.0)
    let current_value = match stat_name {
        "skill_level" => driver.skill_level,
        "stamina" => driver.stamina,
        "weather_tolerance" => driver.weather_tolerance,
        "experience" => driver.experience,
        "consistency" => driver.consistency,
        "focus" => driver.focus,
        _ => return Err(sqlx::Error::Protocol("Invalid stat name".into())),
    };

    // Check if stat is already at max (1.0) - only prevent if already at max
    if current_value >= 1.0 {
        return Err(sqlx::Error::Protocol(
            format!(
                "{} is already at maximum (1.0) and cannot be increased further",
                stat_name
            )
            .into(),
        ));
    }

    // Note: We allow leveling up even if adding 0.1 would exceed 1.0
    // The LEAST() function in the SQL query will cap it at 1.0

    // Build the update query dynamically based on stat name
    let update_query = match stat_name {
        "skill_level" => "UPDATE driver SET skill_level = LEAST(skill_level + 0.1, 1.0), spent_exp = spent_exp + 100, updated_at = NOW() WHERE id = $1 RETURNING *",
        "stamina" => "UPDATE driver SET stamina = LEAST(stamina + 0.1, 1.0), spent_exp = spent_exp + 100, updated_at = NOW() WHERE id = $1 RETURNING *",
        "weather_tolerance" => "UPDATE driver SET weather_tolerance = LEAST(weather_tolerance + 0.1, 1.0), spent_exp = spent_exp + 100, updated_at = NOW() WHERE id = $1 RETURNING *",
        "experience" => "UPDATE driver SET experience = LEAST(experience + 0.1, 1.0), spent_exp = spent_exp + 100, updated_at = NOW() WHERE id = $1 RETURNING *",
        "consistency" => "UPDATE driver SET consistency = LEAST(consistency + 0.1, 1.0), spent_exp = spent_exp + 100, updated_at = NOW() WHERE id = $1 RETURNING *",
        "focus" => "UPDATE driver SET focus = LEAST(focus + 0.1, 1.0), spent_exp = spent_exp + 100, updated_at = NOW() WHERE id = $1 RETURNING *",
        _ => return Err(sqlx::Error::Protocol("Invalid stat name".into())),
    };

    let updated_driver = sqlx::query_as::<_, DriverDb>(update_query)
        .bind(driver_id)
        .fetch_one(pool)
        .await?;

    Ok(updated_driver)
}

// ========== Car Queries ==========

pub async fn create_car(pool: &PgPool, request: CreateCarRequest) -> Result<CarDb, sqlx::Error> {
    let car = sqlx::query_as::<_, CarDb>(
        r#"
        INSERT INTO car (number, team_id, handling, acceleration, top_speed,
                         reliability, fuel_consumption, tire_wear, base_performance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(request.number)
    .bind(request.team_id)
    .bind(request.handling)
    .bind(request.acceleration)
    .bind(request.top_speed)
    .bind(request.reliability)
    .bind(request.fuel_consumption)
    .bind(request.tire_wear)
    .bind(request.base_performance)
    .fetch_one(pool)
    .await?;

    Ok(car)
}

pub async fn get_car_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CarDb>, sqlx::Error> {
    let car = sqlx::query_as::<_, CarDb>("SELECT * FROM car WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(car)
}

pub async fn get_car_by_number(pool: &PgPool, number: i32) -> Result<Option<CarDb>, sqlx::Error> {
    let car = sqlx::query_as::<_, CarDb>("SELECT * FROM car WHERE number = $1")
        .bind(number)
        .fetch_optional(pool)
        .await?;

    Ok(car)
}

pub async fn list_cars(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<CarDb>, sqlx::Error> {
    let cars = sqlx::query_as::<_, CarDb>("SELECT * FROM car ORDER BY number LIMIT $1 OFFSET $2")
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(cars)
}

pub async fn list_unassigned_cars(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<CarDb>, sqlx::Error> {
    let cars = sqlx::query_as::<_, CarDb>(
        "SELECT * FROM car WHERE team_id IS NULL ORDER BY number LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(cars)
}

pub async fn list_cars_by_team(
    pool: &PgPool,
    team_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<CarDb>, sqlx::Error> {
    let cars = sqlx::query_as::<_, CarDb>(
        "SELECT * FROM car WHERE team_id = $1 ORDER BY number LIMIT $2 OFFSET $3",
    )
    .bind(team_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(cars)
}

pub async fn update_car(
    pool: &PgPool,
    id: Uuid,
    request: CreateCarRequest,
) -> Result<CarDb, sqlx::Error> {
    let car = sqlx::query_as::<_, CarDb>(
        r#"
        UPDATE car
        SET number = $2, team_id = $3, handling = $4, acceleration = $5,
            top_speed = $6, reliability = $7, fuel_consumption = $8, tire_wear = $9,
            base_performance = $10, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.number)
    .bind(request.team_id)
    .bind(request.handling)
    .bind(request.acceleration)
    .bind(request.top_speed)
    .bind(request.reliability)
    .bind(request.fuel_consumption)
    .bind(request.tire_wear)
    .bind(request.base_performance)
    .fetch_one(pool)
    .await?;

    Ok(car)
}

pub async fn delete_car(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM car WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn assign_car_to_team(
    pool: &PgPool,
    car_id: Uuid,
    team_id: Uuid,
) -> Result<CarDb, sqlx::Error> {
    let car = sqlx::query_as::<_, CarDb>(
        r#"
        UPDATE car
        SET team_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(car_id)
    .bind(team_id)
    .fetch_one(pool)
    .await?;

    Ok(car)
}

pub async fn count_cars_by_team(pool: &PgPool, team_id: Uuid) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM car WHERE team_id = $1")
        .bind(team_id)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

// ========== Track Queries ==========

pub async fn create_track(
    pool: &PgPool,
    request: CreateTrackRequest,
) -> Result<TrackDb, sqlx::Error> {
    let track = sqlx::query_as::<_, TrackDb>(
        r#"
        INSERT INTO track (track_id, name, description, laps, lap_length_km)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(request.track_id)
    .bind(request.name)
    .bind(request.description)
    .bind(request.laps)
    .bind(request.lap_length_km)
    .fetch_one(pool)
    .await?;

    Ok(track)
}

pub async fn get_track_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TrackDb>, sqlx::Error> {
    let track = sqlx::query_as::<_, TrackDb>("SELECT * FROM track WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(track)
}

pub async fn get_track_by_track_id(
    pool: &PgPool,
    track_id: &str,
) -> Result<Option<TrackDb>, sqlx::Error> {
    let track = sqlx::query_as::<_, TrackDb>("SELECT * FROM track WHERE track_id = $1")
        .bind(track_id)
        .fetch_optional(pool)
        .await?;

    Ok(track)
}

pub async fn list_tracks(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<TrackDb>, sqlx::Error> {
    let tracks =
        sqlx::query_as::<_, TrackDb>("SELECT * FROM track ORDER BY name LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

    Ok(tracks)
}

pub async fn update_track(
    pool: &PgPool,
    id: Uuid,
    request: CreateTrackRequest,
) -> Result<TrackDb, sqlx::Error> {
    let track = sqlx::query_as::<_, TrackDb>(
        r#"
        UPDATE track
        SET track_id = $2, name = $3, description = $4, laps = $5,
            lap_length_km = $6, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.track_id)
    .bind(request.name)
    .bind(request.description)
    .bind(request.laps)
    .bind(request.lap_length_km)
    .fetch_one(pool)
    .await?;

    Ok(track)
}

pub async fn delete_track(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM track WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ========== Player Queries ==========

pub async fn create_player(
    pool: &PgPool,
    request: CreatePlayerRequest,
) -> Result<PlayerDb, sqlx::Error> {
    // Hash password
    let password_hash = hash_password(&request.password)
        .map_err(|e| sqlx::Error::Protocol(format!("Failed to hash password: {}", e).into()))?;
    let player = sqlx::query_as::<_, PlayerDb>(
        r#"
        INSERT INTO player (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(request.username)
    .bind(request.email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(player)
}

pub async fn get_player_by_id(pool: &PgPool, id: Uuid) -> Result<Option<PlayerDb>, sqlx::Error> {
    let player = sqlx::query_as::<_, PlayerDb>("SELECT * FROM player WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(player)
}

pub async fn get_player_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<PlayerDb>, sqlx::Error> {
    let player = sqlx::query_as::<_, PlayerDb>("SELECT * FROM player WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    Ok(player)
}

pub async fn list_players(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<PlayerDb>, sqlx::Error> {
    let players =
        sqlx::query_as::<_, PlayerDb>("SELECT * FROM player ORDER BY username LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

    Ok(players)
}

pub async fn update_player(
    pool: &PgPool,
    id: Uuid,
    request: CreatePlayerRequest,
) -> Result<PlayerDb, sqlx::Error> {
    let player = sqlx::query_as::<_, PlayerDb>(
        r#"
        UPDATE player
        SET username = $2, email = $3, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.username)
    .bind(request.email)
    .fetch_one(pool)
    .await?;

    Ok(player)
}

pub async fn delete_player(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM player WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ========== Race Queries ==========

pub async fn create_race(
    pool: &PgPool,
    request: CreateRaceRequest,
    creator_id: Uuid,
) -> Result<RaceDb, sqlx::Error> {
    let status = request
        .status
        .unwrap_or_else(|| "REGISTRATION_OPEN".to_string());

    let race = sqlx::query_as::<_, RaceDb>(
        r#"
        INSERT INTO race (track_id, laps, status, start_datetime, creator_id, description)
        VALUES ($1, $2, $3::race_status, $4, $5, $6)
        RETURNING id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at
        "#,
    )
    .bind(request.track_id)
    .bind(request.laps)
    .bind(status)
    .bind(request.start_datetime)
    .bind(creator_id)
    .bind(request.description)
    .fetch_one(pool)
    .await?;

    Ok(race)
}

pub async fn get_race_by_id(pool: &PgPool, id: Uuid) -> Result<Option<RaceDb>, sqlx::Error> {
    let race = sqlx::query_as::<_, RaceDb>(
        "SELECT id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at FROM race WHERE id = $1"
    )
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(race)
}

pub async fn list_races(
    pool: &PgPool,
    limit: i64,
    offset: i64,
    status_filter: Option<Vec<&str>>,
) -> Result<Vec<RaceDb>, sqlx::Error> {
    let query = if let Some(statuses) = status_filter {
        // Determine sort order: DESC for finished/canceled races (done), ASC for upcoming races
        let is_done = statuses
            .iter()
            .any(|s| *s == "FINISHED" || *s == "CANCELED");
        let order_direction = if is_done { "DESC" } else { "ASC" };

        // Build a query with status filtering
        let status_placeholders: Vec<String> =
            (1..=statuses.len()).map(|i| format!("${}", i)).collect();
        let status_list = status_placeholders.join(", ");
        let base_query = format!(
            "SELECT id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at FROM race WHERE status::text IN ({}) ORDER BY COALESCE(start_datetime, created_at) {} LIMIT ${} OFFSET ${}",
            status_list,
            order_direction,
            statuses.len() + 1,
            statuses.len() + 2
        );

        let mut query_builder = sqlx::query_as::<_, RaceDb>(&base_query);
        for status in statuses {
            query_builder = query_builder.bind(status);
        }
        query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else {
        // No status filter, return all races (ascending by default)
        sqlx::query_as::<_, RaceDb>(
            "SELECT id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at FROM race ORDER BY COALESCE(start_datetime, created_at) ASC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(query)
}

pub async fn list_races_by_creator(
    pool: &PgPool,
    creator_id: Uuid,
) -> Result<Vec<RaceDb>, sqlx::Error> {
    let races = sqlx::query_as::<_, RaceDb>(
        "SELECT id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at FROM race WHERE creator_id = $1 ORDER BY created_at DESC",
    )
    .bind(creator_id)
    .fetch_all(pool)
    .await?;

    Ok(races)
}

// ========== Registration Queries ==========

pub async fn create_registration(
    pool: &PgPool,
    race_id: Uuid,
    team_id: Uuid,
) -> Result<RegistrationDb, sqlx::Error> {
    let registration = sqlx::query_as::<_, RegistrationDb>(
        r#"
        INSERT INTO registration (race_id, team_id)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(race_id)
    .bind(team_id)
    .fetch_one(pool)
    .await?;

    Ok(registration)
}

pub async fn delete_registration(
    pool: &PgPool,
    race_id: Uuid,
    team_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM registration WHERE race_id = $1 AND team_id = $2")
        .bind(race_id)
        .bind(team_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_registration(
    pool: &PgPool,
    race_id: Uuid,
    team_id: Uuid,
) -> Result<Option<RegistrationDb>, sqlx::Error> {
    let registration = sqlx::query_as::<_, RegistrationDb>(
        "SELECT * FROM registration WHERE race_id = $1 AND team_id = $2",
    )
    .bind(race_id)
    .bind(team_id)
    .fetch_optional(pool)
    .await?;

    Ok(registration)
}

pub async fn list_registrations_by_race(
    pool: &PgPool,
    race_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<RegistrationDb>, sqlx::Error> {
    let registrations = sqlx::query_as::<_, RegistrationDb>(
        "SELECT * FROM registration WHERE race_id = $1 ORDER BY created_at LIMIT $2 OFFSET $3",
    )
    .bind(race_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(registrations)
}

pub async fn list_registrations_by_team(
    pool: &PgPool,
    team_id: Uuid,
) -> Result<Vec<RegistrationDb>, sqlx::Error> {
    let registrations = sqlx::query_as::<_, RegistrationDb>(
        "SELECT * FROM registration WHERE team_id = $1 ORDER BY created_at DESC",
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;

    Ok(registrations)
}

// Response type for registration with race and track details
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegistrationWithRaceDetails {
    pub registration_id: Uuid,
    pub race_id: Uuid,
    pub team_id: Uuid,
    pub track_name: String,
    pub track_id: String,
    pub laps: i32,
    pub race_status: String,
    pub start_datetime: Option<chrono::DateTime<chrono::Utc>>,
    pub description: Option<String>,
    pub registration_created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_registrations_with_race_details_by_team(
    pool: &PgPool,
    team_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<RegistrationWithRaceDetails>, sqlx::Error> {
    let registrations = sqlx::query_as::<_, RegistrationWithRaceDetails>(
        r#"
        SELECT 
            r.id as registration_id,
            r.race_id,
            r.team_id,
            t.name as track_name,
            t.track_id,
            race.laps,
            race.status::text as race_status,
            race.start_datetime,
            race.description,
            r.created_at as registration_created_at
        FROM registration r
        INNER JOIN race ON r.race_id = race.id
        INNER JOIN track t ON race.track_id = t.id
        WHERE r.team_id = $1
        ORDER BY 
            COALESCE(race.start_datetime, race.created_at) ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(team_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(registrations)
}

pub async fn count_registrations_by_race(pool: &PgPool, race_id: Uuid) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM registration WHERE race_id = $1")
        .bind(race_id)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

pub async fn update_race_status(
    pool: &PgPool,
    race_id: Uuid,
    status: &str,
) -> Result<RaceDb, sqlx::Error> {
    let race = sqlx::query_as::<_, RaceDb>(
        r#"
        UPDATE race
        SET status = $2::race_status, updated_at = NOW()
        WHERE id = $1
        RETURNING id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at
        "#,
    )
    .bind(race_id)
    .bind(status)
    .fetch_one(pool)
    .await?;

    Ok(race)
}

/// Start a race by setting its status to ONGOING and recording the start datetime
pub async fn start_race(pool: &PgPool, race_id: Uuid) -> Result<RaceDb, sqlx::Error> {
    use chrono::Utc;
    let now = Utc::now();

    let race = sqlx::query_as::<_, RaceDb>(
        r#"
        UPDATE race
        SET status = 'ONGOING'::race_status, start_datetime = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at
        "#,
    )
    .bind(race_id)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(race)
}

/// Finish a race by setting its status to FINISHED
pub async fn finish_race(pool: &PgPool, race_id: Uuid) -> Result<RaceDb, sqlx::Error> {
    let race = sqlx::query_as::<_, RaceDb>(
        r#"
        UPDATE race
        SET status = 'FINISHED'::race_status, updated_at = NOW()
        WHERE id = $1
        RETURNING id, track_id, laps, status::text as status, start_datetime, creator_id, description, created_at, updated_at
        "#,
    )
    .bind(race_id)
    .fetch_one(pool)
    .await?;

    Ok(race)
}

// ========== Event Queries ==========

pub async fn create_event(
    pool: &PgPool,
    request: CreateEventRequest,
) -> Result<EventDb, sqlx::Error> {
    let event = sqlx::query_as::<_, EventDb>(
        r#"
        INSERT INTO event (
            race_id, event_type, description, time_offset_seconds,
            car_number, car_id, team_id, driver_id, tire, fuel
        )
        VALUES ($1, $2::event_type, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, race_id, event_type::text as event_type, description, time_offset_seconds,
            car_number, car_id, team_id, driver_id, tire, fuel, created_at, updated_at
        "#,
    )
    .bind(request.race_id)
    .bind(request.event_type)
    .bind(request.description)
    .bind(request.time_offset_seconds)
    .bind(request.car_number)
    .bind(request.car_id)
    .bind(request.team_id)
    .bind(request.driver_id)
    .bind(request.tire)
    .bind(request.fuel)
    .fetch_one(pool)
    .await?;

    Ok(event)
}

pub async fn get_event_by_id(pool: &PgPool, id: Uuid) -> Result<Option<EventDb>, sqlx::Error> {
    let event = sqlx::query_as::<_, EventDb>(
        "SELECT id, race_id, event_type::text as event_type, description, time_offset_seconds, car_number, car_id, team_id, driver_id, tire, fuel, created_at, updated_at FROM event WHERE id = $1"
    )
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(event)
}

pub async fn list_events_by_race(
    pool: &PgPool,
    race_id: Uuid,
) -> Result<Vec<EventDb>, sqlx::Error> {
    let events = sqlx::query_as::<_, EventDb>(
        "SELECT id, race_id, event_type::text as event_type, description, time_offset_seconds, car_number, car_id, team_id, driver_id, tire, fuel, created_at, updated_at FROM event WHERE race_id = $1 ORDER BY time_offset_seconds ASC, created_at ASC",
    )
    .bind(race_id)
    .fetch_all(pool)
    .await?;

    Ok(events)
}

pub async fn list_events_by_race_and_type(
    pool: &PgPool,
    race_id: Uuid,
    event_type: &str,
) -> Result<Vec<EventDb>, sqlx::Error> {
    let events = sqlx::query_as::<_, EventDb>(
        "SELECT id, race_id, event_type::text as event_type, description, time_offset_seconds, car_number, car_id, team_id, driver_id, tire, fuel, created_at, updated_at FROM event WHERE race_id = $1 AND event_type = $2::event_type ORDER BY time_offset_seconds ASC, created_at ASC",
    )
    .bind(race_id)
    .bind(event_type)
    .fetch_all(pool)
    .await?;

    Ok(events)
}

pub async fn delete_event(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM event WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_events_by_race(pool: &PgPool, race_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM event WHERE race_id = $1")
        .bind(race_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

// ========== JWT Token Queries ==========

pub async fn create_jwt_token(
    pool: &PgPool,
    player_id: Uuid,
    token: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<JwtTokenDb, sqlx::Error> {
    let jwt_token = sqlx::query_as::<_, JwtTokenDb>(
        r#"
        INSERT INTO jwt_token (player_id, token, expires_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(player_id)
    .bind(token)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(jwt_token)
}

pub async fn get_jwt_token_by_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<JwtTokenDb>, sqlx::Error> {
    let jwt_token = sqlx::query_as::<_, JwtTokenDb>(
        r#"
        SELECT * FROM jwt_token
        WHERE token = $1 AND expires_at > NOW()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(jwt_token)
}

pub async fn delete_expired_jwt_tokens(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM jwt_token
        WHERE expires_at < NOW()
        "#,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_jwt_token_by_token(pool: &PgPool, token: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM jwt_token WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ========== Race Result Queries ==========

pub async fn create_race_result(
    pool: &PgPool,
    request: CreateRaceResultRequest,
) -> Result<RaceResultDb, sqlx::Error> {
    let result = sqlx::query_as::<_, RaceResultDb>(
        r#"
        INSERT INTO race_result (
            race_id, car_id, driver_id, team_id, car_number,
            final_position, race_time_seconds, status, laps_completed,
            total_distance_km
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::race_result_status, $9, $10)
        ON CONFLICT (race_id, car_id) DO UPDATE SET
            final_position = EXCLUDED.final_position,
            race_time_seconds = EXCLUDED.race_time_seconds,
            status = EXCLUDED.status,
            laps_completed = EXCLUDED.laps_completed,
            total_distance_km = EXCLUDED.total_distance_km,
            updated_at = NOW()
        RETURNING id, race_id, car_id, driver_id, team_id, car_number,
            final_position, race_time_seconds, status::text as status,
            laps_completed, total_distance_km, created_at, updated_at
        "#,
    )
    .bind(request.race_id)
    .bind(request.car_id)
    .bind(request.driver_id)
    .bind(request.team_id)
    .bind(request.car_number)
    .bind(request.final_position)
    .bind(request.race_time_seconds)
    .bind(request.status)
    .bind(request.laps_completed)
    .bind(request.total_distance_km)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn get_race_results_by_race(
    pool: &PgPool,
    race_id: Uuid,
) -> Result<Vec<RaceResultDb>, sqlx::Error> {
    let results = sqlx::query_as::<_, RaceResultDb>(
        r#"
        SELECT id, race_id, car_id, driver_id, team_id, car_number,
            final_position, race_time_seconds, status::text as status,
            laps_completed, total_distance_km, created_at, updated_at
        FROM race_result
        WHERE race_id = $1
        ORDER BY final_position ASC
        "#,
    )
    .bind(race_id)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn get_race_result_by_race_and_car(
    pool: &PgPool,
    race_id: Uuid,
    car_id: Uuid,
) -> Result<Option<RaceResultDb>, sqlx::Error> {
    let result = sqlx::query_as::<_, RaceResultDb>(
        r#"
        SELECT id, race_id, car_id, driver_id, team_id, car_number,
            final_position, race_time_seconds, status::text as status,
            laps_completed, total_distance_km, created_at, updated_at
        FROM race_result
        WHERE race_id = $1 AND car_id = $2
        "#,
    )
    .bind(race_id)
    .bind(car_id)
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

pub async fn get_race_results_by_driver(
    pool: &PgPool,
    driver_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<DriverRaceResultDb>, sqlx::Error> {
    let results = sqlx::query_as::<_, DriverRaceResultDb>(
        r#"
        SELECT 
            rr.id as race_result_id,
            rr.race_id,
            t.track_id,
            t.name as track_name,
            r.start_datetime as race_date,
            rr.final_position
        FROM race_result rr
        INNER JOIN race r ON rr.race_id = r.id
        INNER JOIN track t ON r.track_id = t.id
        WHERE rr.driver_id = $1
        ORDER BY COALESCE(r.start_datetime, r.created_at) DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(driver_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn count_race_results_by_driver(
    pool: &PgPool,
    driver_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM race_result WHERE driver_id = $1")
        .bind(driver_id)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

/// Award experience to a driver based on their race position
/// Experience decreases from 50 points for 1st place to 5 points for 10th place
/// Positions beyond 10th also get 5 points
fn calculate_experience_gain(position: i32) -> i32 {
    // Position 1 = 50 points, decreasing by 5 per position
    // Minimum 5 points for positions 10 and beyond
    let exp = 50 - ((position - 1) * 5);
    exp.max(5)
}

/// Award experience to a driver after a race
pub async fn award_driver_experience(
    pool: &PgPool,
    driver_id: Uuid,
    experience_gain: i32,
) -> Result<DriverDb, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>(
        r#"
        UPDATE driver
        SET total_exp = total_exp + $2, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(driver_id)
    .bind(experience_gain)
    .fetch_one(pool)
    .await?;

    Ok(driver)
}

/// Save race results for all cars in a race
/// This function takes a snapshot of all cars and their final state
pub async fn save_race_results(
    pool: &PgPool,
    race_id: Uuid,
    cars: &std::collections::HashMap<u32, crate::models::car::Car>,
    tick_count: u64,
    tick_duration_seconds: f32,
) -> Result<(), sqlx::Error> {
    for car in cars.values() {
        // Calculate race time in seconds
        // For finished/DNF cars, use finished_time; for others, use current tick_count
        let race_time_ticks = if car.finished_time > 0 {
            car.finished_time
        } else {
            tick_count
        };
        let race_time_seconds = race_time_ticks as f32 * tick_duration_seconds;

        // Determine status
        let status = match car.status {
            crate::models::car::CarStatus::Finished => "FINISHED",
            crate::models::car::CarStatus::Dnf => "DNF",
            _ => "DNF", // Any other status (Racing, Pit) is treated as DNF if race ended
        };

        let request = CreateRaceResultRequest {
            race_id,
            car_id: car.uid,
            driver_id: car.driver.uid,
            team_id: car.team.uid,
            car_number: car.number as i32,
            final_position: car.race_position as i32,
            race_time_seconds,
            status: status.to_string(),
            laps_completed: car.lap as i32,
            total_distance_km: car.total_distance,
        };

        if let Err(e) = create_race_result(pool, request).await {
            eprintln!("Failed to save race result for car {}: {}", car.number, e);
            // Continue with other cars even if one fails
        } else {
            // Award experience to the driver based on position
            let exp_gain = calculate_experience_gain(car.race_position as i32);
            if let Err(e) = award_driver_experience(pool, car.driver.uid, exp_gain).await {
                eprintln!(
                    "Failed to award experience to driver {}: {}",
                    car.driver.uid, e
                );
                // Continue even if experience award fails
            }
        }
    }

    Ok(())
}
