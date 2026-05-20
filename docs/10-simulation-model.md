# Simulation model (in-memory)

Live race logic lives in `server/src/models/race.rs` and related model files. It is **not** stored relationally during the race except for optional `event` logging and post-race `race_result` writes.

## RaceState

- **Shared**: `Arc<Mutex<RaceState>>` across API, WebSocket, game loop.
- **Fields**: `track`, `cars` (HashMap by car number), `run_state`, `tick_count`, `tick_duration_seconds`, `events`, optional `race_id` (DB UUID), optional `db_pool`.
- **Empty start**: Server boots with `RaceState::empty()`; races load via scheduled race API or watchdog.

## Run states (`RaceRunState`)

| State | Meaning |
|-------|---------|
| Paused | Loaded but not advancing |
| Running | Tick updates active |
| LastLap | Final lap phase |
| Finished | Race complete; results saved for DB races |

## Tick loop

- Interval: **100 ms** (`main.rs`).
- Each tick: `RaceState::update()` when running — advances track positions, lap counting, tire wear, fuel, weather, driver stress, pit stops, AI decisions.
- After update: `get_client_view()` serialized to WebSocket clients.

## Loading a scheduled race

`load_scheduled_race` (and watchdog preload):

1. Read race + track from DB.
2. Load track geometry/config from `assets/tracks/{track_id}/track.json` (or `ASSETS_DIR`).
3. Build cars from registered teams (and AI teams up to `MAX_PARTICIPANTS` = 5).
4. Set `race_id` on state for finish persistence.
5. Initial `run_state` often Paused until start.

## Car / driver behavior

- **Driving style** (`relax`, `normal`, `aggressive`): Affects stress accumulation and performance multipliers (see `car.rs`, `race.rs` update).
- **Tires**: Compounds (soft, medium, hard, wet, etc.); wear reduces grip; pit changes compound.
- **Fuel**: Consumes per tick; pit can refuel.
- **Speed / position**: Derived from car stats, driver skills, tire state, weather, track wetness.
- **Pit**: Player or AI requests stop; car status `Pit`; service time influenced by team `pit_efficiency`.
- **DNF / reliability**: Failure risk hooks exist (some TODO in code).

## AI pit (`ai_pit_decision`)

For cars with `player_uuid == null`:

- Considers fuel, laps remaining, tire wear, weather wetness.
- Chooses tire compound (e.g. wet if track wetness high).
- Skips if already pitting or on last lap.

## Commands (external control)

`commands.rs` parses strings mutating `RaceState`:

- Race: `start`, `pause`, `stop`
- Car: `order {num} {style}`, `pit {num} [tire] [refuel N]`, `nopit {num}`

REST live handlers build these command strings.

## Weather

- Track has `current_weather` and `wetness` in client view.
- Weather can change during race (`update_weather`); affects tire choice and performance.

## Finish pipeline

When `run_state` becomes `Finished`:

1. If `race_id` set: `finish_race` → DB status `FINISHED`.
2. `save_race_results` → insert `race_result` per car.
3. Award driver XP and team cash from race performance (see `queries.rs` post-race logic).
4. Auto-restart flag exists but is effectively disabled; new races start via API/watchdog.

## Client view (`RaceStateClientView`)

Subset of internal state for JSON WS/API: track summary, car positions, lap info, `race_status` enum.

## Constants

- `MAX_PARTICIPANTS = 5` teams per race.
- Tick duration stored per state (used for time calculations).

*Source: `server/src/models/race.rs`, `car.rs`, `driver.rs`, `commands.rs`, `server/src/main.rs`*
