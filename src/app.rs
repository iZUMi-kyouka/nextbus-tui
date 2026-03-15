use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ratatui::widgets::ListState;

use crate::models::{AppEvent, BusStop, Route, ShuttleServiceResult};

pub const AUTO_REFRESH_SECS: u64 = 30;

static STOPS_TOML: &str = include_str!("../assets/stops.toml");
static ROUTES_TOML: &str = include_str!("../assets/routes.toml");

#[derive(serde::Deserialize)]
struct StopsFile {
    stops: Vec<BusStop>,
}

#[derive(serde::Deserialize)]
struct RoutesFile {
    routes: Vec<Route>,
}

/// Cached API response for one stop
pub struct CachedData {
    pub result: ShuttleServiceResult,
    pub fetched_at: Instant,
    /// Non-fatal: we keep old data on error and show the message
    pub error: Option<String>,
}

pub struct App {
    /// All stops from stops.toml
    pub stops: Vec<BusStop>,
    /// All routes from routes.toml (includes colour)
    pub routes: Vec<Route>,
    /// Sorted + filtered indices into `stops` (favourites first, then alphabetical)
    pub sorted_indices: Vec<usize>,
    /// ratatui list state; owns the scroll offset
    pub list_state: ListState,
    /// Index into `sorted_indices` (which item is highlighted)
    pub selected: usize,
    /// Stop names the user has starred
    pub favourites: HashSet<String>,
    /// API results keyed by stop name
    pub cache: HashMap<String, CachedData>,
    /// Stops that have an in-flight fetch
    pub loading: HashSet<String>,
    /// Current search/filter text
    pub search_query: String,
    /// Whether the search input is active
    pub searching: bool,
    /// Set to true to exit the app
    pub should_quit: bool,
    /// Transient status message (text, time set)
    pub status_msg: Option<(String, Instant)>,
    /// Digits typed so far for a number-jump (max 2)
    pub jump_buf: String,
    /// When the last jump digit was typed
    pub jump_at: Option<Instant>,
    /// Sender for posting events from background threads
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

    // ── List management ────────────────────────────────────────────────────────

    /// Rebuild `sorted_indices` applying the current search filter.
    /// Favourites come first (alphabetical), then the rest (alphabetical).
    pub fn rebuild_list(&mut self) {
        let q = self.search_query.to_lowercase();
        let matches = |s: &BusStop| -> bool {
            q.is_empty()
                || s.caption.to_lowercase().contains(&q)
                || s.name.to_lowercase().contains(&q)
        };

        let mut favs: Vec<usize> = self
            .stops
            .iter()
            .enumerate()
            .filter(|(_, s)| self.favourites.contains(&s.name) && matches(s))
            .map(|(i, _)| i)
            .collect();
        favs.sort_by(|&a, &b| self.stops[a].caption.cmp(&self.stops[b].caption));

        let mut rest: Vec<usize> = self
            .stops
            .iter()
            .enumerate()
            .filter(|(_, s)| !self.favourites.contains(&s.name) && matches(s))
            .map(|(i, _)| i)
            .collect();
        rest.sort_by(|&a, &b| self.stops[a].caption.cmp(&self.stops[b].caption));

        favs.extend(rest);
        self.sorted_indices = favs;

        // Clamp selection so it stays in bounds
        if !self.sorted_indices.is_empty() {
            self.selected = self.selected.min(self.sorted_indices.len() - 1);
        } else {
            self.selected = 0;
        }
        self.list_state.select(Some(self.selected));
    }

    /// The currently highlighted bus stop, if any.
    pub fn current_stop(&self) -> Option<&BusStop> {
        self.sorted_indices.get(self.selected).map(|&i| &self.stops[i])
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
            self.ensure_data();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.sorted_indices.len() {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
            self.ensure_data();
        }
    }

    pub fn go_first(&mut self) {
        if !self.sorted_indices.is_empty() {
            self.selected = 0;
            self.list_state.select(Some(0));
            self.ensure_data();
        }
    }

    pub fn go_last(&mut self) {
        if !self.sorted_indices.is_empty() {
            self.selected = self.sorted_indices.len() - 1;
            self.list_state.select(Some(self.selected));
            self.ensure_data();
        }
    }

    // ── Data fetching ──────────────────────────────────────────────────────────

    /// Fetch only if we don't already have (or are fetching) data for this stop.
    pub fn ensure_data(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if !self.cache.contains_key(&name) && !self.loading.contains(&name) {
                self.start_fetch(name);
            }
        }
    }

    /// Unconditional refresh of the current stop (skips only if already in flight).
    pub fn refresh_current(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if !self.loading.contains(&name) {
                self.start_fetch(name);
                self.set_status("Refreshing...");
            }
        }
    }

    fn start_fetch(&mut self, stop_name: String) {
        self.loading.insert(stop_name.clone());
        let tx = self.tx.clone();
        thread::spawn(move || {
            let event = match crate::api::fetch_shuttle_service(&stop_name) {
                Ok(data) => AppEvent::DataReceived { stop_name, data },
                Err(e) => AppEvent::FetchError {
                    stop_name,
                    error: e,
                },
            };
            let _ = tx.send(event);
        });
    }

    // ── Event handlers ─────────────────────────────────────────────────────────

    pub fn handle_tick(&mut self) {
        // Expire status messages after 3 s
        if let Some((_, at)) = &self.status_msg {
            if at.elapsed() > Duration::from_secs(3) {
                self.status_msg = None;
            }
        }

        // Commit a pending single-digit jump after 1 s with no second digit
        if !self.jump_buf.is_empty() {
            if self.jump_at.map(|t| t.elapsed() >= Duration::from_secs(1)).unwrap_or(false) {
                self.commit_jump();
            }
        }

        // Auto-refresh if stale
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            let stale = self
                .cache
                .get(&name)
                .map(|c| c.fetched_at.elapsed() >= Duration::from_secs(AUTO_REFRESH_SECS))
                .unwrap_or(false);
            if stale && !self.loading.contains(&name) {
                self.start_fetch(name);
            }
        }
    }

    pub fn handle_data(&mut self, stop_name: String, data: ShuttleServiceResult) {
        self.loading.remove(&stop_name);
        self.cache.insert(
            stop_name,
            CachedData {
                result: data,
                fetched_at: Instant::now(),
                error: None,
            },
        );
    }

    pub fn handle_error(&mut self, stop_name: String, error: String) {
        self.loading.remove(&stop_name);
        if let Some(cached) = self.cache.get_mut(&stop_name) {
            // Keep the old data but surface the error
            cached.error = Some(error);
            // Reset timer so we don't immediately retry
            cached.fetched_at = Instant::now();
        } else {
            self.cache.insert(
                stop_name.clone(),
                CachedData {
                    result: ShuttleServiceResult {
                        name: Some(stop_name),
                        caption: None,
                        shuttles: vec![],
                        timestamp: None,
                    },
                    fetched_at: Instant::now(),
                    error: Some(error),
                },
            );
        }
    }

    // ── Number jump ────────────────────────────────────────────────────────────

    /// Called when a digit key is pressed in normal mode.
    pub fn push_jump_digit(&mut self, digit: char) {
        self.jump_buf.push(digit);
        self.jump_at = Some(Instant::now());
        if self.jump_buf.len() == 2 {
            self.commit_jump();
        }
    }

    /// Cancel any in-progress jump without navigating.
    pub fn cancel_jump(&mut self) {
        self.jump_buf.clear();
        self.jump_at = None;
    }

    pub fn commit_jump(&mut self) {
        if let Ok(n) = self.jump_buf.parse::<usize>() {
            if n > 0 && n <= self.sorted_indices.len() {
                self.selected = n - 1;
                self.list_state.select(Some(self.selected));
                self.ensure_data();
            }
        }
        self.jump_buf.clear();
        self.jump_at = None;
    }

    // ── Favourites ─────────────────────────────────────────────────────────────

    pub fn toggle_favourite(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if self.favourites.contains(&name) {
                self.favourites.remove(&name);
                self.set_status("Removed from favourites");
            } else {
                self.favourites.insert(name);
                self.set_status("Added to favourites *");
            }
            crate::config::save(&self.favourites);
            self.rebuild_list();
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    pub fn set_status(&mut self, msg: &str) {
        self.status_msg = Some((msg.to_string(), Instant::now()));
    }

    /// Seconds until auto-refresh fires for the current stop, or None if loading.
    pub fn seconds_until_refresh(&self) -> Option<u64> {
        let name = self.current_stop()?.name.clone();
        if self.loading.contains(&name) {
            return None;
        }
        let elapsed = self.cache.get(&name)?.fetched_at.elapsed().as_secs();
        Some(AUTO_REFRESH_SECS.saturating_sub(elapsed))
    }

    /// How many of the visible stops are favourites (used by ui for separator).
    pub fn fav_count_in_list(&self) -> usize {
        self.sorted_indices
            .iter()
            .filter(|&&i| self.favourites.contains(&self.stops[i].name))
            .count()
    }

}
