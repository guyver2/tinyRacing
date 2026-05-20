# Known gaps and technical debt

Items that confuse agents or users if assumed “already fixed.”

| Gap | Impact | Where |
|-----|--------|-------|
| Live REST/WS use in-memory `race_id` **1**; scheduled races use DB UUIDs | Two parallel “race” concepts; frontend hardcodes `RACE_ID = 1` | `api.rs` assert, `RaceHeader.vue`, `DriverControl.vue` |
| [`Readme.md`](../Readme.md) may drift from current run/deploy steps | Check against `DOCKER.md` and `docs/04-architecture.md` | Root |
| JWT secret hardcoded in `auth.rs` | Security risk in production | `server/src/auth.rs` |
| `auth_middleware` not mounted on router | DB token table unused for most routes | `auth_middleware.rs`, `api.rs` |
| Logout deletes DB token; other routes only verify JWT signature | Session invalidation inconsistent | `auth.rs`, handlers |
| `DriverStrategyForm.vue` unused | Dead code | `tiny-racing-vue/src/components/` |
| `live_tx` broadcast events not exposed on WebSocket | Clients only get full state snapshots | `api.rs`, `main.rs` |
| ncurses UI and web UI are separate control surfaces | Feature parity not guaranteed | `ncurses_ui.rs` |
| Performance failure / aggressive risk marked TODO in sim | Incomplete gameplay | `race.rs` comments |
| Registration close at full capacity uses `MAX_PARTICIPANTS` (5) | Comment in code once said “MAX/2” — behavior is full grid | `register_for_race` |

## Doc maintenance

When fixing a gap, update this file and the relevant requirement doc (`03`, `07`, or `06`).

*Last reviewed: 2026-05-20*
