use std::time::Duration;

use super::App;

impl App {
    /// Called every 500 ms by the background tick thread.
    /// Handles status message expiry, jump timeout, and auto-refresh.
    pub fn handle_tick(&mut self) {
        // Expire transient status messages after 3 s.
        if let Some((_, at)) = &self.status_msg {
            if at.elapsed() > Duration::from_secs(3) {
                self.status_msg = None;
            }
        }

        // Commit a pending single-digit jump after 1 s with no second digit.
        if !self.jump_buf.is_empty() {
            if self.jump_at.map(|t| t.elapsed() >= Duration::from_secs(1)).unwrap_or(false) {
                self.commit_jump();
            }
        }

        // Auto-refresh the current stop if its cache entry is stale.
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            let stale = self
                .cache
                .get(&name)
                .map(|c| c.fetched_at.elapsed() >= Duration::from_secs(super::AUTO_REFRESH_SECS))
                .unwrap_or(false);
            if stale && !self.loading.contains(&name) {
                self.start_fetch(name);
            }
        }
    }
}
