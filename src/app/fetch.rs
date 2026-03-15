use crate::time::Instant;

use crate::models::{AppEvent, ShuttleServiceResult};

use super::{App, CachedData};

impl App {
    /// Fetch data for the current stop only if not already cached or in-flight.
    pub fn ensure_data(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if !self.fetch.cache.contains_key(&name) && !self.fetch.loading.contains(&name) {
                self.start_fetch(name);
            }
        }
    }

    /// Unconditional refresh of the current stop (skips only if already in-flight).
    pub fn refresh_current(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if !self.fetch.loading.contains(&name) {
                self.start_fetch(name);
                let msg = self.i18n.t("status-refreshing");
                self.set_status(&msg);
            }
        }
    }

    /// Spawn a background thread to fetch data for `stop_name`.
    pub(super) fn start_fetch(&mut self, stop_name: String) {
        self.fetch.loading.insert(stop_name.clone());
        let tx = self.fetch.tx.clone();

        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let event = match crate::api::fetch_shuttle_service(&stop_name) {
                Ok(data) => AppEvent::DataReceived { stop_name, data },
                Err(e) => AppEvent::FetchError {
                    stop_name,
                    error: e,
                },
            };
            let _ = tx.send(event);
        });

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let event = match crate::api::fetch_shuttle_service_async(&stop_name).await {
                    Ok(data) => AppEvent::DataReceived { stop_name, data },
                    Err(e) => AppEvent::FetchError {
                        stop_name,
                        error: e,
                    },
                };
                let _ = tx.send(event);
            });
        }
    }

    pub fn handle_data(&mut self, stop_name: String, data: ShuttleServiceResult) {
        self.fetch.loading.remove(&stop_name);
        self.fetch.cache.insert(
            stop_name,
            CachedData {
                result: data,
                fetched_at: Instant::now(),
                error: None,
            },
        );
    }

    pub fn handle_error(&mut self, stop_name: String, error: String) {
        self.fetch.loading.remove(&stop_name);
        if let Some(cached) = self.fetch.cache.get_mut(&stop_name) {
            cached.error = Some(error);
            cached.fetched_at = Instant::now();
        } else {
            self.fetch.cache.insert(
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

#[cfg(test)]
mod tests {
    use super::{App, CachedData};
    use crate::models::ShuttleServiceResult;
    use crate::time::Instant;
    use std::sync::mpsc;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        App::new_test(tx)
    }

    fn empty_result(name: &str) -> ShuttleServiceResult {
        ShuttleServiceResult {
            name: Some(name.to_string()),
            caption: None,
            shuttles: vec![],
            timestamp: None,
        }
    }

    #[test]
    fn handle_data_inserts_into_cache() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.fetch.loading.insert(stop.clone());
        app.handle_data(stop.clone(), empty_result(&stop));
        assert!(app.fetch.cache.contains_key(&stop));
    }

    #[test]
    fn handle_data_removes_from_loading() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.fetch.loading.insert(stop.clone());
        app.handle_data(stop.clone(), empty_result(&stop));
        assert!(!app.fetch.loading.contains(&stop));
    }

    #[test]
    fn handle_data_stores_no_error() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.fetch.loading.insert(stop.clone());
        app.handle_data(stop.clone(), empty_result(&stop));
        assert!(app.fetch.cache[&stop].error.is_none());
    }

    #[test]
    fn handle_error_without_cache_creates_error_entry() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.fetch.loading.insert(stop.clone());
        app.handle_error(stop.clone(), "timeout".to_string());
        assert!(!app.fetch.loading.contains(&stop));
        let cached = app.fetch.cache.get(&stop).unwrap();
        assert_eq!(cached.error.as_deref(), Some("timeout"));
        assert!(cached.result.shuttles.is_empty());
    }

    #[test]
    fn handle_error_with_cache_keeps_data_updates_error() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.fetch.cache.insert(
            stop.clone(),
            CachedData {
                result: empty_result(&stop),
                fetched_at: Instant::now(),
                error: None,
            },
        );
        app.fetch.loading.insert(stop.clone());
        app.handle_error(stop.clone(), "network error".to_string());
        let cached = app.fetch.cache.get(&stop).unwrap();
        assert_eq!(cached.error.as_deref(), Some("network error"));
        assert_eq!(cached.result.name.as_deref(), Some(stop.as_str()));
    }

    #[test]
    fn ensure_data_marks_stop_loading() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.ensure_data();
        assert!(app.fetch.loading.contains(&stop));
    }

    #[test]
    fn ensure_data_skips_if_cached() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.fetch.cache.insert(
            stop.clone(),
            CachedData {
                result: empty_result(&stop),
                fetched_at: Instant::now(),
                error: None,
            },
        );
        app.ensure_data();
        assert!(!app.fetch.loading.contains(&stop));
    }

    #[test]
    fn ensure_data_skips_if_already_loading() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.fetch.loading.insert(stop.clone());
        app.ensure_data();
        assert!(app.fetch.loading.contains(&stop));
    }
}
