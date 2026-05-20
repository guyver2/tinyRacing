# Product overview

## What it is

tinyRacing is a **small-team racing manager**: you recruit drivers and cars, spend team cash on upgrades, register for scheduled races, and during a live race you set **driving style** and **pit strategy** (tires, fuel). You do not steer the car; the simulation runs automatically.

## Player goals

1. Create an account and a team.
2. Build a competitive roster (up to 4 drivers, 2 cars) via the market.
3. Assign drivers to cars and improve stats (driver XP, car cash upgrades).
4. Register for upcoming races on real tracks.
5. Watch the live race and issue pit/style commands for your entries.
6. Review results and history.

## Major feature areas

| Area | Description |
|------|-------------|
| Authentication | Register, login, JWT session |
| Team management | One team per player; logo, colors, pit efficiency, cash |
| Market | Buy unassigned drivers/cars from a global pool |
| Roster | Assign driver ↔ car; view team registrations |
| Progression | Driver level-up (XP); car stat improvements (cash) |
| Race scheduling | Create/list races; register teams; watchdog auto-start |
| Live race | WebSocket state stream; REST pause/start/pit/style |
| Tracks | Browse circuits; races filtered by track |
| Results | Persisted per-car results after scheduled races finish |

## Core loops

**Meta-game loop:** Login → create team → market purchases → assign roster → level up / improve car → register for race → (optional) start race early via API.

**Live loop:** Open Game (`/`) → WebSocket shows positions → for your cars: change driving style, request pit (tire compound, refuel, cancel) → race ends → results on Races screen.

**Scheduled race loop:** Creator or system defines race (track, laps, start time) → teams register while `REGISTRATION_OPEN` → watchdog marks `UPCOMING`, loads sim paused → at start time loads and runs → status `FINISHED`, `race_result` rows written.

## Out of scope (web client)

- **Ncurses UI** — Terminal UI when `DISABLE_UI=false` (`server/src/ncurses_ui.rs`).
- **track_preprocessor/** — Python tool to prepare track geometry/assets.
- **Text commands** — `server/src/commands.rs` used by ncurses and live REST; not exposed as a public chat API.

## Repository layout

| Path | Role |
|------|------|
| `server/` | Rust binary: API, WebSocket, game loop, DB |
| `tiny-racing-vue/` | Vue 3 SPA |
| `server/assets/` | Tracks, tires, weather images |
| `server/migrations/` | SQL schema |

*Source: project structure, `server/src/main.rs`, `tiny-racing-vue/src/router/index.ts`*
