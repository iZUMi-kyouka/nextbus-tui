use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::App;

impl App {
    /// Dispatch a keyboard event to the appropriate mode handler.
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.searching {
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
                self.rebuild_list();
                self.ensure_data();
            }
            (KeyCode::Char('r'), _) => {
                self.cancel_jump();
                self.refresh_current();
            }
            (KeyCode::Char('/'), _) => {
                self.cancel_jump();
                self.searching = true;
            }
            (KeyCode::Char(c), _) if c.is_ascii_digit() => self.push_jump_digit(c),
            (KeyCode::Enter, _) if !self.jump_buf.is_empty() => self.commit_jump(),
            _ => self.cancel_jump(),
        }
    }
}
