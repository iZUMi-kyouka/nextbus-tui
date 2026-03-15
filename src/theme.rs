use include_dir::{Dir, include_dir};
use ratatui::style::Color;

static THEMES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/themes");

// ── Public types ──────────────────────────────────────────────────────────────

/// Whether a theme is designed for dark or light terminals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    Auto,
}

/// Resolved ratatui `Color` values for every semantic UI role.
pub struct Palette {
    pub background: Color,
    pub foreground: Color,
    /// Stop-list panel border.
    pub border: Color,
    /// Detail panel border.
    pub detail_border: Color,
    /// Favourites, search overlay, ≤3-min arrivals.
    pub highlight: Color,
    /// "Arriving" label, status messages.
    pub success: Color,
    /// Error messages.
    pub error: Color,
    /// Dim / secondary text, separators.
    pub dim: Color,
    /// Jump prompt accent.
    pub jump: Color,
    /// Five swatch colours shown in the picker: [red, green, yellow, blue, magenta].
    pub swatch: [Color; 5],
}

pub struct Theme {
    pub name: String,
    /// Whether this theme is designed for dark or light backgrounds.
    pub mode: ThemeMode,
    pub palette: Palette,
}

// ── Serde types (private, only used during parsing) ───────────────────────────

#[derive(serde::Deserialize)]
struct ThemeFile {
    name: String,
    /// "dark" or "light"; defaults to "dark" if absent.
    #[serde(default)]
    mode: String,
    colors: ThemeFileColors,
}

#[derive(serde::Deserialize)]
struct ThemeFileColors {
    primary: ThemePrimary,
    normal: ThemeColorSet,
    bright: ThemeColorSet,
}

#[derive(serde::Deserialize)]
struct ThemePrimary {
    background: String,
    foreground: String,
}

#[derive(serde::Deserialize)]
struct ThemeColorSet {
    black: String,
    red: String,
    green: String,
    yellow: String,
    blue: String,
    magenta: String,
    cyan: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn hex(s: &str) -> Color {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return Color::Reset;
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

fn from_file(src: &str) -> Theme {
    let f: ThemeFile = toml::from_str(src).expect("invalid theme file");
    let mode = match f.mode.to_lowercase().as_str() {
        "light" => ThemeMode::Light,
        _ => ThemeMode::Dark,
    };
    let n = &f.colors.normal;
    let p = &f.colors.primary;
    Theme {
        name: f.name,
        mode,
        palette: Palette {
            background: hex(&p.background),
            foreground: hex(&p.foreground),
            border: hex(&n.blue),
            detail_border: hex(&n.cyan),
            highlight: hex(&n.yellow),
            success: hex(&n.green),
            error: hex(&n.red),
            dim: hex(&f.colors.bright.black),
            jump: hex(&n.cyan),
            swatch: [
                hex(&n.red),
                hex(&n.green),
                hex(&n.yellow),
                hex(&n.blue),
                hex(&n.magenta),
            ],
        },
    }
}

fn default_theme() -> Theme {
    Theme {
        name: "Default".to_string(),
        mode: ThemeMode::Dark,
        palette: Palette {
            background: Color::Black,
            foreground: Color::White,
            border: Color::Blue,
            detail_border: Color::Cyan,
            highlight: Color::Yellow,
            success: Color::Green,
            error: Color::Red,
            dim: Color::DarkGray,
            jump: Color::Cyan,
            swatch: [
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue,
                Color::Magenta,
            ],
        },
    }
}

// ── Public loader ─────────────────────────────────────────────────────────────

/// Returns all available themes.  Index 0 is always the built-in default
/// (pitch-black background, standard terminal colours).
/// All `.toml` files in `assets/themes/` are embedded at compile time and
/// loaded in alphabetical order — no manual list to maintain.
pub fn load_themes() -> Vec<Theme> {
    let mut themes = vec![default_theme()];
    let mut files: Vec<_> = THEMES_DIR
        .files()
        .filter(|f| f.path().extension().and_then(|e| e.to_str()) == Some("toml"))
        .collect();
    files.sort_by_key(|f| f.path());
    for file in files {
        if let Some(src) = file.contents_utf8() {
            themes.push(from_file(src));
        }
    }
    themes
}
