# User requirements

Maps Vue routes to user-visible behavior. Auth uses JWT in `localStorage` (`jwt_token`, `jwt_token_expires_at`).

## Navigation (AppHeader)

| Link | Visible when | Route |
|------|----------------|-------|
| Game | Always | `/` |
| My Team | Logged in | `/my-team` |
| All Teams | Always | `/teams` |
| Races | Always | `/races` |
| Tracks | Always | `/tracks` |
| Market | Logged in | `/market` |
| Login / Register | Logged out | `/login`, `/register` |
| Logout | Logged in | — |

## Screens and user stories

### Game (`/`, `game.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| G1 | As a visitor, I want to watch the live race so I can follow positions and timing | WebSocket connects; leaderboard and track visualizer show when state is non-empty |
| G2 | As a visitor with no active race, I want guidance to find races | Empty state directs user to Races |
| G3 | As a logged-in player, I want to control only my cars | Driver controls shown when `car.player_uuid` matches JWT `sub` |
| G4 | As a spectator, I want pause/start when allowed | RaceHeader calls `POST /race/1/start` and `POST /race/1/pause` |
| G5 | As a player, I want pit and driving style for my car | DriverControl: `PUT` driving style, `POST` pit (tires, refuel, cancel) |

### My Team (`/my-team`, `Team.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| T1 | As a new player, I want to create my team | Multipart form: name, color, number, logo; requires auth |
| T2 | As a manager, I want to see cash, pit efficiency, logo | Team header displays DB fields |
| T3 | As a manager, I want to see drivers and cars | Grids with links to `/drivers/:id`, `/cars/:id` |
| T4 | As a manager, I want to assign a driver to a car | assign-car API from team page |
| T5 | As a manager, I want upcoming race registrations | List with unregister action |

### All Teams (`/teams`, `AllTeams.vue`; `/teams/:teamId`, `Team.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| AT1 | As a visitor, I want to browse teams | Paginated list |
| AT2 | As a visitor, I want to view another team's roster | Read-only Team view when not owner |

### Races (`/races`, `/races/:raceId`, `Races.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| R1 | As a visitor, I want upcoming and finished races | Separate tables/lists |
| R2 | As a logged-in player, I want to create a race | Form: track, laps, start time, description |
| R3 | As a manager, I want to register/unregister my team | Buttons call register/delete registration APIs |
| R4 | As a manager, I want to start a race immediately | `start-now` for eligible races |
| R5 | As a visitor, I want to see results | RaceResultsModal from results API |

### Market (`/market`, `Market.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| M1 | As a logged-in player with a team, I want to buy drivers and cars | Expandable cards; buy deducts cash; max roster limits enforced by API |

### Tracks (`/tracks`, `Tracks.vue`; `/tracks/:trackId`, `TrackDetail.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| TR1 | As a visitor, I want to browse tracks | List with links to detail |
| TR2 | As a visitor, I want races on a track | TrackDetail filters races; same register/start actions as Races |

### Driver detail (`/drivers/:driverId`, `DriverDetail.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| D1 | As a visitor, I want driver stats and avatar | GET driver |
| D2 | As owner, I want to spend XP on a stat | Level-up UI; 100 XP per +0.1 stat |
| D3 | As a visitor, I want race history | race-results API |

### Car detail (`/cars/:carId`, `CarDetail.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| C1 | As a visitor, I want car stats | GET car |
| C2 | As owner, I want cash upgrades | improve API per stat |

### Login / Register (`LoginForm.vue`, `RegisterForm.vue`)

| ID | Story | Acceptance |
|----|-------|------------|
| A1 | As a new user, I want to register | username, optional email, password |
| A2 | As a user, I want to login and stay signed in | JWT stored until expiry; 401 clears token |

## Personas

- **Guest** — Spectate game, browse teams/races/tracks; no market or my team.
- **Player** — Full team management, market, registration, live controls for own cars.
- **Race creator** — Logged-in player who creates scheduled races (`POST /races`).

## E2E coverage

Playwright specs under `tiny-racing-vue/e2e/`: `auth`, `teams`, `races`, `market`, `tracks`, `drivers-cars`, `vue.spec.ts`. Use as acceptance checklists when changing UX.

*Source: `tiny-racing-vue/src/router/index.ts`, `AppHeader.vue`, component behavior*
