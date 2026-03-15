use std::time::Duration;

use super::App;

impl App {
    /// Called every 500 ms by the background tick thread.
    /// Handles status message expiry, jump timeout, and auto-refresh.
    pub fn handle_tick(&mut self) {
        // Expire transient status messages after 3 s.
        if let Some((_, at)) = &self.status_msg
            && at.elapsed() > Duration::from_secs(3)
        {
            self.status_msg = None;
        }

        // Commit a pending single-digit jump after 1 s with no second digit.
        if !self.jump_buf.is_empty()
            && self
                .jump_at
                .map(|t| t.elapsed() >= Duration::from_secs(1))
                .unwrap_or(false)
        {
            self.commit_jump();
        }

        // Auto-refresh the current stop if its cache entry is stale.
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            let stale = self
                .cache
                .get(&name)
                .map(|c| c.fetched_at.elapsed() >= Duration::from_secs(self.auto_refresh_secs))
                .unwrap_or(false);
            if stale && !self.loading.contains(&name) {
                self.start_fetch(name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::app::CachedData;
    use crate::models::ShuttleServiceResult;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        let mut app = App::new(tx);
        app.favourites.clear();
        app.fav_view = false;
        app.auto_refresh_secs = 30; // reset to known value; config may differ
        app.i18n = crate::i18n::I18n::new("en");
        app.rebuild_list();
        app
    }

    fn old_instant(secs: u64) -> Instant {
        Instant::now()
            .checked_sub(Duration::from_secs(secs))
            .unwrap_or_else(Instant::now)
    }

    #[test]
    fn status_msg_expires_after_3s() {
        let mut app = make_app();
        app.status_msg = Some(("old".to_string(), old_instant(4)));
        app.handle_tick();
        assert!(app.status_msg.is_none());
    }

    #[test]
    fn fresh_status_msg_is_retained() {
        let mut app = make_app();
        app.status_msg = Some(("fresh".to_string(), Instant::now()));
        app.handle_tick();
        assert!(app.status_msg.is_some());
    }

    #[test]
    fn jump_committed_after_1s_timeout() {
        let mut app = make_app();
        app.jump_buf = "5".to_string();
        app.jump_at = Some(old_instant(2));
        app.handle_tick();
        assert!(app.jump_buf.is_empty());
        assert!(app.jump_at.is_none());
        assert_eq!(app.selected, 4); // position 5 → index 4
    }

    #[test]
    fn jump_not_committed_if_under_1s() {
        let mut app = make_app();
        app.jump_buf = "5".to_string();
        app.jump_at = Some(Instant::now());
        app.handle_tick();
        assert_eq!(app.jump_buf, "5");
    }

    #[test]
    fn tick_starts_refresh_for_stale_stop() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.cache.insert(
            stop.clone(),
            CachedData {
                result: ShuttleServiceResult {
                    name: Some(stop.clone()),
                    caption: None,
                    shuttles: vec![],
                    timestamp: None,
                },
                fetched_at: old_instant(31),
                error: None,
            },
        );
        app.handle_tick();
        assert!(
            app.loading.contains(&stop),
            "stale cache should trigger a refresh fetch"
        );
    }

    #[test]
    fn tick_does_not_refresh_fresh_stop() {
        let mut app = make_app();
        let stop = app.current_stop().unwrap().name.clone();
        app.cache.insert(
            stop.clone(),
            CachedData {
                result: ShuttleServiceResult {
                    name: Some(stop.clone()),
                    caption: None,
                    shuttles: vec![],
                    timestamp: None,
                },
                fetched_at: Instant::now(),
                error: None,
            },
        );
        app.handle_tick();
        assert!(!app.loading.contains(&stop));
    }
}
