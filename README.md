# nextbus-tui

A terminal UI (TUI) desktop client for NUS internal shuttle bus (ISB) arrival times, built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

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

- **33 NUS bus stops** with live arrival times from the NUS shuttle service API
- **Auto-refresh** every 20 seconds (configurable 5–300 s)
- **Favourites** — star stops for quick access; a dedicated favourites view shows only your starred stops
- **Search / filter** — spotlight-style overlay to filter stops by name
- **Number jump** — type a stop number to jump directly to it
- **Mouse support** — scroll, click to select stops, click footer buttons
- **Themes** — 6 built-in colour schemes with a live theme picker
- **Settings** — configure auto-refresh interval, default view, and more
- **Experimental wasm web runtime** — browser test mode for app state, tick loop, and live fetch

---

## WASM status (experimental)

- `wasm32-unknown-unknown` compiles and runs in a local browser test harness.
- The shared `ratatui` UI is rendered onto an HTML `<canvas>` in wasm test mode.
- Native desktop runtime remains the default, full-featured target.
- For wasm builds, the default API base is `https://nusbus.flovt.net/ShuttleService` (same query param format: `?busstopname=<NAME>`).

### Local testing

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

This serves `index.html` and starts the wasm runtime in your browser.

### Native size-bloat guard

To ensure wasm-only crates are never linked into native targets, run:

```bash
bash scripts/check-native-no-wasm.sh
```

The script checks Linux, Windows, and macOS native target dependency graphs.

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
| `f` | Toggle favourite on the current stop |
| `F` | Toggle favourites-only view |
| `r` | Force-refresh the current stop |
| `/` | Open search overlay |
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
├── main.rs          — Target-gated entrypoint (native terminal / wasm web)
├── web.rs           — WASM runtime using ratzilla canvas backend
├── theme.rs         — Theme/palette types and theme loader
├── app/
│   ├── mod.rs       — App struct, constructor, config snapshot
│   ├── input.rs     — Native keyboard event dispatch
│   ├── mouse.rs     — Native mouse event dispatch
│   ├── jump.rs      — Number-jump logic
│   ├── fetch.rs     — API fetch and cache management
│   ├── tick.rs      — Auto-refresh and timer logic
│   ├── favourites.rs — Favourite management and persistence
│   ├── nav.rs       — Stop list navigation and filtering
│   └── overlay.rs   — Settings/theme/search overlay behavior
└── ui/
    ├── mod.rs        — Root render function and layout
    ├── title.rs      — Title bar
    ├── stop_list.rs  — Stop list panel
    ├── detail.rs     — Bus arrival detail panel
    ├── footer.rs     — Footer hints
    ├── search.rs     — Search overlay
    ├── theme_picker.rs — Theme picker popup
    ├── settings.rs   — Settings overlay rendering
    └── helpers.rs    — Shared style/formatting utilities
```

---

## Disclaimer

This application is an unofficial client and is **not** endorsed by, affiliated with, or supported by the National University of Singapore (NUS).

The software is provided "as is", without warranty of any kind. Users assume full responsibility for any damage or liability arising from the use of this software.
