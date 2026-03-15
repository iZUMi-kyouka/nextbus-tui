use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::App;

impl App {
    /// Dispatch a keyboard event to the appropriate mode handler.
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.showing_theme_picker {
            self.handle_theme_picker_key(key);
        } else if self.searching {
            self.handle_search_key(key);
        } else {
            self.handle_normal_key(key);
        }
    }

    // ── Search / filter mode ───────────────────────────────────────────────────

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.searching = false;
                self.search_query.clear();
                self.rebuild_list();
                self.ensure_data();
            }
            KeyCode::Enter => {
                self.searching = false;
                self.ensure_data();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.rebuild_list();
                self.ensure_data();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.rebuild_list();
                self.ensure_data();
            }
            // Allow navigation while the overlay is open.
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            _ => {}
        }
    }

    // ── Theme picker mode ─────────────────────────────────────────────────────

    fn handle_theme_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('X') => self.showing_theme_picker = false,
            KeyCode::Enter => {
                self.theme_idx = self.theme_picker_cursor;
                self.showing_theme_picker = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.theme_picker_cursor > 0 {
                    self.theme_picker_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.theme_picker_cursor + 1 < self.themes.len() {
                    self.theme_picker_cursor += 1;
                }
            }
            _ => {}
        }
    }

    // ── Normal mode ────────────────────────────────────────────────────────────

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.cancel_jump();
                self.should_quit = true;
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                self.cancel_jump();
                self.move_up();
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                self.cancel_jump();
                self.move_down();
            }
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.cancel_jump();
                self.go_first();
            }
            (KeyCode::Char('G'), _) => {
                self.cancel_jump();
                self.go_last();
            }
            (KeyCode::Home, _) => {
                self.cancel_jump();
                self.go_first();
            }
            (KeyCode::End, _) => {
                self.cancel_jump();
                self.go_last();
            }
            (KeyCode::Char('f'), _) => {
                self.cancel_jump();
                self.toggle_favourite();
            }
            (KeyCode::Char('F'), _) => {
                self.cancel_jump();
                self.fav_view = !self.fav_view;
                self.search_query.clear();
                self.rebuild_list();
                self.ensure_data();
            }
            (KeyCode::Char('r'), _) => {
                self.cancel_jump();
                self.refresh_current();
            }
            (KeyCode::Char('/'), _) => {
                self.cancel_jump();
                self.search_query.clear();
                self.rebuild_list();
                self.searching = true;
            }
            (KeyCode::Char('x'), _) => {
                self.cancel_jump();
                self.theme_idx = (self.theme_idx + 1) % self.themes.len();
            }
            (KeyCode::Char('X'), _) => {
                self.cancel_jump();
                self.theme_picker_cursor = self.theme_idx;
                self.showing_theme_picker = true;
            }
            (KeyCode::Char(c), _) if c.is_ascii_digit() => self.push_jump_digit(c),
            (KeyCode::Enter, _) if !self.jump_buf.is_empty() => self.commit_jump(),
            _ => self.cancel_jump(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::mpsc;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        let mut app = App::new(tx);
        app.favourites.clear();
        app.rebuild_list();
        app
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    // ── Normal mode ────────────────────────────────────────────────────────────

    #[test]
    fn q_sets_should_quit() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn ctrl_c_sets_should_quit() {
        let mut app = make_app();
        app.handle_key(ctrl(KeyCode::Char('c')));
        assert!(app.should_quit);
    }

    #[test]
    fn down_moves_selection() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn j_moves_selection_down() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn up_at_top_stays() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn k_at_top_stays() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn g_goes_to_first() {
        let mut app = make_app();
        app.selected = 15;
        app.list_state.select(Some(15));
        app.handle_key(key(KeyCode::Char('g')));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn shift_g_goes_to_last() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('G')));
        assert_eq!(app.selected, app.sorted_indices.len() - 1);
    }

    #[test]
    fn f_toggles_favourite() {
        let mut app = make_app();
        let name = app.current_stop().unwrap().name.clone();
        app.handle_key(key(KeyCode::Char('f')));
        assert!(app.favourites.contains(&name));
    }

    #[test]
    fn shift_f_enables_fav_view() {
        let mut app = make_app();
        assert!(!app.fav_view);
        app.handle_key(key(KeyCode::Char('F')));
        assert!(app.fav_view);
    }

    #[test]
    fn shift_f_twice_toggles_back() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('F')));
        app.handle_key(key(KeyCode::Char('F')));
        assert!(!app.fav_view);
    }

    #[test]
    fn shift_f_clears_search_query() {
        let mut app = make_app();
        app.search_query = "COM".to_string();
        app.handle_key(key(KeyCode::Char('F')));
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn slash_opens_search_with_empty_query() {
        let mut app = make_app();
        app.search_query = "old".to_string();
        app.handle_key(key(KeyCode::Char('/')));
        assert!(app.searching);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn x_cycles_theme() {
        let mut app = make_app();
        let initial = app.theme_idx;
        app.handle_key(key(KeyCode::Char('x')));
        assert_eq!(app.theme_idx, (initial + 1) % app.themes.len());
    }

    #[test]
    fn x_wraps_theme_at_end() {
        let mut app = make_app();
        app.theme_idx = app.themes.len() - 1;
        app.handle_key(key(KeyCode::Char('x')));
        assert_eq!(app.theme_idx, 0);
    }

    #[test]
    fn shift_x_opens_theme_picker() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('X')));
        assert!(app.showing_theme_picker);
    }

    #[test]
    fn shift_x_preselects_current_theme() {
        let mut app = make_app();
        app.theme_idx = 3;
        app.handle_key(key(KeyCode::Char('X')));
        assert_eq!(app.theme_picker_cursor, 3);
    }

    #[test]
    fn digit_buffers_jump_in_large_list() {
        let mut app = make_app();
        app.handle_key(key(KeyCode::Char('5')));
        assert_eq!(app.jump_buf, "5");
    }

    // ── Search mode ────────────────────────────────────────────────────────────

    #[test]
    fn search_char_appends_to_query() {
        let mut app = make_app();
        app.searching = true;
        app.handle_key(key(KeyCode::Char('C')));
        assert_eq!(app.search_query, "C");
    }

    #[test]
    fn search_backspace_removes_char() {
        let mut app = make_app();
        app.searching = true;
        app.search_query = "COM".to_string();
        app.rebuild_list();
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.search_query, "CO");
    }

    #[test]
    fn search_esc_closes_and_clears() {
        let mut app = make_app();
        app.searching = true;
        app.search_query = "COM".to_string();
        app.handle_key(key(KeyCode::Esc));
        assert!(!app.searching);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn search_enter_closes_keeps_filter() {
        let mut app = make_app();
        app.searching = true;
        app.search_query = "COM".to_string();
        app.rebuild_list();
        let n = app.sorted_indices.len();
        app.handle_key(key(KeyCode::Enter));
        assert!(!app.searching);
        assert_eq!(app.search_query, "COM");
        assert_eq!(app.sorted_indices.len(), n);
    }

    // ── Theme picker mode ─────────────────────────────────────────────────────

    #[test]
    fn theme_picker_esc_closes() {
        let mut app = make_app();
        app.showing_theme_picker = true;
        app.handle_key(key(KeyCode::Esc));
        assert!(!app.showing_theme_picker);
    }

    #[test]
    fn theme_picker_shift_x_closes() {
        let mut app = make_app();
        app.showing_theme_picker = true;
        app.handle_key(key(KeyCode::Char('X')));
        assert!(!app.showing_theme_picker);
    }

    #[test]
    fn theme_picker_enter_applies_and_closes() {
        let mut app = make_app();
        app.showing_theme_picker = true;
        app.theme_picker_cursor = 2;
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.theme_idx, 2);
        assert!(!app.showing_theme_picker);
    }

    #[test]
    fn theme_picker_j_moves_cursor_down() {
        let mut app = make_app();
        app.showing_theme_picker = true;
        app.theme_picker_cursor = 0;
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.theme_picker_cursor, 1);
    }

    #[test]
    fn theme_picker_k_moves_cursor_up() {
        let mut app = make_app();
        app.showing_theme_picker = true;
        app.theme_picker_cursor = 3;
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.theme_picker_cursor, 2);
    }

    #[test]
    fn theme_picker_k_at_top_stays() {
        let mut app = make_app();
        app.showing_theme_picker = true;
        app.theme_picker_cursor = 0;
        app.handle_key(key(KeyCode::Char('k')));
        assert_eq!(app.theme_picker_cursor, 0);
    }

    #[test]
    fn theme_picker_j_at_bottom_stays() {
        let mut app = make_app();
        app.showing_theme_picker = true;
        app.theme_picker_cursor = app.themes.len() - 1;
        app.handle_key(key(KeyCode::Char('j')));
        assert_eq!(app.theme_picker_cursor, app.themes.len() - 1);
    }
}
