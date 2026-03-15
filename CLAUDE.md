# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run
cargo run

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Check without building
cargo check

# Lint
cargo clippy
```

## Architecture

This is a terminal UI (TUI) app for viewing NUS campus shuttle bus arrival times, built with **ratatui** + **crossterm** for rendering and **ureq** + **serde_json** for HTTP/JSON.

### Data flow

```
main.rs  ──► run_loop()
              │
              ├─ keyboard events ──► handle_key() ──► App methods
              │
              └─ background events (mpsc::Receiver<AppEvent>)
                    ├─ AppEvent::Tick         ──► app.handle_tick()
                    ├─ AppEvent::DataReceived ──► app.handle_data()
                    └─ AppEvent::FetchError   ──► app.handle_error()
```

Background threads (one per in-flight fetch) call `api::fetch_shuttle_service()` and send results back via an `mpsc::Sender<AppEvent>`. A separate tick thread fires every 500 ms to drive auto-refresh and status message expiry.

### Module responsibilities

| File | Role |
|------|------|
| `main.rs` | Terminal setup/teardown, event loop, keyboard dispatch |
| `app.rs` | All mutable application state (`App`), list management, fetch orchestration, event handlers |
| `ui.rs` | Pure rendering: title bar, left stop list, right detail panel, footer |
| `api.rs` | Single function that GETs `https://nnextbus.nusmods.com/ShuttleService?busstopname=<NAME>` |
| `models.rs` | All serde structs (`BusStop`, `ApiResponse`, `ShuttleServiceResult`, `Shuttle`, `Config`) plus `AppEvent` enum |
| `config.rs` | Load/save favourites to `~/.config/nextbus-tui/config.json` |

### Key design points

- **Stop list** is stored as `Vec<BusStop>` (from `assets/stops.json`, embedded via `include_str!`). The rendered order is `sorted_indices: Vec<usize>` — indices into that vec, sorted favourites-first then alphabetically, filtered by `search_query`.
- **Cache** is `HashMap<String, CachedData>` keyed by stop `name` (e.g. `"COM3"`). Stale entries (>30 s) trigger an auto-refresh on the next tick.
- **Loading state** is a `HashSet<String>` of stop names currently being fetched; prevents duplicate in-flight requests.
- **Favourites** are persisted immediately on toggle via `config::save()`.
- The UI layout is: 1-row title bar / main 33%–67% horizontal split / 1-row footer.

### Assets

- `assets/stops.json` — static list of all NUS bus stops (embedded at compile time).
- `assets/routes.json` — route data (present but not yet used by the app).
