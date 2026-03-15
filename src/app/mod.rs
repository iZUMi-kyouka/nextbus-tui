pub(crate) mod favourites;
pub(crate) mod fetch;
pub(crate) mod input;
pub(crate) mod jump;
pub(crate) mod list;
pub(crate) mod mouse;
pub(crate) mod tick;

use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::time::Instant;

use ratatui::widgets::ListState;

use crate::models::{AppEvent, BusStop, Route, ShuttleServiceResult};

pub const AUTO_REFRESH_SECS: u64 = 30;

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
            fav_view: false,
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
}
