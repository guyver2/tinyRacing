use super::models::*;
use sqlx::PgPool;
use uuid::Uuid;

// ========== Team Queries ==========

pub async fn create_team(pool: &PgPool, request: CreateTeamRequest) -> Result<TeamDb, sqlx::Error> {
    let team = sqlx::query_as!(
        // this query check against the database schema for the correct types at compile time query_as!
        TeamDb,
        r#"
        INSERT INTO team (number, name, logo, color, pit_efficiency)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        request.number,
        request.name,
        request.logo,
        request.color,
        request.pit_efficiency
    )
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

pub async fn list_teams(pool: &PgPool) -> Result<Vec<TeamDb>, sqlx::Error> {
    let teams = sqlx::query_as::<_, TeamDb>("SELECT * FROM team ORDER BY number")
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
        UPDATE team
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
    let result = sqlx::query("DELETE FROM team WHERE id = $1")
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
        INSERT INTO driver (first_name, last_name, date_of_birth, nationality, gender, skill_level, stamina, weather_tolerance, experience, consistency, focus, team_id, car_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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

pub async fn list_drivers(pool: &PgPool) -> Result<Vec<DriverDb>, sqlx::Error> {
    let drivers =
        sqlx::query_as::<_, DriverDb>("SELECT * FROM driver ORDER BY last_name, first_name")
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

pub async fn list_cars(pool: &PgPool) -> Result<Vec<CarDb>, sqlx::Error> {
    let cars = sqlx::query_as::<_, CarDb>("SELECT * FROM car ORDER BY number")
        .fetch_all(pool)
        .await?;

    Ok(cars)
}

pub async fn list_cars_by_team(pool: &PgPool, team_id: Uuid) -> Result<Vec<CarDb>, sqlx::Error> {
    let cars = sqlx::query_as::<_, CarDb>("SELECT * FROM car WHERE team_id = $1 ORDER BY number")
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

pub async fn list_tracks(pool: &PgPool) -> Result<Vec<TrackDb>, sqlx::Error> {
    let tracks = sqlx::query_as::<_, TrackDb>("SELECT * FROM track ORDER BY name")
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
    let player = sqlx::query_as::<_, PlayerDb>(
        r#"
        INSERT INTO player (username, email)
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

pub async fn list_players(pool: &PgPool) -> Result<Vec<PlayerDb>, sqlx::Error> {
    let players = sqlx::query_as::<_, PlayerDb>("SELECT * FROM player ORDER BY username")
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
