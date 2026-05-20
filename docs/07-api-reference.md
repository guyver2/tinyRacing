# API reference

Base URL: `http://host:3000` (no `/v1` prefix). Docker/nginx may expose this as `/api` on port 8080.

**Response envelope** (typical):

```json
{ "status": "success", "data": { }, "message": "optional" }
```

**Auth**: `Authorization: Bearer <jwt>` where required. Pagination: `?limit=20&offset=0`.

---

## Authentication

| Method | Path | Auth | Body | Handler |
|--------|------|------|------|---------|
| POST | `/auth/login` | Public | `{ username, password }` | `login` |
| POST | `/auth/register` | Public | `{ username, email?, password }` | `register` |
| POST | `/auth/logout` | Bearer | — | `logout` |

Login success `data`: `{ token, expires_at }`.

---

## Teams

| Method | Path | Auth | Notes |
|--------|------|------|-------|
| GET | `/teams` | Public | Query: `limit`, `offset`, optional `player_id` |
| GET | `/teams/my` | Bearer | Current player's team |
| POST | `/teams` | Bearer | `multipart/form-data`: name, color, number?, pit_efficiency?, logo file |
| GET | `/teams/{team_id}` | Public | UUID |
| GET | `/teams/{team_id}/drivers` | Public | Paginated |
| GET | `/teams/{team_id}/cars` | Public | Paginated |
| GET | `/teams/{team_id}/registrations` | Public | Race registrations for team |

---

## Drivers

| Method | Path | Auth | Notes |
|--------|------|------|-------|
| GET | `/drivers` | Public | Paginated list |
| GET | `/drivers/unassigned` | Public | Market pool |
| GET | `/drivers/{driver_id}` | Public | Includes avatar URL in response wrapper |
| GET | `/drivers/{driver_id}/race-results` | Public | Historical results |
| POST | `/drivers/{driver_id}/buy` | Bearer | Returns updated `TeamDb` |
| POST | `/drivers/{driver_id}/assign-car` | Bearer | `{ "car_id": "uuid" \| null }` |
| POST | `/drivers/{driver_id}/level-up` | Bearer | `{ "stat": "skill_level" \| ... }` |

---

## Cars

| Method | Path | Auth | Notes |
|--------|------|------|-------|
| GET | `/cars` | Public | Paginated |
| GET | `/cars/unassigned` | Public | Market pool |
| GET | `/cars/{car_id}` | Public | |
| POST | `/cars/{car_id}/buy` | Bearer | Returns updated `TeamDb` |
| POST | `/cars/{car_id}/improve` | Bearer | `{ "stat": "handling" \| ... }` → `{ car, team }` |

---

## Tracks

| Method | Path | Auth |
|--------|------|------|
| GET | `/tracks` | Public |
| GET | `/tracks/{track_id}` | Public | UUID internal id |

---

## Players

| Method | Path | Auth |
|--------|------|------|
| GET | `/players` | Public |
| GET | `/players/{player_id}` | Public |

---

## Scheduled races (database)

| Method | Path | Auth | Notes |
|--------|------|------|-------|
| GET | `/races` | Public | Filters via query (status, track, pagination) |
| POST | `/races` | Bearer | `CreateRaceRequest`: track_id, laps, start_datetime?, description? |
| GET | `/races/{race_id}` | Public | UUID |
| POST | `/races/{race_id}/register` | Bearer | Team registration |
| DELETE | `/races/{race_id}/register` | Bearer | Unregister own team |
| GET | `/races/{race_id}/registrations` | Public | |
| POST | `/races/{race_id}/start-now` | Public | Load + start sim for this race |
| GET | `/races/{race_id}/results` | Public | `race_result` rows |

---

## Live race (in-memory simulation)

**Note:** `race_id` in path must be **`1`** for pit/style handlers (assert in code). Meta-game race UUIDs are separate.

| Method | Path | Auth | Notes |
|--------|------|------|-------|
| GET | `/race/{race_id}` | Public | `RaceStateClientView` |
| POST | `/race/{race_id}/start` | Public | |
| POST | `/race/{race_id}/pause` | Public | |
| POST | `/race/{race_id}/stop` | Public | |
| GET | `/race/{race_id}/car/{car_number}` | Public | Single car status |
| PUT | `/race/{race_id}/car/{car_number}/driving-style` | Bearer + ownership | `{ "style": "relax" \| "normal" \| "aggressive" }` |
| POST | `/race/{race_id}/car/{car_number}/pit` | Bearer + ownership | `{ "tires"?, "refuel"?, "cancel"?: true }` |

---

## Static assets

| Method | Path | Auth |
|--------|------|------|
| GET | `/assets/*` | Public | Files under `server/assets/` |

---

## Frontend mapping

| ApiService function | Endpoint |
|---------------------|----------|
| `login` | POST `/auth/login` |
| `register` | POST `/auth/register` |
| `logout` | POST `/auth/logout` |
| `getTeams`, `getTeam`, `getMyTeam`, `createTeam` | `/teams`… |
| `getDriver`, `buyDriver`, `levelUpDriver`, … | `/drivers`… |
| `getCar`, `buyCar`, `improveCar` | `/cars`… |
| `getTracks` | GET `/tracks` |
| `getRaces`, `createRace`, `registerForRace`, … | `/races`… |

Ad-hoc in components: `/race/1/...` (see `docs/06-frontend.md`).

---

## Error codes

| HTTP | Meaning |
|------|---------|
| 400 | Validation / business rule failure |
| 401 | Missing or invalid auth |
| 404 | Entity not found |
| 500 | DB unavailable or internal error |

*Source: `server/src/api.rs`, `tiny-racing-vue/src/services/ApiService.ts`*
