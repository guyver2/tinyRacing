# tinyRacing — Agent guide

tinyRacing is a **team-management racing simulation**: players run a small F1-style team (drivers, cars, cash), register for scheduled races, and influence live races through pit strategy and driving style—not direct driving.

## Stack

| Layer | Technology | Location |
|-------|------------|----------|
| Frontend | Vue 3, TypeScript, Vite | `tiny-racing-vue/` |
| Backend | Rust (Axum REST, Warp WebSocket) | `server/` |
| Database | PostgreSQL (SQLx) | `server/migrations/` |
| Assets | Track JSON/SVG, logos | `server/assets/` |

## Documentation map

Read these before changing behavior. Update the matching doc when you change code.

| Document | Read when… |
|----------|------------|
| [docs/README.md](docs/README.md) | Index and last-verified date |
| [docs/01-product-overview.md](docs/01-product-overview.md) | Understanding product scope and player loops |
| [docs/02-user-requirements.md](docs/02-user-requirements.md) | UI routes, screens, user stories |
| [docs/03-functional-requirements.md](docs/03-functional-requirements.md) | Business rules (cash, XP, registration, pit) |
| [docs/04-architecture.md](docs/04-architecture.md) | Processes, ports, data flow |
| [docs/05-backend.md](docs/05-backend.md) | Rust modules and server responsibilities |
| [docs/06-frontend.md](docs/06-frontend.md) | Vue components, routing, API usage |
| [docs/07-api-reference.md](docs/07-api-reference.md) | REST endpoints (authoritative paths) |
| [docs/08-websocket.md](docs/08-websocket.md) | Live race WebSocket protocol |
| [docs/09-database.md](docs/09-database.md) | Tables, enums, relationships |
| [docs/10-simulation-model.md](docs/10-simulation-model.md) | In-memory race tick, physics, AI |
| [docs/11-known-gaps.md](docs/11-known-gaps.md) | Technical debt and doc/code mismatches |

## Operational docs (existing)

- [DOCKER.md](DOCKER.md) — Compose, nginx proxy, ports
- [server/DATABASE_SETUP.md](server/DATABASE_SETUP.md) — Postgres setup
- [tiny-racing-vue/e2e/README.md](tiny-racing-vue/e2e/README.md) — Playwright E2E

## Update rules

1. Change code first.
2. Update the relevant `docs/*.md` section (especially `03`, `07`, `09` for behavior/schema changes).
3. Add or remove items in `docs/11-known-gaps.md` when fixing or introducing debt.
4. Set **Last verified** date in [docs/README.md](docs/README.md).

Documentation is **text-only** (prose and markdown tables). Do not add diagrams or mermaid to these files.

## Source of truth

- HTTP routes: `server/src/api.rs`
- SQL / persistence: `server/migrations/`, `server/src/database/queries.rs`
- Live sim: `server/src/models/race.rs`
- Frontend API: `tiny-racing-vue/src/services/ApiService.ts`

Retired docs are listed in [docs/archive/README.md](docs/archive/README.md). Root [api_documentation.md](api_documentation.md) is a stub → [docs/07-api-reference.md](docs/07-api-reference.md).
