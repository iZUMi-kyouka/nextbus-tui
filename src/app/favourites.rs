use std::time::Instant;

use super::App;

impl App {
    pub fn toggle_favourite(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if self.favourites.contains(&name) {
                self.favourites.remove(&name);
                let msg = self.i18n.t("status-fav-removed");
                self.set_status(&msg);
            } else {
                self.favourites.insert(name);
                let msg = self.i18n.t("status-fav-added");
                self.set_status(&msg);
            }
            crate::config::save(&self.config_snapshot());
            self.rebuild_list();
        }
    }

    /// Set a transient status message, replacing any existing one.
    pub fn set_status(&mut self, msg: &str) {
        self.status_msg = Some((msg.to_string(), Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use std::sync::mpsc;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        let mut app = App::new(tx);
        app.favourites.clear();
        app.fav_view = false;
        app.i18n = crate::i18n::I18n::new("en");
        app.rebuild_list();
        app
    }

    #[test]
    fn toggle_favourite_adds_stop() {
        let mut app = make_app();
        let name = app.current_stop().unwrap().name.clone();
        assert!(!app.favourites.contains(&name));
        app.toggle_favourite();
        assert!(app.favourites.contains(&name));
    }

    #[test]
    fn toggle_favourite_removes_stop() {
        let mut app = make_app();
        let name = app.current_stop().unwrap().name.clone();
        app.favourites.insert(name.clone());
        app.rebuild_list();
        app.toggle_favourite();
        assert!(!app.favourites.contains(&name));
    }

    #[test]
    fn toggle_favourite_sets_status_message() {
        let mut app = make_app();
        assert!(app.status_msg.is_none());
        app.toggle_favourite();
        assert!(app.status_msg.is_some());
    }

    #[test]
    fn set_status_stores_message() {
        let mut app = make_app();
        app.set_status("hello test");
        let (msg, _) = app.status_msg.as_ref().unwrap();
        assert_eq!(msg, "hello test");
    }

    #[test]
    fn set_status_replaces_previous() {
        let mut app = make_app();
        app.set_status("first");
        app.set_status("second");
        let (msg, _) = app.status_msg.as_ref().unwrap();
        assert_eq!(msg, "second");
    }
}
