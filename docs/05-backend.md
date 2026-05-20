# Backend (Rust)

Crate: `tiny_racing` in `server/`. Entry: `server/src/main.rs`.

## Runtime responsibilities

| Component | File(s) | Role |
|-----------|---------|------|
| HTTP API | `api.rs` | Axum routes, JSON envelope, auth extraction per handler |
| WebSocket | `main.rs` (Warp) | Fan-out `RaceStateClientView` JSON |
| Game loop | `main.rs` | 100 ms tick, finish persistence |
| Watchdog | `watchdog.rs` | Scheduled race lifecycle |
| Simulation | `models/race.rs`, `car.rs`, `driver.rs`, etc. | Physics and race logic |
| Commands | `commands.rs` | String commands for pit/start/pause/order |
| Auth | `auth.rs` | JWT + bcrypt; `jwt_token` table |
| Database | `database/` | SQLx pool, migrations, queries |
| NCurses UI | `ncurses_ui.rs` | Terminal race UI |
| Avatars | `models/driver_avatar.rs` | Generate/serve driver images |

## Module map (`server/src/`)

### `api.rs`

- Builds router (~40 routes).
- `AppState`: `SharedRaceState`, `live_tx`, optional `PgPool`.
- Helpers: pagination, `driver_to_response` (avatar URL), `verify_car_ownership_and_registration`.
- Bridges live controls to `commands::handle_command`.

### `models/`

| File | Contents |
|------|----------|
| `race.rs` | `RaceState`, `RaceRunState`, `update()`, `load_scheduled_race`, `get_client_view`, AI pit, events |
| `car.rs` | Car stats, tire, fuel, status, speed, pit |
| `driver.rs` | Driver stats, `DrivingStyle`, stress |
| `team.rs` | Team in sim |
| `track.rs` | Track config from JSON |
| `tire.rs` | Compounds, wear |
| `weather.rs` | Weather state |
| `event.rs` | Runtime race events |

### `database/`

| File | Contents |
|------|----------|
| `connection.rs` | Pool, `init_from_env`, migrate |
| `migrations.rs` | Custom migration runner (`.sql`, `.up.sql`/`.down.sql`) |
| `models.rs` | `*Db` structs, request DTOs |
| `queries.rs` | All SQL: CRUD, races, registrations, results, improve, level-up |
| `mod.rs` | `finish_race`, `save_race_results` exports |

### `commands.rs`

Text commands, e.g. `start`, `pause`, `pit {num} {tire} refuel {amount}`, `order {num} {style}`, `nopit {num}`. Used by ncurses and API live handlers.

### `auth.rs` / `auth_middleware.rs`

- Secret: compile-time `JWT_SECRET` (should be env in production).
- Expiry: 24 hours.
- Middleware validates JWT + DB token but router does not mount it today.

### `watchdog.rs`

`RaceWatchdog::check_races`: cancel past-due, mark UPCOMING (preload paused), start due races if sim idle.

### `constants.rs`

`DEFAULT_PAGE_SIZE = 20`.

## Examples (`server/examples/`)

| Script | Purpose |
|--------|---------|
| `seed_db.rs` | Populate teams, drivers, cars, tracks |
| `run_migrations.rs` | Apply migrations |
| `database_example.rs` | DB usage sample |
| `generate_driver_avatar.rs` | Avatar asset generation |

## Assets

- Served at `GET /assets/*` from `server/assets/` (or `ASSETS_DIR`).
- Tracks: `assets/tracks/{track_id}/track.json`, `track.svg`.

## Ports

| Port | Protocol |
|------|----------|
| 3000 | HTTP REST |
| 3030 | WebSocket `/ws` |

*Source: `server/src/`, `server/Cargo.toml`*
