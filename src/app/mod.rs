pub(crate) mod favourites;
pub(crate) mod fetch;
pub(crate) mod input;
pub(crate) mod jump;
pub(crate) mod mouse;
pub(crate) mod nav;
pub(crate) mod overlay;
pub(crate) mod tick;

// SG-specific modules (native only for fetch, but nav + types universal)
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod sg_fetch;
pub(crate) mod sg_nav;

use crate::i18n::I18n;
use crate::message::Message;
use crate::models::{
    AppEvent, AppMode, BusStop, Route, SgArrivalResult, SgBusStop, ShuttleServiceResult,
};
use crate::theme::{Theme, ThemeMode};

use crate::time::Instant;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;

use ratatui::widgets::ListState;

/// Number of interactive rows in the settings overlay
/// (interval / view / language / theme mode / default mode).
pub const SETTINGS_ROW_COUNT: usize = 5;

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

/// Cached NUS API response for one stop.
pub struct CachedData {
    pub result: ShuttleServiceResult,
    pub fetched_at: Instant,
    /// Non-fatal: we keep stale data visible and surface the message.
    pub error: Option<String>,
}

/// Cached SG arrival data for one stop.
pub struct SgCachedData {
    pub result: SgArrivalResult,
    pub fetched_at: Instant,
    pub error: Option<String>,
}

// ── Sub-structs ───────────────────────────────────────────────────────────────

/// Static data loaded at startup; never mutated after init.
pub struct Domain {
    pub stops: Vec<BusStop>,
    pub routes: Vec<Route>,
    pub themes: Vec<Theme>,
    pub sg_stops: Vec<SgBusStop>,
}

impl Domain {
    fn load_embedded() -> Self {
        let stops: Vec<BusStop> = toml::from_str::<StopsFile>(STOPS_TOML)
            .expect("assets/stops.toml is invalid")
            .stops;
        let routes: Vec<Route> = toml::from_str::<RoutesFile>(ROUTES_TOML)
            .expect("assets/routes.toml is invalid")
            .routes;
        let themes = crate::theme::load_themes();
        Domain {
            stops,
            routes,
            themes,
            sg_stops: Vec::new(),
        }
    }
}

/// Persisted user settings. Any mutation → call `settings.persist(lang)`.
pub struct Settings {
    pub favourites: HashSet<String>,
    pub sg_favourites: HashSet<String>,
    pub auto_refresh_secs: u64,
    pub default_fav_view: bool,
    pub default_mode: AppMode,
    pub theme_mode: ThemeMode,
    pub theme_idx: usize,
}

impl Settings {
    fn from_config(cfg: &crate::models::Config) -> Self {
        Settings {
            favourites: cfg.favourites.iter().cloned().collect(),
            sg_favourites: cfg.sg_favourites.iter().cloned().collect(),
            auto_refresh_secs: cfg.refresh_interval_secs,
            default_fav_view: cfg.default_fav_view,
            default_mode: cfg.default_mode,
            theme_mode: cfg.theme_mode,
            theme_idx: cfg.theme_idx,
        }
    }

    /// Persist the current settings to disk.
    pub fn persist(&self, lang: &str) {
        let mut favs: Vec<String> = self.favourites.iter().cloned().collect();
        favs.sort();
        let mut sg_favs: Vec<String> = self.sg_favourites.iter().cloned().collect();
        sg_favs.sort();
        let cfg = crate::models::Config {
            favourites: favs,
            sg_favourites: sg_favs,
            refresh_interval_secs: self.auto_refresh_secs,
            default_fav_view: self.default_fav_view,
            default_mode: self.default_mode,
            language: lang.to_owned(),
            theme_mode: self.theme_mode,
            theme_idx: self.theme_idx,
        };
        crate::config::save(&cfg);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            favourites: HashSet::new(),
            sg_favourites: HashSet::new(),
            auto_refresh_secs: 20,
            default_fav_view: false,
            default_mode: AppMode::NusCampus,
            theme_mode: ThemeMode::Dark,
            theme_idx: 0,
        }
    }
}

/// Ephemeral NUS navigation / input state. Never persisted.
#[derive(Default)]
pub struct NavState {
    pub selected: usize,
    pub sorted_indices: Vec<usize>,
    pub list_state: ListState,
    pub search_query: String,
    pub searching: bool,
    pub fav_view: bool,
    pub jump_buf: String,
    pub jump_at: Option<Instant>,
    /// Inner height of the rendered list panel (rows available for items).
    /// Set by `render_list` each frame; used to clamp scroll when near the bottom.
    pub list_height: u16,
}

/// Ephemeral SG navigation / input state. Never persisted.
#[derive(Default)]
pub struct SgNavState {
    pub selected: usize,
    pub sorted_indices: Vec<usize>,
    pub list_state: ListState,
    pub search_query: String,
    pub searching: bool,
    pub fav_view: bool,
    pub jump_buf: String,
    pub jump_at: Option<Instant>,
    pub list_height: u16,
    pub stops_loading: bool,
    pub stops_error: Option<String>,
    pub stops_load_progress: usize,
}

/// Transient UI overlay state. Never persisted.
#[derive(Default)]
pub struct OverlayState {
    pub showing_theme_picker: bool,
    pub theme_picker_cursor: usize,
    /// Theme index active when the picker was opened; restored if user cancels.
    pub original_theme_idx: usize,
    pub showing_lang_picker: bool,
    pub lang_picker_cursor: usize,
    pub showing_settings: bool,
    pub settings_cursor: usize,
    pub settings_edit_mode: bool,
    pub settings_edit_buf: String,
    pub status_msg: Option<(String, Instant)>,
}

/// Network fetch runtime state (NUS).
pub struct FetchState {
    pub cache: HashMap<String, CachedData>,
    pub loading: HashSet<String>,
    pub tx: mpsc::Sender<AppEvent>,
}

impl FetchState {
    fn new(tx: mpsc::Sender<AppEvent>) -> Self {
        FetchState {
            cache: HashMap::new(),
            loading: HashSet::new(),
            tx,
        }
    }
}

/// Network fetch runtime state (SG).
pub struct SgFetchState {
    pub cache: HashMap<String, SgCachedData>,
    pub loading: HashSet<String>,
    pub tx: mpsc::Sender<AppEvent>,
}

impl SgFetchState {
    fn new(tx: mpsc::Sender<AppEvent>) -> Self {
        SgFetchState {
            cache: HashMap::new(),
            loading: HashSet::new(),
            tx,
        }
    }
}

// ── Top-level coordinator ─────────────────────────────────────────────────────

pub struct App {
    pub domain: Domain,
    pub settings: Settings,
    pub nav: NavState,
    pub sg_nav: SgNavState,
    pub overlay: OverlayState,
    pub fetch: FetchState,
    pub sg_fetch: SgFetchState,
    pub mode: AppMode,
    pub i18n: I18n,
    pub should_quit: bool,
    /// `false` while the terminal window does not have focus.
    pub focused: bool,
}

impl App {
    pub fn new(tx: mpsc::Sender<AppEvent>) -> Self {
        let config = crate::config::load();
        let i18n = I18n::new(&config.language);
        let settings = Settings::from_config(&config);
        let fav_view = settings.default_fav_view;
        let default_mode = settings.default_mode;

        let mut domain = Domain::load_embedded();

        // Load SG stops from disk cache if available (native only)
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(cache) = crate::config::load_sg_stops() {
            // Accept if < 7 days old
            if let Ok(cached_at) = chrono::DateTime::parse_from_rfc3339(&cache.cached_at) {
                let age =
                    chrono::Utc::now().signed_duration_since(cached_at.with_timezone(&chrono::Utc));
                if age.num_days() < 7 {
                    domain.sg_stops = cache.stops;
                }
            }
        }

        let sg_fetch = SgFetchState::new(tx.clone());
        let fetch = FetchState::new(tx);

        let mut app = App {
            domain,
            settings,
            nav: NavState {
                fav_view,
                ..NavState::default()
            },
            sg_nav: SgNavState::default(),
            overlay: OverlayState::default(),
            fetch,
            sg_fetch,
            mode: default_mode,
            i18n,
            should_quit: false,
            focused: true,
        };
        app.rebuild_list();
        app.nav.list_state.select(Some(0));
        app.rebuild_sg_list();
        app.sg_nav.list_state.select(Some(0));
        app
    }

    #[cfg(test)]
    pub fn new_test(tx: mpsc::Sender<AppEvent>) -> Self {
        let sg_fetch = SgFetchState::new(tx.clone());
        let mut app = App {
            domain: Domain::load_embedded(),
            settings: Settings::default(),
            nav: NavState::default(),
            sg_nav: SgNavState::default(),
            overlay: OverlayState::default(),
            fetch: FetchState::new(tx),
            sg_fetch,
            mode: AppMode::NusCampus,
            i18n: I18n::new("en"),
            should_quit: false,
            focused: true,
        };
        app.rebuild_list();
        app.nav.list_state.select(Some(0));
        app.rebuild_sg_list();
        app
    }

    // ── Theme helpers ─────────────────────────────────────────────────────────

    pub fn theme(&self) -> &Theme {
        let target = self.effective_mode();
        let candidate = &self.domain.themes[self.settings.theme_idx];
        if candidate.mode == target {
            candidate
        } else {
            self.domain
                .themes
                .iter()
                .find(|t| t.mode == target)
                .unwrap_or(candidate)
        }
    }

    /// Returns the effective colour mode: `Dark` or `Light` (never `Auto`).
    fn effective_mode(&self) -> ThemeMode {
        match self.settings.theme_mode {
            ThemeMode::Auto => {
                let hour = Self::current_hour();
                if (6..18).contains(&hour) {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                }
            }
            mode => mode,
        }
    }

    fn current_hour() -> u32 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use chrono::Timelike;
            chrono::Local::now().hour()
        }
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::new_0().get_hours() as u32
        }
    }

    /// Indices into `self.domain.themes` that the theme picker should display,
    /// filtered to match the current effective mode.
    pub fn picker_theme_indices(&self) -> Vec<usize> {
        let target = self.effective_mode();
        self.domain
            .themes
            .iter()
            .enumerate()
            .filter(|(_, t)| t.mode == target)
            .map(|(i, _)| i)
            .collect()
    }

    // ── Mode switching ─────────────────────────────────────────────────────────

    pub fn switch_mode(&mut self) {
        self.mode = match self.mode {
            AppMode::NusCampus => AppMode::SgPublicBus,
            AppMode::SgPublicBus => AppMode::NusCampus,
        };
        let msg_key = match self.mode {
            AppMode::NusCampus => "status-mode-nus",
            AppMode::SgPublicBus => "status-mode-sg",
        };
        let msg = self.i18n.t(msg_key);
        self.set_status(&msg);
        #[cfg(not(target_arch = "wasm32"))]
        if self.mode == AppMode::SgPublicBus {
            if self.domain.sg_stops.is_empty() && !self.sg_nav.stops_loading {
                self.start_sg_stops_fetch();
            } else {
                self.ensure_sg_data();
            }
        }
    }

    // ── Message dispatch ──────────────────────────────────────────────────────

    pub fn update(&mut self, msg: Message) {
        match msg {
            // Navigation — routes to active mode
            Message::MoveUp => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    self.sg_move_up();
                } else {
                    self.move_up();
                }
            }
            Message::MoveDown => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    self.sg_move_down();
                } else {
                    self.move_down();
                }
            }
            Message::GoFirst => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    self.sg_go_first();
                } else {
                    self.go_first();
                }
            }
            Message::GoLast => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    self.sg_go_last();
                } else {
                    self.go_last();
                }
            }
            Message::JumpDigit(c) => {
                if self.mode == AppMode::SgPublicBus {
                    self.sg_push_jump_digit(c);
                } else {
                    self.push_jump_digit(c);
                }
            }
            Message::CommitJump => {
                if self.mode == AppMode::SgPublicBus {
                    self.sg_commit_jump();
                } else {
                    self.commit_jump();
                }
            }
            Message::CancelJump => {
                self.cancel_jump();
                self.sg_cancel_jump();
            }

            // Search
            Message::OpenSearch => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    self.sg_nav.search_query.clear();
                    self.rebuild_sg_list();
                    self.sg_nav.searching = true;
                } else {
                    self.nav.search_query.clear();
                    self.rebuild_list();
                    self.nav.searching = true;
                }
            }
            Message::CloseSearch { keep_filter } => {
                if self.mode == AppMode::SgPublicBus {
                    self.sg_nav.searching = false;
                    if !keep_filter {
                        self.sg_nav.search_query.clear();
                        self.rebuild_sg_list();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    self.ensure_sg_data();
                } else {
                    self.nav.searching = false;
                    if !keep_filter {
                        self.nav.search_query.clear();
                        self.rebuild_list();
                    }
                    self.ensure_data();
                }
            }
            Message::SearchChar(c) => {
                if self.mode == AppMode::SgPublicBus {
                    self.sg_nav.search_query.push(c);
                    self.rebuild_sg_list();
                    #[cfg(not(target_arch = "wasm32"))]
                    self.ensure_sg_data();
                } else {
                    self.nav.search_query.push(c);
                    self.rebuild_list();
                    self.ensure_data();
                }
            }
            Message::SearchBackspace => {
                if self.mode == AppMode::SgPublicBus {
                    self.sg_nav.search_query.pop();
                    self.rebuild_sg_list();
                    #[cfg(not(target_arch = "wasm32"))]
                    self.ensure_sg_data();
                } else {
                    self.nav.search_query.pop();
                    self.rebuild_list();
                    self.ensure_data();
                }
            }

            // List view
            Message::ToggleFavourite => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    self.toggle_sg_favourite();
                } else {
                    self.toggle_favourite();
                }
            }
            Message::ToggleFavView => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    self.sg_nav.fav_view = !self.sg_nav.fav_view;
                    self.sg_nav.search_query.clear();
                    self.rebuild_sg_list();
                    #[cfg(not(target_arch = "wasm32"))]
                    self.ensure_sg_data();
                } else {
                    self.nav.fav_view = !self.nav.fav_view;
                    self.nav.search_query.clear();
                    self.rebuild_list();
                    self.ensure_data();
                }
            }

            // Theme
            Message::CycleTheme => {
                self.cancel_jump();
                self.sg_cancel_jump();
                let indices = self.picker_theme_indices();
                if !indices.is_empty() {
                    let pos = indices
                        .iter()
                        .position(|&i| i == self.settings.theme_idx)
                        .unwrap_or(0);
                    self.settings.theme_idx = indices[(pos + 1) % indices.len()];
                    self.settings.persist(&self.i18n.lang);
                }
            }
            Message::OpenThemePicker => {
                self.cancel_jump();
                self.sg_cancel_jump();
                let indices = self.picker_theme_indices();
                self.overlay.theme_picker_cursor = indices
                    .iter()
                    .position(|&i| i == self.settings.theme_idx)
                    .unwrap_or(0);
                self.overlay.original_theme_idx = self.settings.theme_idx;
                self.overlay.showing_theme_picker = true;
            }
            Message::CloseThemePicker => {
                // User cancelled — restore the theme that was active before opening.
                self.settings.theme_idx = self.overlay.original_theme_idx;
                self.overlay.showing_theme_picker = false;
            }
            Message::ThemePickerUp => {
                if self.overlay.theme_picker_cursor > 0 {
                    self.overlay.theme_picker_cursor -= 1;
                }
                // Live preview: apply hovered theme immediately.
                let indices = self.picker_theme_indices();
                if let Some(&idx) = indices.get(self.overlay.theme_picker_cursor) {
                    self.settings.theme_idx = idx;
                }
            }
            Message::ThemePickerDown => {
                let n = self.picker_theme_indices().len();
                if self.overlay.theme_picker_cursor + 1 < n {
                    self.overlay.theme_picker_cursor += 1;
                }
                // Live preview: apply hovered theme immediately.
                let indices = self.picker_theme_indices();
                if let Some(&idx) = indices.get(self.overlay.theme_picker_cursor) {
                    self.settings.theme_idx = idx;
                }
            }
            Message::ThemePickerApply => {
                let indices = self.picker_theme_indices();
                if let Some(&idx) = indices.get(self.overlay.theme_picker_cursor) {
                    self.settings.theme_idx = idx;
                    self.settings.persist(&self.i18n.lang);
                }
                self.overlay.showing_theme_picker = false;
            }

            // Language picker
            Message::CloseLangPicker => {
                self.overlay.showing_lang_picker = false;
            }
            Message::LangPickerUp => {
                if self.overlay.lang_picker_cursor > 0 {
                    self.overlay.lang_picker_cursor -= 1;
                }
            }
            Message::LangPickerDown => {
                let n = crate::i18n::LANGUAGES.len();
                if self.overlay.lang_picker_cursor + 1 < n {
                    self.overlay.lang_picker_cursor += 1;
                }
            }
            Message::LangPickerApply => self.apply_lang_picker(),

            // Settings overlay
            Message::OpenSettings => {
                self.cancel_jump();
                self.sg_cancel_jump();
                self.overlay.settings_cursor = 0;
                self.overlay.settings_edit_mode = false;
                self.overlay.settings_edit_buf.clear();
                self.overlay.showing_settings = true;
            }
            Message::CloseSettings => {
                self.overlay.showing_settings = false;
            }
            Message::SettingsUp => {
                if self.overlay.settings_cursor > 0 {
                    self.overlay.settings_cursor -= 1;
                }
            }
            Message::SettingsDown => {
                if self.overlay.settings_cursor + 1 < SETTINGS_ROW_COUNT {
                    self.overlay.settings_cursor += 1;
                }
            }
            Message::SettingsActivateRow => self.activate_settings_row(),
            Message::SettingsEditChar(c) => {
                if self.overlay.settings_edit_buf.len() < 3 {
                    self.overlay.settings_edit_buf.push(c);
                }
            }
            Message::SettingsEditBackspace => {
                self.overlay.settings_edit_buf.pop();
            }
            Message::SettingsEditCancel => {
                self.overlay.settings_edit_mode = false;
                self.overlay.settings_edit_buf.clear();
            }
            Message::SettingsEditCommit => self.commit_refresh_interval(),

            // Mode switching
            Message::SwitchMode => self.switch_mode(),

            // Background events (NUS)
            Message::Tick => self.handle_tick(),
            Message::DataReceived { stop_name, data } => self.handle_data(stop_name, data),
            Message::FetchError { stop_name, error } => self.handle_error(stop_name, error),

            // Background events (SG)
            Message::SgDataReceived { stop_code, data } => {
                #[cfg(not(target_arch = "wasm32"))]
                self.handle_sg_data(stop_code, data);
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = (stop_code, data);
                }
            }
            Message::SgFetchError { stop_code, error } => {
                #[cfg(not(target_arch = "wasm32"))]
                self.handle_sg_error(stop_code, error);
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = (stop_code, error);
                }
            }
            Message::SgStopsLoaded { stops } => {
                #[cfg(not(target_arch = "wasm32"))]
                self.handle_sg_stops_loaded(stops);
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = stops;
                }
            }
            Message::SgStopsError { error } => {
                #[cfg(not(target_arch = "wasm32"))]
                self.handle_sg_stops_error(error);
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = error;
                }
            }

            // Mouse
            Message::ListClick(target) => {
                self.cancel_jump();
                self.sg_cancel_jump();
                if self.mode == AppMode::SgPublicBus {
                    if target < self.sg_nav.sorted_indices.len() {
                        self.sg_nav.selected = target;
                        self.sg_nav.list_state.select(Some(target));
                        #[cfg(not(target_arch = "wasm32"))]
                        self.ensure_sg_data();
                    }
                } else {
                    if target < self.nav.sorted_indices.len() {
                        self.nav.selected = target;
                        self.nav.list_state.select(Some(target));
                        self.ensure_data();
                    }
                }
            }
            Message::ScrollListUp => {
                if self.mode == AppMode::SgPublicBus {
                    self.sg_scroll_up();
                } else {
                    self.scroll_up();
                }
            }
            Message::ScrollListDown => {
                if self.mode == AppMode::SgPublicBus {
                    self.sg_scroll_down();
                } else {
                    self.scroll_down();
                }
            }

            // Focus
            Message::FocusGained => {
                self.focused = true;
                // Immediately refresh the current stop so the display is
                // up-to-date after the window was in the background.
                if self.mode == AppMode::SgPublicBus {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.ensure_sg_data();
                } else {
                    self.ensure_data();
                }
            }
            Message::FocusLost => {
                self.focused = false;
            }

            // Control
            Message::RefreshCurrent => {
                self.cancel_jump();
                self.sg_cancel_jump();
                #[cfg(not(target_arch = "wasm32"))]
                if self.mode == AppMode::SgPublicBus {
                    self.refresh_current_sg();
                } else {
                    self.refresh_current();
                }
                #[cfg(target_arch = "wasm32")]
                self.refresh_current();
            }
            Message::Quit => {
                self.cancel_jump();
                self.sg_cancel_jump();
                self.should_quit = true;
            }
        }
    }
}
