use std::thread;
use std::time::Instant;

use crate::models::{AppEvent, ShuttleServiceResult};

use super::{App, CachedData};

impl App {
    /// Fetch data for the current stop only if not already cached or in-flight.
    pub fn ensure_data(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if !self.cache.contains_key(&name) && !self.loading.contains(&name) {
                self.start_fetch(name);
            }
        }
    }

    /// Unconditional refresh of the current stop (skips only if already in-flight).
    pub fn refresh_current(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if !self.loading.contains(&name) {
                self.start_fetch(name);
                self.set_status("Refreshing...");
            }
        }
    }

    /// Spawn a background thread to fetch data for `stop_name`.
    pub(super) fn start_fetch(&mut self, stop_name: String) {
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
            // Keep stale data visible; just surface the error and reset the timer.
            cached.error = Some(error);
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
}
