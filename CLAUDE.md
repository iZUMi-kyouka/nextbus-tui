# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Setup

After cloning, install the project git hooks (pre-commit: fmt check; pre-push: fmt + tests):

```bash
bash scripts/install-hooks.sh
```

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
| `main.rs` | Terminal setup/teardown, event loop, mouse/keyboard dispatch |
| `theme.rs` | `Palette`/`Theme` types; `load_themes()` embeds TOML files from `assets/themes/` |
| `app/mod.rs` | `App` struct, constructor, `theme()` accessor |
| `app/input.rs` | Keyboard event dispatch; modal priority: theme picker → search → normal |
| `app/mouse.rs` | Mouse event dispatch (scroll, click on list, footer buttons) |
| `app/list.rs` | `rebuild_list()`: sorted/filtered `sorted_indices`; fav-view branch |
| `app/jump.rs` | Number-jump buffer; immediate commit when list < 10 items |
| `app/fetch.rs` | API fetch and cache management |
| `app/tick.rs` | Auto-refresh (interval from `app.auto_refresh_secs`) and status message expiry |
| `app/favourites.rs` | Favourite toggle and persistence |
| `app/settings.rs` | Settings overlay key handling (interval edit, default-view toggle, language stub) |
| `ui/mod.rs` | Root render function; background fill; layout split |
| `ui/title.rs` | Title bar (theme-aware) |
| `ui/stop_list.rs` | Stop list panel with fav highlight and ellipsis truncation |
| `ui/detail.rs` | Bus arrival detail panel with shuttle table |
| `ui/footer.rs` | Footer hints (context-sensitive: jump / status / search / normal) |
| `ui/search.rs` | Search overlay |
| `ui/theme_picker.rs` | Theme picker popup with colour swatches |
| `ui/settings.rs` | Settings overlay (auto-refresh interval, default view, language stub) |
| `ui/helpers.rs` | Shared style/formatting utilities (`arrival_style`, `route_color`, `fmt_arrival`) |
| `api.rs` | Single function: GETs `https://nnextbus.nusmods.com/ShuttleService?busstopname=<NAME>` |
| `models.rs` | Serde structs (`BusStop`, `ApiResponse`, `ShuttleServiceResult`, `Shuttle`, `Config`) + `AppEvent` |
| `config.rs` | Load/save favourites to `~/.config/nextbus-tui/config.json` |

### Key design points

- **Stop list** is stored as `Vec<BusStop>` (from `assets/stops.toml`, embedded via `include_str!`). The rendered order is `sorted_indices: Vec<usize>` — indices into that vec, sorted favourites-first then alphabetically, filtered by `search_query`.
- **Cache** is `HashMap<String, CachedData>` keyed by stop `name` (e.g. `"COM3"`). Stale entries (> `app.auto_refresh_secs`) trigger an auto-refresh on the next tick.
- **Config** (`~/.config/nextbus-tui/config.json`) persists favourites, `refresh_interval_secs`, and `default_fav_view`. Call `app.config_snapshot()` to build a `Config` from live state before calling `config::save()`.
- **Loading state** is a `HashSet<String>` of stop names currently being fetched; prevents duplicate in-flight requests.
- **Favourites** are persisted immediately on toggle via `config::save()`.
- The UI layout is: 1-row title bar / main 33%–67% horizontal split / 1-row footer.

### Assets

- `assets/stops.json` — static list of all NUS bus stops (embedded at compile time).
- `assets/routes.json` — route colour data (embedded; used by `ui/helpers.rs` for bus name badge colours).
- `assets/themes/*.toml` — one TOML file per colour scheme, embedded via `include_str!` in `theme.rs`.

## Keeping README.md up to date

**README.md is user-facing documentation. Keep it in sync whenever you:**

- Add, remove, or rename a key binding
- Add or remove a feature (Features section)
- Add, remove, or rename a theme
- Change the `src/` module layout (Architecture section)

The architecture tree in README.md mirrors the actual `src/` directory structure. Update it whenever a source file is added, removed, or its role changes significantly.
