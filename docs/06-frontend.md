# Frontend (Vue 3)

Location: `tiny-racing-vue/`. Build: Vite. Entry: `src/main.ts` → `App.vue` + router.

## Stack

- Vue 3.5, TypeScript, Vue Router 4
- Chart.js + vue-chartjs (radar charts on team/driver/car pages)
- Vitest (minimal), Playwright E2E

## Routing

| Path | Component | Notes |
|------|-----------|-------|
| `/` | `game.vue` | Live race |
| `/my-team` | `Team.vue` | Own team |
| `/teams` | `AllTeams.vue` | Directory |
| `/teams/:teamId` | `Team.vue` | Props: teamId |
| `/races` | `Races.vue` | List + create |
| `/races/:raceId` | `Races.vue` | Deep link |
| `/drivers/:driverId` | `DriverDetail.vue` | |
| `/cars/:carId` | `CarDetail.vue` | |
| `/market` | `Market.vue` | Auth expected |
| `/tracks` | `Tracks.vue` | |
| `/tracks/:trackId` | `TrackDetail.vue` | |
| `/login` | `LoginForm.vue` | |
| `/register` | `RegisterForm.vue` | |

## State management

- **Auth**: `ApiService.ts` — `getToken`, `setToken`, `isAuthenticated`, `getPlayerId` (JWT decode `sub`).
- **Live race**: `WebSocketService.ts` module singleton — `useRaceData()` returns `raceState`, `connected` refs.
- **Pages**: Local `ref` for lists, forms, loading, errors.

## Services

### `ApiService.ts`

Central REST wrappers: auth, teams, drivers, cars, tracks, races, registrations, market actions. Uses `apiRequest()` with JSON and Bearer header.

Exported helpers include: `login`, `register`, `logout`, `getMyTeam`, `createTeam`, `buyDriver`, `buyCar`, `assignDriverToCar`, `levelUpDriver`, `improveCar`, `getRaces`, `createRace`, `registerForRace`, `startRaceNow`, etc.

### `WebSocketService.ts`

Connects at import; URL from `VITE_WS_URL` or `ws(s)://hostname:3030/ws`.

## Components by feature

### Shell

- `App.vue` — `authenticated` ref; logout; passes auth to children
- `AppHeader.vue` — Navigation

### Live race

- `game.vue` — WS data, mobile panes, filters player cars
- `RaceHeader.vue` — Track info, start/pause (`RACE_ID = 1`)
- `CarsTable.vue` — Leaderboard; fetches driver avatars
- `TrackVisualizer.vue`, `TrackSvg.vue` — Track map; loads `/assets/tracks/...`
- `ConnectionStatus.vue` — WS badge
- `DriverControl.vue` — Style + pit for owned cars
- `DriverStrategyForm.vue` — **Unused** (no imports)

### Team

- `Team.vue` — Create/edit roster, assignments, registrations
- `AllTeams.vue` — Paginated list

### Races / tracks

- `Races.vue`, `TrackDetail.vue` — Scheduling UI
- `RaceResultsModal.vue`, `MobileRaceDetailPopup.vue`

### Market / detail

- `Market.vue` — Unassigned pool
- `DriverDetail.vue`, `CarDetail.vue` — Progression UI
- `DriverStatsRadarChart.vue`, `CarStatsRadarChart.vue`

## Live race API usage (not in ApiService)

Hardcoded `RACE_ID = 1`:

| Component | Calls |
|-----------|-------|
| `RaceHeader.vue` | `POST /race/1/start`, `POST /race/1/pause` |
| `DriverControl.vue` | `PUT /race/1/car/{n}/driving-style`, `POST /race/1/car/{n}/pit`, `GET /drivers/{uid}` |

Scheduled races use UUID paths via `ApiService` (`/races/{uuid}/...`).

## Types

- `src/types/index.ts` — WebSocket `RaceState`, `Car`, `Driver`, `Track`, `Tire`
- `ApiService.ts` — `TeamDb`, `DriverDb`, `CarDb`, `RaceDb`, etc.

## Utils

- `constants.ts` — `DEFAULT_PAGE_SIZE = 20`
- `countryFlags.ts` — Nationality flags
- `colorUtils.ts` — Team colors

## Environment

| Variable | Default behavior |
|----------|------------------|
| `VITE_API_URL` | If unset: `{protocol}//{hostname}:3000` |
| `VITE_WS_URL` | If unset: `ws(s)://{hostname}:3030` |

Relative URLs (e.g. `/api`) supported for nginx proxy.

*Source: `tiny-racing-vue/src/`*
