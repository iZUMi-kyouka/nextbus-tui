# nextbus-tui

<p align="center">
  <img src="assets/icon.png" alt="nextbus-tui icon" width="120" />
</p>

A terminal UI (TUI) desktop client for real-time bus arrival times in Singapore, built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

Supports two modes: **NUS campus shuttles** (all 33 internal stops) and **SG Public Bus** (all ~5,000 LTA stops island-wide). Press `Tab` to switch between them.

[![CI](https://github.com/iZUMi-kyouka/nextbus-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/iZUMi-kyouka/nextbus-tui/actions/workflows/ci.yml)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)

---

## Installation

### Option A — Build from source

Requires [Rust](https://rustup.rs) (stable toolchain).

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && \
  git clone https://github.com/iZUMi-kyouka/nextbus-tui && \
  cd nextbus-tui && cargo build --release && ./target/release/nextbus-tui

# Windows (PowerShell)
winget install Rustlang.Rustup; `
  git clone https://github.com/iZUMi-kyouka/nextbus-tui; `
  cd nextbus-tui; cargo build --release; .\target\release\nextbus-tui.exe
```

### Option B — Download pre-built binary

```bash
# Linux (x86_64)
curl -Lo nextbus-tui https://github.com/iZUMi-kyouka/nextbus-tui/releases/latest/download/nextbus-tui-linux-x86_64 \
  && chmod +x nextbus-tui && ./nextbus-tui

# macOS (Apple Silicon)
curl -Lo nextbus-tui https://github.com/iZUMi-kyouka/nextbus-tui/releases/latest/download/nextbus-tui-macos-aarch64 \
  && chmod +x nextbus-tui && ./nextbus-tui

# macOS (Intel)
curl -Lo nextbus-tui https://github.com/iZUMi-kyouka/nextbus-tui/releases/latest/download/nextbus-tui-macos-x86_64 \
  && chmod +x nextbus-tui && ./nextbus-tui

# Windows — download from the Releases page and run the .exe directly
```

### Option C — Run the install wizard (recommended)

Downloads the latest release for your OS and runs a guided install.

- Linux: choose install path, optional `.desktop`, optional `nnbus` command to start app
- macOS: choose install path, optional `.command`, optional `nnbus` command to start app
- Windows: choose Program Files or user install, optional shortcuts, optional `nnbus` command to start up

```bash
# Linux / macOS (download script only, then run)
curl -fsSL -o install.sh https://raw.githubusercontent.com/iZUMi-kyouka/nextbus-tui/master/install.sh
bash install.sh
```

```powershell
# Windows (PowerShell, download script only, then run)
Invoke-WebRequest -Uri https://raw.githubusercontent.com/iZUMi-kyouka/nextbus-tui/master/install.ps1 -OutFile .\install.ps1
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

> Note: `nextbus-tui` is a terminal UI app. Launchers created by the wizard open it in a terminal-compatible context.

---

## Features

- **NUS Campus mode** — all 33 internal shuttle stops with live arrival times
- **SG Public Bus mode** — all ~5,000 LTA DataMall stops island-wide with real-time ETAs; press `Tab` to switch
- **MRT disruption banner** — amber alert bar (SG mode) when LTA reports a train service disruption; dismiss with `d`
- **Auto-refresh** every 20 seconds (configurable 5–300 s)
- **Favourites** — star stops for quick access; a dedicated favourites view shows only your starred stops
- **Search / filter** — spotlight-style overlay to filter stops by name
- **Number jump** — type a stop number to jump directly to it
- **Mouse support** — scroll, click to select stops, click footer buttons
- **Themes** — 6 built-in colour schemes with a live theme picker
- **Multi-language UI** — English, 简体中文, 繁體中文, 日本語, Bahasa Melayu, தமிழ், Tiếng Việt; auto-detected from system locale on first launch
- **Settings** — configure auto-refresh interval, default view, default mode, and more
- **Experimental wasm web runtime** — browser test mode for app state, tick loop, and live fetch

---

## WASM status (experimental)

- `wasm32-unknown-unknown` compiles and runs in a local browser test harness.
- The shared `ratatui` UI is rendered onto an HTML `<canvas>` in wasm test mode.
- Native desktop runtime remains the default, full-featured target.
- For wasm builds, the default API base is `https://nusbus.flovt.net/ShuttleService` (same query param format: `?busstopname=<NAME>`).
- WASM runs on ratzilla WebGL backend.
- English/Malay use ratzilla's default atlas for stable Latin metrics.
- Japanese/Chinese use custom per-language atlases in `assets/atlas/atlas.<lang>.atlas`.

### Local testing

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

This serves `index.html` and starts the wasm runtime in your browser.

### Custom WebGL atlas workflow

```bash
bash scripts/download-noto-fonts.sh
bash scripts/atlas-workflow.sh
```

What this does:

- Extracts required characters from `assets/i18n/**/*.ftl` and `assets/**/*.toml`
- Writes/updates custom atlases for Japanese/Chinese (`assets/atlas/atlas.<lang>.atlas`)
- Builds real glyph bitmaps from local Noto fonts when available
- Falls back to symbol alias compose mode when Noto fonts are missing
- Verifies CJK atlas coverage against `assets/atlas/required_chars.<lang>.txt`

If verification reports missing characters, replace the affected `assets/atlas/atlas.<lang>.atlas`
with your custom generated atlas and rerun the script.

Note: fallback compose mode ensures deterministic rendering for missing symbols, but proper glyph
shapes require the Noto-based atlas build step.

### Deploying to `nusbus.flovt.net`

The `proxy_api` Cloudflare Worker can now serve both:

- `/` and other non-API paths: static Trunk output (`dist/`)
- `/ShuttleService`: cached proxy to NUSMods

Build and deploy from `proxy_api/`:

```bash
cd proxy_api
npm install
npm run deploy:web
```

This runs a release wasm build (`trunk build --release`) and deploys Worker + assets on the same hostname.

### Native size-bloat guard

To ensure wasm-only crates are never linked into native targets, run:

```bash
bash scripts/check-native-no-wasm.sh
```

The script checks Linux, Windows, and macOS native target dependency graphs.

---

## SG Public Bus

Press `Tab` from the NUS Campus view to enter **SG Public Bus** mode, which covers the full island-wide network via the [LTA DataMall API](https://datamall.lta.gov.sg).

### What you see

- **Stop list** (left) — ~5,000 stops listed by description; search by stop code, road name, or landmark
- **Arrival detail** (right) — next 3 buses per service with ETA, load level (green/amber/red), bus type, and operator
- **MRT disruption banner** — appears at the top when LTA reports a major train disruption; press `d` to dismiss

### Stop data

On first entry into SG mode, stop data is downloaded from the LTA proxy (~5,000 stops, ~10 pages of 500) and cached to `~/.config/nextbus-tui/sg_stops.json` for 7 days. Subsequent launches load from the cache instantly.

### SG-specific keys

| Key | Action |
|-----|--------|
| `Tab` | Switch between NUS Campus and SG Public Bus |
| `d` | Dismiss the MRT disruption banner |

All other keys (navigation, search, favourites, refresh, themes, settings) work identically in both modes.

---

## Key Bindings

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `g` | Go to first stop |
| `G` | Go to last stop |
| `1`–`9`, `01`–`99` | Jump to stop by position number |

### Actions

| Key | Action |
|-----|--------|
| `Tab` | Switch between NUS Campus and SG Public Bus |
| `f` | Toggle favourite on the current stop |
| `F` | Toggle favourites-only view |
| `r` | Force-refresh the current stop |
| `/` | Open search overlay |
| `d` | Dismiss the MRT disruption banner (SG mode) |
| `s` / `S` | Open settings |
| `q` / `Ctrl-C` | Quit |

### Themes

| Key | Action |
|-----|--------|
| `x` | Cycle to the next theme |
| `X` | Open the theme picker popup |

### Search overlay

| Key | Action |
|-----|--------|
| Type | Filter stops in real time |
| `↑` / `↓` | Navigate the filtered list |
| `↵` | Confirm and close the overlay |
| `Esc` | Cancel and clear the filter |

### Theme picker

| Key | Action |
|-----|--------|
| `↑` / `k`, `↓` / `j` | Navigate themes |
| `↵` | Apply the selected theme |
| `Esc` / `X` | Close without applying |

### Settings

| Key | Action |
|-----|--------|
| `↑` / `k`, `↓` / `j` | Navigate settings rows |
| `↵` / `Space` | Edit / toggle the selected setting |
| `0`–`9` | Type a new refresh interval (edit mode) |
| `⌫` | Delete last digit (edit mode) |
| `↵` | Confirm new value (edit mode) |
| `Esc` | Cancel edit / close settings |
| `s` / `S` | Close settings |

---

## Themes

Six colour schemes are bundled (selectable with `x` / `X`):

| Theme | Description |
|-------|-------------|
| **Default** | Explicit black background, standard terminal colours |
| **Catppuccin Mocha** | Pastel dark theme |
| **Dracula** | Classic purple-tinted dark theme |
| **Gruvbox Dark** | Warm retro palette |
| **Nord** | Cool arctic blue palette |
| **Solarized Dark** | Precision colour palette |
| **Tokyo Night** | Deep blue metropolitan theme |

---

## Architecture

```
src/
├── main.rs           — Target-gated entrypoint (native terminal / wasm web)
├── web.rs            — WASM runtime loop/event wiring
├── web_atlas.rs      — WASM WebGL backend creation + custom atlas loading/fallback
├── theme.rs          — Theme/palette types and theme loader
├── sg_api.rs         — LTA DataMall proxy client (Bus Arrival, Bus Stops, Train Alerts)
├── app/
│   ├── mod.rs        — App struct, constructor, message dispatch
│   ├── input.rs      — Native keyboard event dispatch
│   ├── mouse.rs      — Native mouse event dispatch
│   ├── jump.rs       — Number-jump logic
│   ├── fetch.rs      — NUS API fetch and cache management
│   ├── sg_fetch.rs   — SG arrival fetch, stop list loading, train alert polling
│   ├── tick.rs       — Auto-refresh and timer logic (NUS + SG + train alert)
│   ├── favourites.rs — Favourite management and persistence
│   ├── nav.rs        — NUS stop list navigation and filtering
│   ├── sg_nav.rs     — SG stop list navigation and filtering
│   └── overlay.rs    — Settings/theme/search overlay behavior
└── ui/
    ├── mod.rs         — Root render function and layout (mode-aware)
    ├── title.rs       — Title bar with mode badge
    ├── alert_banner.rs — MRT disruption banner (SG mode)
    ├── stop_list.rs   — NUS stop list panel
    ├── sg_stop_list.rs — SG stop list panel
    ├── detail.rs      — NUS bus arrival detail panel
    ├── sg_detail.rs   — SG bus arrival detail panel
    ├── footer.rs      — Footer hints (mode-aware)
    ├── search.rs      — Search overlay
    ├── theme_picker.rs — Theme picker popup
    ├── settings.rs    — Settings overlay rendering
    └── helpers.rs     — Shared style/formatting utilities
```

---

## Disclaimer

This application is an unofficial client and is **not** endorsed by, affiliated with, or supported by the National University of Singapore (NUS).

The software is provided "as is", without warranty of any kind. Users assume full responsibility for any damage or liability arising from the use of this software.
