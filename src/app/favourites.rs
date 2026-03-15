use std::time::Instant;

use super::App;

impl App {
    pub fn toggle_favourite(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if self.favourites.contains(&name) {
                self.favourites.remove(&name);
                self.set_status("Removed from favourites");
            } else {
                self.favourites.insert(name);
                self.set_status("Added to favourites \u{2605}");
            }
            crate::config::save(&self.favourites);
            self.rebuild_list();
        }
    }

    /// Set a transient status message, replacing any existing one.
    pub fn set_status(&mut self, msg: &str) {
        self.status_msg = Some((msg.to_string(), Instant::now()));
    }
}
