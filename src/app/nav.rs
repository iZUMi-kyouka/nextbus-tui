use crate::models::BusStop;
use crate::time::Instant;

use super::App;

impl App {
    /// Rebuild `sorted_indices` applying the current search filter.
    /// Favourites come first (alphabetical within each group).
    pub fn rebuild_list(&mut self) {
        let q = self.nav.search_query.to_lowercase();
        let matches = |s: &BusStop| -> bool {
            q.is_empty()
                || s.caption.to_lowercase().contains(&q)
                || s.name.to_lowercase().contains(&q)
        };

        if self.nav.fav_view {
            let mut favs: Vec<usize> = self
                .domain
                .stops
                .iter()
                .enumerate()
                .filter(|(_, s)| self.settings.favourites.contains(&s.name) && matches(s))
                .map(|(i, _)| i)
                .collect();
            favs.sort_by(|&a, &b| {
                self.domain.stops[a]
                    .caption
                    .cmp(&self.domain.stops[b].caption)
            });
            self.nav.sorted_indices = favs;
        } else {
            let mut favs: Vec<usize> = self
                .domain
                .stops
                .iter()
                .enumerate()
                .filter(|(_, s)| self.settings.favourites.contains(&s.name) && matches(s))
                .map(|(i, _)| i)
                .collect();
            favs.sort_by(|&a, &b| {
                self.domain.stops[a]
                    .caption
                    .cmp(&self.domain.stops[b].caption)
            });

            let mut rest: Vec<usize> = self
                .domain
                .stops
                .iter()
                .enumerate()
                .filter(|(_, s)| !self.settings.favourites.contains(&s.name) && matches(s))
                .map(|(i, _)| i)
                .collect();
            rest.sort_by(|&a, &b| {
                self.domain.stops[a]
                    .caption
                    .cmp(&self.domain.stops[b].caption)
            });

            favs.extend(rest);
            self.nav.sorted_indices = favs;
        }

        // Clamp selection so it stays in bounds after filtering.
        self.nav.selected = if self.nav.sorted_indices.is_empty() {
            0
        } else {
            self.nav.selected.min(self.nav.sorted_indices.len() - 1)
        };
        self.update_nav_offset();
    }

    /// The currently highlighted bus stop, if any.
    pub fn current_stop(&self) -> Option<&BusStop> {
        self.nav
            .sorted_indices
            .get(self.nav.selected)
            .map(|&i| &self.domain.stops[i])
    }

    pub fn move_up(&mut self) {
        if self.nav.selected > 0 {
            self.nav.selected -= 1;
            self.update_nav_offset();
            self.on_nav_move();
        }
    }

    pub fn move_down(&mut self) {
        if self.nav.selected + 1 < self.nav.sorted_indices.len() {
            self.nav.selected += 1;
            self.update_nav_offset();
            self.on_nav_move();
        }
    }

    pub fn go_first(&mut self) {
        if !self.nav.sorted_indices.is_empty() {
            self.nav.selected = 0;
            self.update_nav_offset();
            self.on_nav_move();
        }
    }

    /// Scroll the viewport up by 3 rows without moving the selection.
    pub fn scroll_up(&mut self) {
        let off = self.nav.list_state.offset();
        *self.nav.list_state.offset_mut() = off.saturating_sub(3);
    }

    /// Scroll the viewport down by 3 rows without moving the selection.
    /// The offset is capped so the last item never scrolls above the viewport bottom.
    pub fn scroll_down(&mut self) {
        let len = self.nav.sorted_indices.len();
        let h = self.nav.list_height as usize;
        let max_off = len.saturating_sub(h.max(1));
        let off = self.nav.list_state.offset();
        *self.nav.list_state.offset_mut() = (off + 3).min(max_off);
    }

    pub fn go_last(&mut self) {
        if !self.nav.sorted_indices.is_empty() {
            self.nav.selected = self.nav.sorted_indices.len() - 1;
            self.update_nav_offset();
            self.on_nav_move();
        }
    }

    /// Leading-edge + trailing debounce for NUS navigation.
    ///
    /// Fires `ensure_data()` immediately when the user first moves (or resumes
    /// after a 300 ms pause), so single keypresses are instant. During rapid
    /// cycling, only `last_nav_at` is updated; the tick fires one final fetch
    /// after the cursor has been still for ≥ 300 ms.
    pub(super) fn on_nav_move(&mut self) {
        use std::time::Duration;
        let is_first_move = self
            .nav
            .last_nav_at
            .map(|t| t.elapsed() >= Duration::from_millis(300))
            .unwrap_or(true);
        self.nav.last_nav_at = Some(Instant::now());
        if is_first_move {
            self.ensure_data();
        }
    }

    /// Adjust the viewport offset so `nav.selected` is visible.
    pub(super) fn update_nav_offset(&mut self) {
        let h = self.nav.list_height as usize;
        if h == 0 {
            return;
        }
        let sel = self.nav.selected;
        let off = self.nav.list_state.offset();
        if sel < off {
            *self.nav.list_state.offset_mut() = sel;
        } else if sel >= off + h {
            *self.nav.list_state.offset_mut() = sel + 1 - h;
        }
    }

    // ── UI query helpers ───────────────────────────────────────────────────────

    /// Seconds until the auto-refresh fires for the current stop, or `None` if loading.
    pub fn seconds_until_refresh(&self) -> Option<u64> {
        let name = self.current_stop()?.name.clone();
        if self.fetch.loading.contains(&name) {
            return None;
        }
        let elapsed = self.fetch.cache.get(&name)?.fetched_at.elapsed().as_secs();
        Some(self.settings.auto_refresh_secs.saturating_sub(elapsed))
    }

    /// How many visible stops are favourites (used by the list renderer for the separator).
    pub fn fav_count_in_list(&self) -> usize {
        self.nav
            .sorted_indices
            .iter()
            .filter(|&&i| {
                self.settings
                    .favourites
                    .contains(&self.domain.stops[i].name)
            })
            .count()
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
    fn rebuild_list_all_stops_in_normal_mode() {
        let app = make_app();
        assert_eq!(app.nav.sorted_indices.len(), 33);
    }

    #[test]
    fn rebuild_list_fav_view_empty_when_no_favs() {
        let mut app = make_app();
        app.nav.fav_view = true;
        app.rebuild_list();
        assert!(app.nav.sorted_indices.is_empty());
    }

    #[test]
    fn rebuild_list_fav_view_shows_only_favourites() {
        let mut app = make_app();
        let name = app.domain.stops[0].name.clone();
        app.settings.favourites.insert(name.clone());
        app.nav.fav_view = true;
        app.rebuild_list();
        assert_eq!(app.nav.sorted_indices.len(), 1);
        assert_eq!(app.domain.stops[app.nav.sorted_indices[0]].name, name);
    }

    #[test]
    fn rebuild_list_search_filter_reduces_results() {
        let mut app = make_app();
        app.nav.search_query = "COM".to_string();
        app.rebuild_list();
        assert!(app.nav.sorted_indices.len() < 33);
        for &i in &app.nav.sorted_indices {
            let s = &app.domain.stops[i];
            assert!(
                s.caption.to_lowercase().contains("com") || s.name.to_lowercase().contains("com"),
                "stop {} doesn't match filter 'COM'",
                s.name
            );
        }
    }

    #[test]
    fn rebuild_list_search_case_insensitive() {
        let (tx1, _) = mpsc::channel();
        let (tx2, _) = mpsc::channel();
        let mut a1 = App::new_test(tx1);
        a1.nav.search_query = "com".to_string();
        a1.rebuild_list();
        let mut a2 = App::new_test(tx2);
        a2.nav.search_query = "COM".to_string();
        a2.rebuild_list();
        assert_eq!(a1.nav.sorted_indices, a2.nav.sorted_indices);
    }

    #[test]
    fn rebuild_list_clamps_selection_after_narrow_filter() {
        let mut app = make_app();
        app.nav.selected = 30;
        app.nav.search_query = "COM3".to_string();
        app.rebuild_list();
        let len = app.nav.sorted_indices.len();
        assert!(app.nav.selected < len.max(1));
    }

    #[test]
    fn move_up_at_top_is_no_op() {
        let mut app = make_app();
        assert_eq!(app.nav.selected, 0);
        app.move_up();
        assert_eq!(app.nav.selected, 0);
    }

    #[test]
    fn move_down_increments_selection() {
        let mut app = make_app();
        app.move_down();
        assert_eq!(app.nav.selected, 1);
    }

    #[test]
    fn move_down_at_bottom_is_no_op() {
        let mut app = make_app();
        let last = app.nav.sorted_indices.len() - 1;
        app.nav.selected = last;
        app.nav.list_state.select(Some(last));
        app.move_down();
        assert_eq!(app.nav.selected, last);
    }

    #[test]
    fn go_first_sets_selection_to_zero() {
        let mut app = make_app();
        app.nav.selected = 10;
        app.go_first();
        assert_eq!(app.nav.selected, 0);
    }

    #[test]
    fn go_last_sets_selection_to_end() {
        let mut app = make_app();
        let expected = app.nav.sorted_indices.len() - 1;
        app.go_last();
        assert_eq!(app.nav.selected, expected);
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
        let name = app.domain.stops[0].name.clone();
        app.settings.favourites.insert(name);
        app.rebuild_list();
        assert_eq!(app.fav_count_in_list(), 1);
    }

    #[test]
    fn favourites_sorted_first_in_normal_view() {
        let mut app = make_app();
        let last_idx = *app.nav.sorted_indices.last().unwrap();
        let fav_name = app.domain.stops[last_idx].name.clone();
        app.settings.favourites.insert(fav_name.clone());
        app.rebuild_list();
        let first_stop = &app.domain.stops[app.nav.sorted_indices[0]];
        assert_eq!(first_stop.name, fav_name);
    }
}
