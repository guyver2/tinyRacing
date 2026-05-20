# Architecture

Text summary of how components connect. No diagram assets.

## Overview

tinyRacing splits **persistent meta-game** (PostgreSQL + Axum REST) from **live race simulation** (single in-memory `RaceState` in a Mutex). The Vue SPA talks REST for management and WebSocket for live telemetry. One Rust binary runs API, WebSocket, simulation tick, and optional ncurses UI concurrently.

## Client (Vue SPA)

- **Router** (`tiny-racing-vue/src/router/index.ts`) loads views lazily.
- **ApiService.ts** — JWT in localStorage; `apiRequest()` adds Bearer header; base URL from `VITE_API_URL` or `hostname:3000`.
- **WebSocketService.ts** — Singleton connects on load; replaces `raceState` ref on each message; reconnects after 5s on close.
- **No global store** — Pinia/Vuex not used; page-local refs plus shared WS composable.

## Server processes (single binary, `server/src/main.rs`)

1. **Axum HTTP** — `0.0.0.0:3000`, router from `api::create_api_router`. CORS allows any origin/method/header.
2. **Warp WebSocket** — `0.0.0.0:3030`, path `/ws`. On connect sends current `RaceStateClientView`; game loop broadcasts full JSON snapshot each tick to all clients.
3. **Game loop** — Tokio task, 100 ms interval. Calls `RaceState::update()` when running; on finish persists results via `finish_race` / `save_race_results`; broadcasts to WS clients and optional ncurses channel.
4. **Watchdog** — If DB available, polls on interval: cancel stale races, mark UPCOMING, auto-start due races, respects single active race.
5. **Ncurses UI** (optional) — Second thread when `DISABLE_UI=false` (default in Docker is `true`).

## Data flows

**Team buys driver:** Vue `POST /drivers/{id}/buy` → Axum `buy_driver` → SQL assign + cash update → JSON team in response.

**Register for race:** Vue `POST /races/{id}/register` → verify status and capacity → insert `registration`.

**Start scheduled race:** Watchdog or `start-now` → `RaceState::load_scheduled_race` (track JSON + registrations + AI fill) → sim paused or running → WS clients receive state.

**Live pit stop:** Vue `POST /race/1/car/{n}/pit` → ownership check → `commands::handle_command` mutates `RaceState` → next tick WS broadcast includes new tire/fuel/status.

**Race completes:** Loop detects `RaceRunState::Finished` → DB race `FINISHED` + `race_result` rows + XP/cash awards.

## Deployment (typical)

Docker Compose: Postgres, backend container, Vue static via nginx. nginx proxies HTTP API (often `/api` → backend `:3000`) and WebSocket (`/ws` → `:3030`). See `DOCKER.md`.

## Configuration

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string |
| `DISABLE_UI` | `true` = headless (default in Docker) |
| `ASSETS_DIR` | Override path to track JSON/assets |
| `VITE_API_URL` | Frontend REST base (Vue build) |
| `VITE_WS_URL` | Frontend WebSocket base (Vue build) |

## Internal broadcast (not public WS)

`AppState` holds `live_tx: broadcast::Sender<LiveEvent>` for race/car/pit events inside the server; clients still receive full state via Warp, not granular events.

*Source: `server/src/main.rs`, `server/src/api.rs`, `DOCKER.md`*
