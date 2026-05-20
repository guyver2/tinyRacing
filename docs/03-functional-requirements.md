# Functional requirements

Business rules enforced by the server unless noted. Handlers live in `server/src/api.rs` and `server/src/database/queries.rs`.

## Authentication

- **Register** (`POST /auth/register`): Creates `player` with bcrypt `password_hash`; username unique.
- **Login** (`POST /auth/login`): Validates credentials; returns JWT (24h expiry) and stores row in `jwt_token`.
- **Logout** (`POST /auth/logout`): Requires Bearer token; deletes token from `jwt_token`.
- **Protected routes**: Handlers parse `Authorization: Bearer` and call `auth::validate_token` (JWT signature/expiry). `auth_middleware.rs` exists but is **not** applied globally; logout is the main route that requires DB token lookup.
- **JWT claims**: `sub` = player UUID; used by frontend as `getPlayerId()`.

## Team

- **One team per player**: `team.player_id` unique in practice via `get_team_by_player`.
- **Create team** (`POST /teams`, multipart): Requires auth; fields `name`, `color`, optional `number`, `pit_efficiency`, logo file. Default `pit_efficiency` random 0.4–0.8 if omitted. Default **cash = 500**. Logo saved under assets.
- **Player cannot create a second team** while one exists (handler checks).
- **Read**: `GET /teams`, `GET /teams/{id}`, `GET /teams/my` (auth), nested drivers/cars/registrations.

## Market (buy)

- **Buy driver** (`POST /drivers/{id}/buy`): Auth + player must have team. Driver must be unassigned. Price = `100 × average(skill, stamina, weather_tolerance, experience, consistency, focus)`. Team **max 4 drivers**. Deduct cash; assign `team_id`.
- **Buy car** (`POST /cars/{id}/buy`): Auth + team. Car unassigned. Price = `100 × average(handling, acceleration, top_speed, reliability, fuel_consumption, tire_wear, base_performance)`. Team **max 2 cars**. Deduct cash; assign `team_id`.

## Roster

- **Assign driver to car** (`POST /drivers/{id}/assign-car`): Auth; driver on player's team; body `{ car_id }` or null to unassign; car must belong to same team if set.

## Driver progression

- **Level up** (`POST /drivers/{id}/level-up`): Body `{ stat }` where stat ∈ `skill_level`, `stamina`, `weather_tolerance`, `experience`, `consistency`, `focus`. Costs **100 XP** (`total_exp - spent_exp >= 100`). Increases stat by **0.1** (capped at 1.0). Driver must be on player's team.

## Car progression

- **Improve** (`POST /cars/{id}/improve`): Body `{ stat }` where stat ∈ `handling`, `acceleration`, `top_speed`, `reliability`, `fuel_consumption`, `tire_wear`. Car on player's team. Cost scales with current stat: **$2** if stat &lt; 0.1, **$100** if ≥ 0.9, else linear between. +0.1 to stat (capped). Deducts team cash in DB.

## Scheduled races

- **Status enum**: `REGISTRATION_OPEN`, `REGISTRATION_CLOSED`, `UPCOMING`, `ONGOING`, `FINISHED`, `CANCELED`.
- **Create** (`POST /races`): Auth; `track_id`, `laps`, optional `start_datetime`, `description`; default status `REGISTRATION_OPEN`.
- **Register** (`POST /races/{id}/register`): Auth + team; race must be `REGISTRATION_OPEN`; start time not in past; not already registered; **max 5 teams** (`MAX_PARTICIPANTS`). At 5 registrations, status → `REGISTRATION_CLOSED`.
- **Unregister** (`DELETE /races/{id}/register`): Auth; removes registration; may reopen `REGISTRATION_CLOSED` if count drops below max.
- **Start now** (`POST /races/{id}/start-now`): Loads race from DB into `RaceState`, starts simulation (public handler).
- **Watchdog** (background): Cancels overdue races; marks `UPCOMING` ~5 min before start (loads sim paused); auto-starts at `start_datetime` if no other race running; only one ongoing race at a time.
- **AI fill**: When loading scheduled race, if fewer than 5 human teams registered, AI teams (`player_id IS NULL`) fill slots.

## Live race (in-memory)

- **Single `RaceState`** shared by REST, WebSocket, game loop.
- **Live REST paths** use `race_id` in URL but handlers **`assert_eq!(race_id, 1)`** for pit/style.
- **Get status** (`GET /race/{race_id}`): Returns client view; effectively race `1`.
- **Start / pause / stop**: `POST /race/{id}/start|pause|stop` via `commands.rs`.
- **Driving style** (`PUT /race/1/car/{num}/driving-style`): Body `{ style }` — `relax`, `normal`, `aggressive`. Auth + car owned by player and registered in current race.
- **Pit** (`POST /race/1/car/{num}/pit`): Body optional `tires`, `refuel`, `cancel`. At least one of tires/refuel unless cancel. Maps to `pit` / `nopit` commands.
- **Tick**: 100 ms; updates positions, wear, fuel, weather, stress, pit logic.
- **Finish**: When sim finishes, scheduled race `status` → `FINISHED`; `race_result` rows saved; drivers earn XP; teams earn **cash = sum of driver XP** from that race.

## Pagination

- List endpoints accept `limit` and `offset` query params; default page size **20** (`DEFAULT_PAGE_SIZE`).

## API response envelope

```json
{ "status": "success|error", "data": ..., "message": "..." }
```

Errors use HTTP status codes via `ApiError` (400, 401, 404, 500).

*Source: `server/src/api.rs`, `server/src/database/queries.rs`, `server/src/watchdog.rs`, `server/src/models/race.rs`*
