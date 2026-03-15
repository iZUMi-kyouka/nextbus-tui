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

        // Commit a pending single-digit jump after 1 s with no second digit.
        if !self.nav.jump_buf.is_empty()
            && self
                .nav
                .jump_at
                .map(|t| t.elapsed() >= Duration::from_secs(1))
                .unwrap_or(false)
        {
            self.commit_jump();
        }

        // Auto-refresh the current stop if its cache entry is stale.
        // Skip when the terminal is in the background — no point making
        // network requests that the user cannot see.
        if self.focused {
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
    }
}
