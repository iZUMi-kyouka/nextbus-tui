use std::time::Duration;

use super::App;

impl App {
    /// Called every 500 ms by the background tick thread.
    /// Handles status message expiry, jump timeout, and auto-refresh.
    pub fn handle_tick(&mut self) {
        // Expire transient status messages after 3 s.
        if let Some((_, at)) = &self.overlay.status_msg {
            if at.elapsed() > Duration::from_secs(3) {
                self.overlay.status_msg = None;
            }
        }

        // Commit a pending single-digit NUS jump after 1 s with no second digit.
        if !self.nav.jump_buf.is_empty()
            && self
                .nav
                .jump_at
                .map(|t| t.elapsed() >= Duration::from_secs(1))
                .unwrap_or(false)
        {
            self.commit_jump();
        }

        // Commit a pending single-digit SG jump after 1 s with no second digit.
        if !self.sg_nav.jump_buf.is_empty()
            && self
                .sg_nav
                .jump_at
                .map(|t| t.elapsed() >= Duration::from_secs(1))
                .unwrap_or(false)
        {
            self.sg_commit_jump();
        }

        // Debounce NUS navigation: fire the fetch 300 ms after the last move.
        if let Some(t) = self.nav.last_nav_at {
            if t.elapsed() >= Duration::from_millis(300) {
                self.nav.last_nav_at = None;
                self.ensure_data();
            }
        }

        // Debounce SG navigation: fire the fetch 300 ms after the last move.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(t) = self.sg_nav.last_nav_at {
            if t.elapsed() >= Duration::from_millis(300) {
                self.sg_nav.last_nav_at = None;
                self.ensure_sg_data();
            }
        }

        // Auto-refresh the current stop if its cache entry is stale.
        // Skip when the terminal is in the background — no point making
        // network requests that the user cannot see.
        if self.focused {
            // NUS auto-refresh
            if self.mode == crate::models::AppMode::NusCampus {
                if let Some(stop) = self.current_stop() {
                    let name = stop.name.clone();
                    let stale = self
                        .fetch
                        .cache
                        .get(&name)
                        .map(|c| {
                            c.fetched_at.elapsed()
                                >= Duration::from_secs(self.settings.auto_refresh_secs)
                        })
                        .unwrap_or(false);
                    if stale && !self.fetch.loading.contains(&name) {
                        self.start_fetch(name);
                    }
                }
            }

            // Train alert polling (SG mode, every 120 s)
            #[cfg(not(target_arch = "wasm32"))]
            if self.mode == crate::models::AppMode::SgPublicBus {
                let needs_fetch = self
                    .train_alert
                    .last_fetched
                    .map(|t| t.elapsed() >= Duration::from_secs(120))
                    .unwrap_or(true);
                if needs_fetch && !self.train_alert.fetching {
                    self.start_train_alert_fetch();
                }
            }

            // SG auto-refresh
            #[cfg(not(target_arch = "wasm32"))]
            if self.mode == crate::models::AppMode::SgPublicBus {
                if let Some(stop) = self.current_sg_stop() {
                    let code = stop.code.clone();
                    let stale = self
                        .sg_fetch
                        .cache
                        .get(&code)
                        .map(|c| {
                            c.fetched_at.elapsed()
                                >= Duration::from_secs(self.settings.auto_refresh_secs.max(20))
                        })
                        .unwrap_or(false);
                    if stale && !self.sg_fetch.loading.contains(&code) {
                        self.start_sg_fetch(code);
                    }
                }
            }
        }
    }
}
