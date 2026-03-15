use crossterm::event::{KeyCode, KeyEvent};
use fluent::FluentArgs;

use super::App;
use crate::theme::ThemeMode;

const REFRESH_MIN: u64 = 5;
const REFRESH_MAX: u64 = 300;

impl App {
    pub fn handle_settings_key(&mut self, key: KeyEvent) {
        if self.settings_edit_mode {
            self.handle_settings_edit_key(key);
        } else {
            self.handle_settings_nav_key(key);
        }
    }

    fn handle_settings_nav_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.showing_settings = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_cursor > 0 {
                    self.settings_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.settings_cursor + 1 < super::SETTINGS_ROW_COUNT {
                    self.settings_cursor += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_settings_row();
            }
            _ => {}
        }
    }

    fn activate_settings_row(&mut self) {
        match self.settings_cursor {
            0 => {
                // Enter edit mode for the auto-refresh interval.
                self.settings_edit_mode = true;
                self.settings_edit_buf = self.auto_refresh_secs.to_string();
            }
            1 => {
                // Toggle the default view and persist immediately.
                self.default_fav_view = !self.default_fav_view;
                let view_key = if self.default_fav_view {
                    "status-view-favs"
                } else {
                    "status-view-all"
                };
                let view_str = self.i18n.t(view_key);
                let mut args = FluentArgs::new();
                args.set("view", view_str.as_str());
                let msg = self.i18n.t_args("status-view-set", &args);
                self.set_status(&msg);
                crate::config::save(&self.config_snapshot());
            }
            2 => {
                // Cycle to the next language and reinitialise the i18n bundle.
                let next = self.i18n.next_lang().to_owned();
                self.i18n = crate::i18n::I18n::new(&next);
                let name = self.i18n.lang_meta.native_name.clone();
                let mut args = FluentArgs::new();
                args.set("name", name.as_str());
                let msg = self.i18n.t_args("status-lang-set", &args);
                self.set_status(&msg);
                crate::config::save(&self.config_snapshot());
            }
            3 => {
                // Cycle through theme modes: Dark → Light → Auto → Dark.
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Auto,
                    ThemeMode::Auto => ThemeMode::Dark,
                };
                let mode_key = match self.theme_mode {
                    ThemeMode::Dark => "settings-theme-mode-dark",
                    ThemeMode::Light => "settings-theme-mode-light",
                    ThemeMode::Auto => "settings-theme-mode-auto",
                };
                let mode_str = self.i18n.t(mode_key);
                let mut args = FluentArgs::new();
                args.set("mode", mode_str.as_str());
                let msg = self.i18n.t_args("status-theme-mode-set", &args);
                self.set_status(&msg);
                crate::config::save(&self.config_snapshot());
            }
            _ => {}
        }
    }

    fn handle_settings_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Cancel: discard the typed value.
                self.settings_edit_mode = false;
                self.settings_edit_buf.clear();
            }
            KeyCode::Enter => {
                self.commit_refresh_interval();
            }
            KeyCode::Backspace => {
                self.settings_edit_buf.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                // Cap at 3 digits (max value 300 fits in 3 digits).
                if self.settings_edit_buf.len() < 3 {
                    self.settings_edit_buf.push(c);
                }
            }
            _ => {}
        }
    }

    fn commit_refresh_interval(&mut self) {
        let parsed: u64 = self
            .settings_edit_buf
            .parse()
            .unwrap_or(self.auto_refresh_secs);
        self.auto_refresh_secs = parsed.clamp(REFRESH_MIN, REFRESH_MAX);
        self.settings_edit_mode = false;
        self.settings_edit_buf.clear();
        crate::config::save(&self.config_snapshot());
        let mut args = FluentArgs::new();
        args.set("seconds", self.auto_refresh_secs as i64);
        let msg = self.i18n.t_args("status-interval-set", &args);
        self.set_status(&msg);
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
        app.fav_view = false;
        app.default_fav_view = false;
        app.i18n = crate::i18n::I18n::new("en");
        app.rebuild_list();
        app
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn s_key_closes_settings() {
        let mut app = make_app();
        app.showing_settings = true;
        app.handle_settings_key(key(KeyCode::Char('s')));
        assert!(!app.showing_settings);
    }

    #[test]
    fn esc_closes_settings() {
        let mut app = make_app();
        app.showing_settings = true;
        app.handle_settings_key(key(KeyCode::Esc));
        assert!(!app.showing_settings);
    }

    #[test]
    fn j_moves_cursor_down() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = 0;
        app.handle_settings_key(key(KeyCode::Char('j')));
        assert_eq!(app.settings_cursor, 1);
    }

    #[test]
    fn k_moves_cursor_up() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = 2;
        app.handle_settings_key(key(KeyCode::Char('k')));
        assert_eq!(app.settings_cursor, 1);
    }

    #[test]
    fn j_at_bottom_stays() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = super::super::SETTINGS_ROW_COUNT - 1;
        app.handle_settings_key(key(KeyCode::Char('j')));
        assert_eq!(app.settings_cursor, super::super::SETTINGS_ROW_COUNT - 1);
    }

    #[test]
    fn k_at_top_stays() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = 0;
        app.handle_settings_key(key(KeyCode::Char('k')));
        assert_eq!(app.settings_cursor, 0);
    }

    #[test]
    fn enter_on_interval_row_enters_edit_mode() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = 0;
        app.handle_settings_key(key(KeyCode::Enter));
        assert!(app.settings_edit_mode);
        assert_eq!(app.settings_edit_buf, app.auto_refresh_secs.to_string());
    }

    #[test]
    fn enter_on_default_view_row_toggles() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = 1;
        let before = app.default_fav_view;
        app.handle_settings_key(key(KeyCode::Enter));
        assert_eq!(app.default_fav_view, !before);
    }

    #[test]
    fn edit_mode_digit_appends() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = 0;
        app.settings_edit_mode = true;
        app.settings_edit_buf = "1".to_string();
        app.handle_settings_key(key(KeyCode::Char('5')));
        assert_eq!(app.settings_edit_buf, "15");
    }

    #[test]
    fn edit_mode_backspace_removes() {
        let mut app = make_app();
        app.settings_edit_mode = true;
        app.settings_edit_buf = "30".to_string();
        app.handle_settings_key(key(KeyCode::Backspace));
        assert_eq!(app.settings_edit_buf, "3");
    }

    #[test]
    fn edit_mode_esc_cancels_without_saving() {
        let mut app = make_app();
        let original = app.auto_refresh_secs;
        app.settings_edit_mode = true;
        app.settings_edit_buf = "99".to_string();
        app.handle_settings_key(key(KeyCode::Esc));
        assert!(!app.settings_edit_mode);
        assert_eq!(app.auto_refresh_secs, original);
    }

    #[test]
    fn edit_mode_enter_commits_value() {
        let mut app = make_app();
        app.settings_edit_mode = true;
        app.settings_edit_buf = "60".to_string();
        app.handle_settings_key(key(KeyCode::Enter));
        assert!(!app.settings_edit_mode);
        assert_eq!(app.auto_refresh_secs, 60);
    }

    #[test]
    fn commit_clamps_below_minimum() {
        let mut app = make_app();
        app.settings_edit_mode = true;
        app.settings_edit_buf = "1".to_string();
        app.handle_settings_key(key(KeyCode::Enter));
        assert_eq!(app.auto_refresh_secs, super::REFRESH_MIN);
    }

    #[test]
    fn commit_clamps_above_maximum() {
        let mut app = make_app();
        app.settings_edit_mode = true;
        app.settings_edit_buf = "999".to_string();
        app.handle_settings_key(key(KeyCode::Enter));
        assert_eq!(app.auto_refresh_secs, super::REFRESH_MAX);
    }

    #[test]
    fn commit_empty_buf_keeps_current() {
        let mut app = make_app();
        let original = app.auto_refresh_secs;
        app.settings_edit_mode = true;
        app.settings_edit_buf.clear();
        app.handle_settings_key(key(KeyCode::Enter));
        assert_eq!(app.auto_refresh_secs, original);
    }

    #[test]
    fn enter_on_language_row_cycles_language() {
        let mut app = make_app();
        app.showing_settings = true;
        app.settings_cursor = 2;
        assert_eq!(app.i18n.lang, "en");
        // First press moves to the next language in LANGUAGES order.
        app.handle_settings_key(key(KeyCode::Enter));
        assert_eq!(app.i18n.lang, crate::i18n::LANGUAGES[1]);
        // Second press advances one more step.
        app.handle_settings_key(key(KeyCode::Enter));
        assert_eq!(app.i18n.lang, crate::i18n::LANGUAGES[2]);
    }
}
