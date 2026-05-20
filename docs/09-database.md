# Database (PostgreSQL)

Access via SQLx in `server/src/database/`. Migrations in `server/migrations/`.

## Tables

### `player`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| username | VARCHAR UNIQUE | |
| email | VARCHAR | optional |
| password_hash | VARCHAR | bcrypt (migration 20251130) |
| created_at, updated_at | TIMESTAMPTZ | auto-trigger |

### `team`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| number | INTEGER UNIQUE | |
| name, logo, color | TEXT/VARCHAR | logo path or URL |
| pit_efficiency | REAL | 0.0–1.0 |
| cash | INTEGER | default 0; new teams get 500 in `create_team` |
| player_id | UUID FK → player | nullable for AI teams |
| created_at, updated_at | TIMESTAMPTZ | |

### `driver`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| first_name, last_name | VARCHAR | |
| date_of_birth | DATE | |
| nationality, gender | VARCHAR | |
| skill_level, stamina, weather_tolerance, experience, consistency, focus | REAL | 0.0–1.0 |
| total_exp, spent_exp | INTEGER | XP pool (migration 20251211) |
| team_id | UUID FK → team | nullable |
| car_id | UUID FK → car | nullable assignment |
| created_at, updated_at | TIMESTAMPTZ | |

### `car`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| number | INTEGER UNIQUE | |
| team_id | UUID FK → team | nullable |
| handling, acceleration, top_speed, reliability, fuel_consumption, tire_wear, base_performance | REAL | bounded in schema |
| created_at, updated_at | TIMESTAMPTZ | |

### `track`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| track_id | VARCHAR UNIQUE | slug e.g. `bahrain` |
| name, description | VARCHAR/TEXT | |
| laps | INTEGER | default lap count |
| lap_length_km | REAL | |
| created_at, updated_at | TIMESTAMPTZ | |

### `race`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| track_id | UUID FK → track | |
| laps | INTEGER | |
| status | race_status enum | see below |
| start_datetime | TIMESTAMPTZ | optional |
| creator_id | UUID FK → player | optional |
| description | TEXT | optional |
| created_at, updated_at | TIMESTAMPTZ | |

### `registration`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| race_id | UUID FK → race | |
| team_id | UUID FK → team | unique per (race_id, team_id) |
| created_at, updated_at | TIMESTAMPTZ | |

### `race_result`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| race_id, car_id, driver_id, team_id | UUID FKs | |
| car_number | INTEGER | |
| final_position | INTEGER | |
| race_time_seconds | REAL | |
| status | race_result_status | FINISHED, DNF |
| laps_completed | INTEGER | |
| total_distance_km | REAL | |
| UNIQUE(race_id, car_id) | | |

### `event`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| race_id | UUID FK | |
| event_type | event_type enum | |
| description | TEXT | |
| time_offset_seconds | REAL | |
| car_number, car_id, team_id, driver_id | optional refs | |
| tire, fuel | optional snapshot | |

### `jwt_token`

| Column | Type | Notes |
|--------|------|-------|
| id | UUID PK | |
| player_id | UUID FK | |
| token | TEXT | |
| expires_at | TIMESTAMPTZ | |

## Enums

### `race_status`

`REGISTRATION_OPEN`, `REGISTRATION_CLOSED`, `FINISHED`, `ONGOING`, `CANCELED`, `UPCOMING` (UPCOMING added migration 20251212).

### `race_result_status`

`FINISHED`, `DNF`

### `event_type`

`START_RACE`, `END_RACE`, `PIT_REQUEST`, `PIT_CANCEL`, `PIT_STOP`, `WEATHER_CHANGE`, `ACCIDENT`, `CAR_FINISHED`, `DNF`, `OTHER`

## Relationships (text)

- One **player** owns zero or one **team** (`team.player_id`).
- One **team** has many **drivers** and many **cars**.
- One **driver** optionally assigned to one **car** (`driver.car_id`).
- One **track** hosts many **races**.
- **Race** links to many **teams** through **registration**.
- **Race** produces many **race_result** rows (one per car).
- **Race** may log many **event** rows during simulation.
- **jwt_token** rows belong to a **player**.

## Migration chronology

| File | Purpose |
|------|---------|
| `20251117000000_initial_schema.sql` | player, team, driver, car, track |
| `20251130000000_add_password_hash_to_player` | auth |
| `20251201000000_create_jwt_tokens` | sessions |
| `20251204000000_add_player_id_to_team` | player ownership |
| `20251206000000_add_cash_to_team` | economy |
| `20251207000000_create_races_table` | race + race_status enum |
| `20251208000000_create_registration_table` | sign-ups |
| `20251209000000_create_event_table` | persisted events |
| `20251210000000_create_race_result_table` | results |
| `20251211000000_add_experience_to_driver` | total_exp, spent_exp |
| `20251212000000_add_upcoming_status_to_race` | UPCOMING status |

## Setup

See `server/DATABASE_SETUP.md` and `server/SQLX_SETUP.md` for local Postgres and compile-time query checking.

*Source: `server/migrations/`, `server/src/database/models.rs`*
