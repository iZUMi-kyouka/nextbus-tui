use crate::models::BusStop;

use super::App;

impl App {
    /// Rebuild `sorted_indices` applying the current search filter.
    /// Favourites come first (alphabetical within each group).
    pub fn rebuild_list(&mut self) {
        let q = self.search_query.to_lowercase();
        let matches = |s: &BusStop| -> bool {
            q.is_empty()
                || s.caption.to_lowercase().contains(&q)
                || s.name.to_lowercase().contains(&q)
        };

        if self.fav_view {
            let mut favs: Vec<usize> = self
                .stops
                .iter()
                .enumerate()
                .filter(|(_, s)| self.favourites.contains(&s.name) && matches(s))
                .map(|(i, _)| i)
                .collect();
            favs.sort_by(|&a, &b| self.stops[a].caption.cmp(&self.stops[b].caption));
            self.sorted_indices = favs;
        } else {
            let mut favs: Vec<usize> = self
                .stops
                .iter()
                .enumerate()
                .filter(|(_, s)| self.favourites.contains(&s.name) && matches(s))
                .map(|(i, _)| i)
                .collect();
            favs.sort_by(|&a, &b| self.stops[a].caption.cmp(&self.stops[b].caption));

            let mut rest: Vec<usize> = self
                .stops
                .iter()
                .enumerate()
                .filter(|(_, s)| !self.favourites.contains(&s.name) && matches(s))
                .map(|(i, _)| i)
                .collect();
            rest.sort_by(|&a, &b| self.stops[a].caption.cmp(&self.stops[b].caption));

            favs.extend(rest);
            self.sorted_indices = favs;
        }

        // Clamp selection so it stays in bounds after filtering.
        self.selected = if self.sorted_indices.is_empty() {
            0
        } else {
            self.selected.min(self.sorted_indices.len() - 1)
        };
        self.list_state.select(Some(self.selected));
    }

    /// The currently highlighted bus stop, if any.
    pub fn current_stop(&self) -> Option<&BusStop> {
        self.sorted_indices.get(self.selected).map(|&i| &self.stops[i])
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
            self.ensure_data();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.sorted_indices.len() {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
            self.ensure_data();
        }
    }

    pub fn go_first(&mut self) {
        if !self.sorted_indices.is_empty() {
            self.selected = 0;
            self.list_state.select(Some(0));
            self.ensure_data();
        }
    }

    pub fn go_last(&mut self) {
        if !self.sorted_indices.is_empty() {
            self.selected = self.sorted_indices.len() - 1;
            self.list_state.select(Some(self.selected));
            self.ensure_data();
        }
    }

    // ── UI query helpers ───────────────────────────────────────────────────────

    /// Seconds until the auto-refresh fires for the current stop, or `None` if loading.
    pub fn seconds_until_refresh(&self) -> Option<u64> {
        let name = self.current_stop()?.name.clone();
        if self.loading.contains(&name) {
            return None;
        }
        let elapsed = self.cache.get(&name)?.fetched_at.elapsed().as_secs();
        Some(super::AUTO_REFRESH_SECS.saturating_sub(elapsed))
    }

    /// How many visible stops are favourites (used by the list renderer for the separator).
    pub fn fav_count_in_list(&self) -> usize {
        self.sorted_indices
            .iter()
            .filter(|&&i| self.favourites.contains(&self.stops[i].name))
            .count()
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
        app.rebuild_list();
        app
    }

    #[test]
    fn rebuild_list_all_stops_in_normal_mode() {
        let app = make_app();
        assert_eq!(app.sorted_indices.len(), 33);
    }

    #[test]
    fn rebuild_list_fav_view_empty_when_no_favs() {
        let mut app = make_app();
        app.fav_view = true;
        app.rebuild_list();
        assert!(app.sorted_indices.is_empty());
    }

    #[test]
    fn rebuild_list_fav_view_shows_only_favourites() {
        let mut app = make_app();
        let name = app.stops[0].name.clone();
        app.favourites.insert(name.clone());
        app.fav_view = true;
        app.rebuild_list();
        assert_eq!(app.sorted_indices.len(), 1);
        assert_eq!(app.stops[app.sorted_indices[0]].name, name);
    }

    #[test]
    fn rebuild_list_search_filter_reduces_results() {
        let mut app = make_app();
        app.search_query = "COM".to_string();
        app.rebuild_list();
        assert!(app.sorted_indices.len() < 33);
        for &i in &app.sorted_indices {
            let s = &app.stops[i];
            assert!(
                s.caption.to_lowercase().contains("com") || s.name.to_lowercase().contains("com"),
                "stop {} doesn't match filter 'COM'",
                s.name
            );
        }
    }

    #[test]
    fn rebuild_list_search_case_insensitive() {
        let make = || {
            let (tx, _rx) = mpsc::channel();
            let mut a = App::new(tx);
            a.favourites.clear();
            a
        };
        let mut a1 = make();
        a1.search_query = "com".to_string();
        a1.rebuild_list();
        let mut a2 = make();
        a2.search_query = "COM".to_string();
        a2.rebuild_list();
        assert_eq!(a1.sorted_indices, a2.sorted_indices);
    }

    #[test]
    fn rebuild_list_clamps_selection_after_narrow_filter() {
        let mut app = make_app();
        app.selected = 30;
        app.search_query = "COM3".to_string();
        app.rebuild_list();
        let len = app.sorted_indices.len();
        assert!(app.selected < len.max(1));
    }

    #[test]
    fn move_up_at_top_is_no_op() {
        let mut app = make_app();
        assert_eq!(app.selected, 0);
        app.move_up();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn move_down_increments_selection() {
        let mut app = make_app();
        app.move_down();
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn move_down_at_bottom_is_no_op() {
        let mut app = make_app();
        let last = app.sorted_indices.len() - 1;
        app.selected = last;
        app.list_state.select(Some(last));
        app.move_down();
        assert_eq!(app.selected, last);
    }

    #[test]
    fn go_first_sets_selection_to_zero() {
        let mut app = make_app();
        app.selected = 10;
        app.go_first();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn go_last_sets_selection_to_end() {
        let mut app = make_app();
        let expected = app.sorted_indices.len() - 1;
        app.go_last();
        assert_eq!(app.selected, expected);
    }

    #[test]
    fn current_stop_returns_some() {
        let app = make_app();
        assert!(app.current_stop().is_some());
    }

    #[test]
    fn fav_count_no_favourites() {
        let app = make_app();
        assert_eq!(app.fav_count_in_list(), 0);
    }

    #[test]
    fn fav_count_with_one_favourite() {
        let mut app = make_app();
        let name = app.stops[0].name.clone();
        app.favourites.insert(name);
        app.rebuild_list();
        assert_eq!(app.fav_count_in_list(), 1);
    }

    #[test]
    fn favourites_sorted_first_in_normal_view() {
        let mut app = make_app();
        let last_idx = *app.sorted_indices.last().unwrap();
        let fav_name = app.stops[last_idx].name.clone();
        app.favourites.insert(fav_name.clone());
        app.rebuild_list();
        let first_stop = &app.stops[app.sorted_indices[0]];
        assert_eq!(first_stop.name, fav_name);
    }
}
