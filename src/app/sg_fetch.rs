#![cfg(not(target_arch = "wasm32"))]

use crate::models::{AppEvent, SgArrivalResult};
use crate::time::Instant;

use super::{App, SgCachedData};

impl App {
    /// Fetch SG arrival data for current SG stop if not already cached or in-flight.
    pub fn ensure_sg_data(&mut self) {
        if let Some(stop) = self.current_sg_stop() {
            let code = stop.code.clone();
            if !self.sg_fetch.cache.contains_key(&code) && !self.sg_fetch.loading.contains(&code) {
                self.start_sg_fetch(code);
            }
        }
    }

    /// Unconditional refresh of the current SG stop.
    pub fn refresh_current_sg(&mut self) {
        if let Some(stop) = self.current_sg_stop() {
            let code = stop.code.clone();
            if !self.sg_fetch.loading.contains(&code) {
                self.start_sg_fetch(code);
                let msg = self.i18n.t("status-refreshing");
                self.set_status(&msg);
            }
        }
    }

    /// Spawn a background thread to fetch SG arrival data.
    pub(super) fn start_sg_fetch(&mut self, stop_code: String) {
        self.sg_fetch.loading.insert(stop_code.clone());
        let tx = self.sg_fetch.tx.clone();
        std::thread::spawn(move || {
            let event = match crate::sg_api::fetch_sg_arrival(&stop_code) {
                Ok(data) => AppEvent::SgDataReceived { stop_code, data },
                Err(e) => AppEvent::SgFetchError {
                    stop_code,
                    error: e,
                },
            };
            let _ = tx.send(event);
        });
    }

    /// Spawn a background thread to load all SG stops.
    pub fn start_sg_stops_fetch(&mut self) {
        if self.sg_nav.stops_loading {
            return; // already in progress
        }
        self.sg_nav.stops_loading = true;
        let tx = self.sg_fetch.tx.clone();
        std::thread::spawn(move || {
            crate::sg_api::fetch_all_sg_stops(tx);
        });
    }

    pub fn handle_sg_data(&mut self, stop_code: String, data: SgArrivalResult) {
        self.sg_fetch.loading.remove(&stop_code);
        self.sg_fetch.cache.insert(
            stop_code,
            SgCachedData {
                result: data,
                fetched_at: Instant::now(),
                error: None,
            },
        );
    }

    pub fn handle_sg_error(&mut self, stop_code: String, error: String) {
        self.sg_fetch.loading.remove(&stop_code);
        if let Some(cached) = self.sg_fetch.cache.get_mut(&stop_code) {
            cached.error = Some(error);
            cached.fetched_at = Instant::now();
        } else {
            self.sg_fetch.cache.insert(
                stop_code,
                SgCachedData {
                    result: crate::models::SgArrivalResult {
                        bus_stop_code: String::new(),
                        services: vec![],
                    },
                    fetched_at: Instant::now(),
                    error: Some(error),
                },
            );
        }
    }

    pub fn handle_sg_stops_loaded(&mut self, stops: Vec<crate::models::SgBusStop>) {
        self.sg_nav.stops_load_progress = stops.len();

        // If the total is not a multiple of 500, or is 0, loading is complete.
        let is_final = !stops.len().is_multiple_of(500) || stops.is_empty();
        if is_final {
            self.sg_nav.stops_loading = false;
            self.sg_nav.stops_error = None;
            // Save to disk cache
            crate::config::save_sg_stops(&stops);
        }
        self.domain.sg_stops = stops;
        self.rebuild_sg_list();
        // Fetch arrival data for current SG stop if mode is SG
        if self.mode == crate::models::AppMode::SgPublicBus {
            self.ensure_sg_data();
        }
    }

    pub fn handle_sg_stops_error(&mut self, error: String) {
        self.sg_nav.stops_loading = false;
        self.sg_nav.stops_error = Some(error);
    }

    /// How many seconds until the SG auto-refresh fires for the current stop.
    pub fn sg_seconds_until_refresh(&self) -> Option<u64> {
        let code = self.current_sg_stop()?.code.clone();
        if self.sg_fetch.loading.contains(&code) {
            return None;
        }
        let elapsed = self
            .sg_fetch
            .cache
            .get(&code)?
            .fetched_at
            .elapsed()
            .as_secs();
        Some(self.settings.auto_refresh_secs.saturating_sub(elapsed))
    }
}
