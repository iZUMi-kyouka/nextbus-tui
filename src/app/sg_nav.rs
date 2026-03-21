use super::App;
use crate::models::SgBusStop;

impl App {
    /// Rebuild `sg_nav.sorted_indices` with fav-first sort and search filter.
    pub fn rebuild_sg_list(&mut self) {
        let q = self.sg_nav.search_query.to_lowercase();
        let stops = &self.domain.sg_stops;
        let favs = &self.settings.sg_favourites;

        let matches = |s: &SgBusStop| -> bool {
            q.is_empty()
                || s.code.to_lowercase().contains(&q)
                || s.road_name.to_lowercase().contains(&q)
                || s.description.to_lowercase().contains(&q)
        };

        if self.sg_nav.fav_view {
            let mut fav_idx: Vec<usize> = stops
                .iter()
                .enumerate()
                .filter(|(_, s)| favs.contains(&s.code) && matches(s))
                .map(|(i, _)| i)
                .collect();
            fav_idx.sort_by(|&a, &b| stops[a].description.cmp(&stops[b].description));
            self.sg_nav.sorted_indices = fav_idx;
        } else {
            let mut fav_idx: Vec<usize> = stops
                .iter()
                .enumerate()
                .filter(|(_, s)| favs.contains(&s.code) && matches(s))
                .map(|(i, _)| i)
                .collect();
            fav_idx.sort_by(|&a, &b| stops[a].description.cmp(&stops[b].description));

            let mut rest: Vec<usize> = stops
                .iter()
                .enumerate()
                .filter(|(_, s)| !favs.contains(&s.code) && matches(s))
                .map(|(i, _)| i)
                .collect();
            rest.sort_by(|&a, &b| stops[a].description.cmp(&stops[b].description));

            fav_idx.extend(rest);
            self.sg_nav.sorted_indices = fav_idx;
        }

        self.sg_nav.selected = if self.sg_nav.sorted_indices.is_empty() {
            0
        } else {
            self.sg_nav
                .selected
                .min(self.sg_nav.sorted_indices.len() - 1)
        };
        self.sg_nav.list_state.select(Some(self.sg_nav.selected));
    }

    /// The currently highlighted SG bus stop.
    pub fn current_sg_stop(&self) -> Option<&SgBusStop> {
        self.sg_nav
            .sorted_indices
            .get(self.sg_nav.selected)
            .map(|&i| &self.domain.sg_stops[i])
    }

    pub fn sg_move_up(&mut self) {
        if self.sg_nav.selected > 0 {
            self.sg_nav.selected -= 1;
            self.sg_nav.list_state.select(Some(self.sg_nav.selected));
            #[cfg(not(target_arch = "wasm32"))]
            self.ensure_sg_data();
        }
    }

    pub fn sg_move_down(&mut self) {
        if self.sg_nav.selected + 1 < self.sg_nav.sorted_indices.len() {
            self.sg_nav.selected += 1;
            self.sg_nav.list_state.select(Some(self.sg_nav.selected));
            #[cfg(not(target_arch = "wasm32"))]
            self.ensure_sg_data();
        }
    }

    pub fn sg_go_first(&mut self) {
        if !self.sg_nav.sorted_indices.is_empty() {
            self.sg_nav.selected = 0;
            self.sg_nav.list_state.select(Some(0));
            #[cfg(not(target_arch = "wasm32"))]
            self.ensure_sg_data();
        }
    }

    pub fn sg_go_last(&mut self) {
        if !self.sg_nav.sorted_indices.is_empty() {
            self.sg_nav.selected = self.sg_nav.sorted_indices.len() - 1;
            self.sg_nav.list_state.select(Some(self.sg_nav.selected));
            #[cfg(not(target_arch = "wasm32"))]
            self.ensure_sg_data();
        }
    }

    pub fn sg_scroll_up(&mut self) {
        let off = self.sg_nav.list_state.offset();
        let new_off = off.saturating_sub(3);
        *self.sg_nav.list_state.offset_mut() = new_off;
        self.sg_nav.list_state.select(Some(self.sg_nav.selected));
        let h = self.sg_nav.list_height as usize;
        if h > 0 && self.sg_nav.selected >= new_off + h {
            let last_visible =
                (new_off + h - 1).min(self.sg_nav.sorted_indices.len().saturating_sub(1));
            self.sg_nav.selected = last_visible;
            self.sg_nav.list_state.select(Some(last_visible));
            #[cfg(not(target_arch = "wasm32"))]
            self.ensure_sg_data();
        }
    }

    pub fn sg_scroll_down(&mut self) {
        let len = self.sg_nav.sorted_indices.len();
        let off = self.sg_nav.list_state.offset();
        let new_off = (off + 3).min(len.saturating_sub(1));
        *self.sg_nav.list_state.offset_mut() = new_off;
        if self.sg_nav.selected < new_off {
            self.sg_nav.selected = new_off;
            self.sg_nav.list_state.select(Some(new_off));
            #[cfg(not(target_arch = "wasm32"))]
            self.ensure_sg_data();
        } else {
            self.sg_nav.list_state.select(Some(self.sg_nav.selected));
        }
    }

    pub fn toggle_sg_favourite(&mut self) {
        if let Some(stop) = self.current_sg_stop() {
            let code = stop.code.clone();
            if self.settings.sg_favourites.contains(&code) {
                self.settings.sg_favourites.remove(&code);
                let msg = self.i18n.t("status-fav-removed");
                self.set_status(&msg);
            } else {
                self.settings.sg_favourites.insert(code);
                let msg = self.i18n.t("status-fav-added");
                self.set_status(&msg);
            }
            self.settings.persist(&self.i18n.lang);
            self.rebuild_sg_list();
        }
    }

    pub fn sg_fav_count_in_list(&self) -> usize {
        self.sg_nav
            .sorted_indices
            .iter()
            .filter(|&&i| {
                self.settings
                    .sg_favourites
                    .contains(&self.domain.sg_stops[i].code)
            })
            .count()
    }

    /// Push a jump digit for SG navigation.
    pub fn sg_push_jump_digit(&mut self, c: char) {
        // Reuse NUS jump logic if list < 10 items
        if self.sg_nav.sorted_indices.len() < 10 {
            let n: usize = (c as u8 - b'0') as usize;
            let target = if n == 0 { 9 } else { n - 1 };
            if target < self.sg_nav.sorted_indices.len() {
                self.sg_nav.selected = target;
                self.sg_nav.list_state.select(Some(target));
                #[cfg(not(target_arch = "wasm32"))]
                self.ensure_sg_data();
            }
            return;
        }
        self.sg_nav.jump_buf.push(c);
        self.sg_nav.jump_at = Some(crate::time::Instant::now());
        // Auto-commit two-digit jumps
        if self.sg_nav.jump_buf.len() >= 2 {
            self.sg_commit_jump();
        }
    }

    pub fn sg_commit_jump(&mut self) {
        if let Ok(n) = self.sg_nav.jump_buf.parse::<usize>() {
            let target = if n == 0 {
                self.sg_nav.sorted_indices.len().saturating_sub(1)
            } else {
                n.saturating_sub(1)
            };
            let target = target.min(self.sg_nav.sorted_indices.len().saturating_sub(1));
            self.sg_nav.selected = target;
            self.sg_nav.list_state.select(Some(target));
            #[cfg(not(target_arch = "wasm32"))]
            self.ensure_sg_data();
        }
        self.sg_nav.jump_buf.clear();
        self.sg_nav.jump_at = None;
    }

    pub fn sg_cancel_jump(&mut self) {
        self.sg_nav.jump_buf.clear();
        self.sg_nav.jump_at = None;
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
    fn rebuild_sg_list_empty_when_no_stops() {
        let app = make_app();
        assert_eq!(app.sg_nav.sorted_indices.len(), 0);
    }

    #[test]
    fn sg_fav_count_no_favourites() {
        let app = make_app();
        assert_eq!(app.sg_fav_count_in_list(), 0);
    }
}
