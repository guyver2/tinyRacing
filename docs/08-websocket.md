# WebSocket (live race)

## Endpoint

- **URL**: `ws://host:3030/ws` (or `wss://` behind TLS)
- **Vue default**: `VITE_WS_URL` if set; else `ws(s)://{hostname}:3030/ws`
- **nginx**: Often proxied as `/ws` on the same host as the SPA (see `DOCKER.md`)

## Server behavior (`server/src/main.rs`)

- Warp WebSocket server on port **3030**, separate from Axum port **3000**.
- On connect: immediately sends one JSON message — current `RaceStateClientView`.
- While race loop runs: broadcasts full serialized state to all connected clients every **~100 ms** (game loop tick).
- Incoming client messages are not processed for commands (read loop is effectively no-op). **Pit and driving style use REST**, not WS.

## Client behavior (`WebSocketService.ts`)

- Connects on module import.
- `message` handler: `JSON.parse(event.data)` → assigns entire object to `raceState` ref.
- On `close`: `connected = false`, reconnect after **5 seconds**.
- `useRaceData()` exposes `raceState` and `connected` to components.

## Payload shape (`RaceStateClientView`)

Serialized from `server/src/models/race.rs`; consumed as `RaceState` in `tiny-racing-vue/src/types/index.ts`.

| Field | Type | Description |
|-------|------|-------------|
| `track` | object | `id`, `name`, `svg_start_offset`, `current_weather`, `wetness`, `elapsed_time` |
| `cars` | array | See below |
| `current_lap` | number | Leader-oriented lap display |
| `total_laps` | number | Race length |
| `race_status` | string | `Paused`, `Running`, `LastLap`, `Finished` (serde enum) |

### Car object (client view)

| Field | Description |
|-------|-------------|
| `car_number` | Race number |
| `race_position` | Position |
| `track_position` | 0–1 along lap |
| `lap` | Current lap |
| `status` | e.g. racing, pit |
| `driving_style` | relax / normal / aggressive |
| `speed` | Current speed |
| `fuel` | Fuel level |
| `tire` | `{ type, wear }` |
| `driver` | Name, skills, stress, uid |
| `team` | number, name, logo, color |
| `player_uuid` | Owner player id if human; null for AI |
| `pit_requested` | Pending pit flag |

## Relation to REST

- **Read** live state: WebSocket (push).
- **Write** controls: `PUT/POST /race/1/car/...` (see `docs/07-api-reference.md`).
- Scheduled race metadata: REST `/races/{uuid}` only.

*Source: `server/src/main.rs`, `server/src/models/race.rs`, `tiny-racing-vue/src/services/WebSocketService.ts`, `tiny-racing-vue/src/types/index.ts`*
