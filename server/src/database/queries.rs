use super::models::*;
use sqlx::PgPool;
use uuid::Uuid;

// ========== Team Queries ==========

pub async fn create_team(
    pool: &PgPool,
    request: CreateTeamRequest,
) -> Result<TeamDb, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>(
        r#"
        INSERT INTO teams (number, name, logo, color, pit_efficiency)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(request.number)
    .bind(request.name)
    .bind(request.logo)
    .bind(request.color)
    .bind(request.pit_efficiency)
    .fetch_one(pool)
    .await?;

    Ok(team)
}

pub async fn get_team_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TeamDb>, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>("SELECT * FROM teams WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(team)
}

pub async fn get_team_by_number(pool: &PgPool, number: i32) -> Result<Option<TeamDb>, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>("SELECT * FROM teams WHERE number = $1")
        .bind(number)
        .fetch_optional(pool)
        .await?;

    Ok(team)
}

pub async fn list_teams(pool: &PgPool) -> Result<Vec<TeamDb>, sqlx::Error> {
    let teams = sqlx::query_as::<_, TeamDb>("SELECT * FROM teams ORDER BY number")
        .fetch_all(pool)
        .await?;

    Ok(teams)
}

pub async fn update_team(
    pool: &PgPool,
    id: Uuid,
    request: CreateTeamRequest,
) -> Result<TeamDb, sqlx::Error> {
    let team = sqlx::query_as::<_, TeamDb>(
        r#"
        UPDATE teams
        SET number = $2, name = $3, logo = $4, color = $5, pit_efficiency = $6, updated_at = NOW()
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
    .fetch_one(pool)
    .await?;

    Ok(team)
}

pub async fn delete_team(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM teams WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ========== Driver Queries ==========

pub async fn create_driver(
    pool: &PgPool,
    request: CreateDriverRequest,
) -> Result<DriverDb, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>(
        r#"
        INSERT INTO drivers (name, skill_level, stamina, weather_tolerance, experience, consistency, focus)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(request.name)
    .bind(request.skill_level)
    .bind(request.stamina)
    .bind(request.weather_tolerance)
    .bind(request.experience)
    .bind(request.consistency)
    .bind(request.focus)
    .fetch_one(pool)
    .await?;

    Ok(driver)
}

pub async fn get_driver_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DriverDb>, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>("SELECT * FROM drivers WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(driver)
}

pub async fn get_driver_by_name(pool: &PgPool, name: &str) -> Result<Option<DriverDb>, sqlx::Error> {
    let driver = sqlx::query_as::<_, DriverDb>("SELECT * FROM drivers WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?;

    Ok(driver)
}

pub async fn list_drivers(pool: &PgPool) -> Result<Vec<DriverDb>, sqlx::Error> {
    let drivers = sqlx::query_as::<_, DriverDb>("SELECT * FROM drivers ORDER BY name")
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
        UPDATE drivers
        SET name = $2, skill_level = $3, stamina = $4, weather_tolerance = $5,
            experience = $6, consistency = $7, focus = $8, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.name)
    .bind(request.skill_level)
    .bind(request.stamina)
    .bind(request.weather_tolerance)
    .bind(request.experience)
    .bind(request.consistency)
    .bind(request.focus)
    .fetch_one(pool)
    .await?;

    Ok(driver)
}

pub async fn delete_driver(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM drivers WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ========== Car Queries ==========

pub async fn create_car(pool: &PgPool, request: CreateCarRequest) -> Result<CarDb, sqlx::Error> {
    let car = sqlx::query_as::<_, CarDb>(
        r#"
        INSERT INTO cars (number, team_id, driver_id, handling, acceleration, top_speed,
                          reliability, fuel_consumption, tire_wear, base_performance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(request.number)
    .bind(request.team_id)
    .bind(request.driver_id)
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
    let car = sqlx::query_as::<_, CarDb>("SELECT * FROM cars WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(car)
}

pub async fn get_car_by_number(pool: &PgPool, number: i32) -> Result<Option<CarDb>, sqlx::Error> {
    let car = sqlx::query_as::<_, CarDb>("SELECT * FROM cars WHERE number = $1")
        .bind(number)
        .fetch_optional(pool)
        .await?;

    Ok(car)
}

pub async fn list_cars(pool: &PgPool) -> Result<Vec<CarDb>, sqlx::Error> {
    let cars = sqlx::query_as::<_, CarDb>("SELECT * FROM cars ORDER BY number")
        .fetch_all(pool)
        .await?;

    Ok(cars)
}

pub async fn list_cars_by_team(pool: &PgPool, team_id: Uuid) -> Result<Vec<CarDb>, sqlx::Error> {
    let cars = sqlx::query_as::<_, CarDb>("SELECT * FROM cars WHERE team_id = $1 ORDER BY number")
        .bind(team_id)
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
        UPDATE cars
        SET number = $2, team_id = $3, driver_id = $4, handling = $5, acceleration = $6,
            top_speed = $7, reliability = $8, fuel_consumption = $9, tire_wear = $10,
            base_performance = $11, updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(request.number)
    .bind(request.team_id)
    .bind(request.driver_id)
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
    let result = sqlx::query("DELETE FROM cars WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// ========== Track Queries ==========

pub async fn create_track(
    pool: &PgPool,
    request: CreateTrackRequest,
) -> Result<TrackDb, sqlx::Error> {
    let track = sqlx::query_as::<_, TrackDb>(
        r#"
        INSERT INTO tracks (track_id, name, description, laps, lap_length_km, svg_start_offset)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(request.track_id)
    .bind(request.name)
    .bind(request.description)
    .bind(request.laps)
    .bind(request.lap_length_km)
    .bind(request.svg_start_offset)
    .fetch_one(pool)
    .await?;

    Ok(track)
}

pub async fn get_track_by_id(pool: &PgPool, id: Uuid) -> Result<Option<TrackDb>, sqlx::Error> {
    let track = sqlx::query_as::<_, TrackDb>("SELECT * FROM tracks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(track)
}

pub async fn get_track_by_track_id(
    pool: &PgPool,
    track_id: &str,
) -> Result<Option<TrackDb>, sqlx::Error> {
    let track = sqlx::query_as::<_, TrackDb>("SELECT * FROM tracks WHERE track_id = $1")
        .bind(track_id)
        .fetch_optional(pool)
        .await?;

    Ok(track)
}

pub async fn list_tracks(pool: &PgPool) -> Result<Vec<TrackDb>, sqlx::Error> {
    let tracks = sqlx::query_as::<_, TrackDb>("SELECT * FROM tracks ORDER BY name")
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
        UPDATE tracks
        SET track_id = $2, name = $3, description = $4, laps = $5,
            lap_length_km = $6, svg_start_offset = $7, updated_at = NOW()
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
    .bind(request.svg_start_offset)
    .fetch_one(pool)
    .await?;

    Ok(track)
}

pub async fn delete_track(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tracks WHERE id = $1")
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
    let player = sqlx::query_as::<_, PlayerDb>(
        r#"
        INSERT INTO players (username, email)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(request.username)
    .bind(request.email)
    .fetch_one(pool)
    .await?;

    Ok(player)
}

pub async fn get_player_by_id(pool: &PgPool, id: Uuid) -> Result<Option<PlayerDb>, sqlx::Error> {
    let player = sqlx::query_as::<_, PlayerDb>("SELECT * FROM players WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(player)
}

pub async fn get_player_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<PlayerDb>, sqlx::Error> {
    let player = sqlx::query_as::<_, PlayerDb>("SELECT * FROM players WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    Ok(player)
}

pub async fn list_players(pool: &PgPool) -> Result<Vec<PlayerDb>, sqlx::Error> {
    let players = sqlx::query_as::<_, PlayerDb>("SELECT * FROM players ORDER BY username")
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
        UPDATE players
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
    let result = sqlx::query("DELETE FROM players WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

