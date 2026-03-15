use std::time::Instant;

use super::App;

impl App {
    pub fn toggle_favourite(&mut self) {
        if let Some(stop) = self.current_stop() {
            let name = stop.name.clone();
            if self.settings.favourites.contains(&name) {
                self.settings.favourites.remove(&name);
                let msg = self.i18n.t("status-fav-removed");
                self.set_status(&msg);
            } else {
                self.settings.favourites.insert(name);
                let msg = self.i18n.t("status-fav-added");
                self.set_status(&msg);
            }
            self.settings.persist(&self.i18n.lang);
            self.rebuild_list();
        }
    }

    /// Set a transient status message, replacing any existing one.
    pub fn set_status(&mut self, msg: &str) {
        self.overlay.status_msg = Some((msg.to_string(), Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use std::sync::mpsc;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        App::new_test(tx)
    }

    #[test]
    fn toggle_favourite_adds_stop() {
        let mut app = make_app();
        let name = app.current_stop().unwrap().name.clone();
        assert!(!app.settings.favourites.contains(&name));
        app.toggle_favourite();
        assert!(app.settings.favourites.contains(&name));
    }

    #[test]
    fn toggle_favourite_removes_stop() {
        let mut app = make_app();
        let name = app.current_stop().unwrap().name.clone();
        app.settings.favourites.insert(name.clone());
        app.rebuild_list();
        app.toggle_favourite();
        assert!(!app.settings.favourites.contains(&name));
    }

    #[test]
    fn toggle_favourite_sets_status_message() {
        let mut app = make_app();
        assert!(app.overlay.status_msg.is_none());
        app.toggle_favourite();
        assert!(app.overlay.status_msg.is_some());
    }

    #[test]
    fn set_status_stores_message() {
        let mut app = make_app();
        app.set_status("hello test");
        let (msg, _) = app.overlay.status_msg.as_ref().unwrap();
        assert_eq!(msg, "hello test");
    }

    #[test]
    fn set_status_replaces_previous() {
        let mut app = make_app();
        app.set_status("first");
        app.set_status("second");
        let (msg, _) = app.overlay.status_msg.as_ref().unwrap();
        assert_eq!(msg, "second");
    }
}
