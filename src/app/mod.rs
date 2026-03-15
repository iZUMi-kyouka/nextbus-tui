pub(crate) mod favourites;
pub(crate) mod fetch;
pub(crate) mod input;
pub(crate) mod jump;
pub(crate) mod list;
pub(crate) mod mouse;
pub(crate) mod settings;
pub(crate) mod tick;

use crate::i18n::I18n;
use crate::theme::Theme;

use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::time::Instant;

use ratatui::widgets::ListState;

use crate::models::{AppEvent, BusStop, Route, ShuttleServiceResult};

/// Number of interactive rows in the settings overlay (interval / view / language).
pub const SETTINGS_ROW_COUNT: usize = 3;

static STOPS_TOML: &str = include_str!("../../assets/stops.toml");
static ROUTES_TOML: &str = include_str!("../../assets/routes.toml");

#[derive(serde::Deserialize)]
struct StopsFile {
    stops: Vec<BusStop>,
}

#[derive(serde::Deserialize)]
struct RoutesFile {
    routes: Vec<Route>,
}

/// Cached API response for one stop.
pub struct CachedData {
    pub result: ShuttleServiceResult,
    pub fetched_at: Instant,
    /// Non-fatal: we keep stale data visible and surface the message.
    pub error: Option<String>,
}

pub struct App {
    /// All stops loaded from assets/stops.toml.
    pub stops: Vec<BusStop>,
    /// All routes loaded from assets/routes.toml (includes colour).
    pub routes: Vec<Route>,
    /// Sorted + filtered indices into `stops` (favourites first, then alphabetical).
    pub sorted_indices: Vec<usize>,
    /// ratatui list state — owns the scroll offset.
    pub list_state: ListState,
    /// Index into `sorted_indices` (which item is highlighted).
    pub selected: usize,
    /// Stop names the user has starred.
    pub favourites: HashSet<String>,
    /// API results keyed by stop name.
    pub cache: HashMap<String, CachedData>,
    /// Stops that have an in-flight fetch.
    pub loading: HashSet<String>,
    /// Current search/filter text.
    pub search_query: String,
    /// Whether the search input overlay is active.
    pub searching: bool,
    /// When true, the stop list shows only favourited stops.
    pub fav_view: bool,
    /// Loaded themes (index 0 = built-in default).
    pub themes: Vec<Theme>,
    /// Index of the active theme in `themes`.
    pub theme_idx: usize,
    /// Whether the theme picker popup is open.
    pub showing_theme_picker: bool,
    /// Cursor position inside the theme picker.
    pub theme_picker_cursor: usize,
    /// Whether the settings overlay is open.
    pub showing_settings: bool,
    /// Which settings row is highlighted.
    pub settings_cursor: usize,
    /// True while the user is editing the refresh interval value.
    pub settings_edit_mode: bool,
    /// Digit buffer for the refresh interval being typed.
    pub settings_edit_buf: String,
    /// Auto-refresh interval in seconds (persisted in config).
    pub auto_refresh_secs: u64,
    /// Whether new sessions open in favourites-only view (persisted in config).
    pub default_fav_view: bool,
    /// Active i18n bundle — drives all user-visible strings.
    pub i18n: I18n,
    /// Set to true to exit the event loop.
    pub should_quit: bool,
    /// Transient status message (text, time it was set).
    pub status_msg: Option<(String, Instant)>,
    /// Digits typed so far for a number-jump (max 2).
    pub jump_buf: String,
    /// When the last jump digit was typed.
    pub jump_at: Option<Instant>,
    /// Sender used by background threads to post events.
    tx: mpsc::Sender<AppEvent>,
}

impl App {
    pub fn new(tx: mpsc::Sender<AppEvent>) -> Self {
        let stops: Vec<BusStop> = toml::from_str::<StopsFile>(STOPS_TOML)
            .expect("assets/stops.toml is invalid")
            .stops;

        let routes: Vec<Route> = toml::from_str::<RoutesFile>(ROUTES_TOML)
            .expect("assets/routes.toml is invalid")
            .routes;

        let config = crate::config::load();
        let favourites: HashSet<String> = config.favourites.into_iter().collect();
        let auto_refresh_secs = config.refresh_interval_secs;
        let default_fav_view = config.default_fav_view;
        let i18n = I18n::new(&config.language);

        let mut app = App {
            stops,
            routes,
            sorted_indices: Vec::new(),
            list_state: ListState::default(),
            selected: 0,
            favourites,
            cache: HashMap::new(),
            loading: HashSet::new(),
            search_query: String::new(),
            searching: false,
            fav_view: default_fav_view,
            themes: crate::theme::load_themes(),
            theme_idx: 0,
            showing_theme_picker: false,
            theme_picker_cursor: 0,
            showing_settings: false,
            settings_cursor: 0,
            settings_edit_mode: false,
            settings_edit_buf: String::new(),
            auto_refresh_secs,
            default_fav_view,
            i18n,
            should_quit: false,
            status_msg: None,
            jump_buf: String::new(),
            jump_at: None,
            tx,
        };
        app.rebuild_list();
        app.list_state.select(Some(0));
        app
    }

    pub fn theme(&self) -> &Theme {
        &self.themes[self.theme_idx]
    }

    /// Build a serialisable snapshot of current config state for persistence.
    pub fn config_snapshot(&self) -> crate::models::Config {
        let mut favs: Vec<String> = self.favourites.iter().cloned().collect();
        favs.sort();
        crate::models::Config {
            favourites: favs,
            refresh_interval_secs: self.auto_refresh_secs,
            default_fav_view: self.default_fav_view,
            language: self.i18n.lang.clone(),
        }
    }
}
