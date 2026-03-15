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
