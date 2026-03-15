use fluent::FluentArgs;

use super::App;
use crate::theme::ThemeMode;

pub const REFRESH_MIN: u64 = 5;
pub const REFRESH_MAX: u64 = 300;

impl App {
    /// Called from `update(Message::SettingsActivateRow)`.
    pub(super) fn activate_settings_row(&mut self) {
        match self.overlay.settings_cursor {
            0 => {
                self.overlay.settings_edit_mode = true;
                self.overlay.settings_edit_buf = self.settings.auto_refresh_secs.to_string();
            }
            1 => {
                self.settings.default_fav_view = !self.settings.default_fav_view;
                let view_key = if self.settings.default_fav_view {
                    "status-view-favs"
                } else {
                    "status-view-all"
                };
                let view_str = self.i18n.t(view_key);
                let mut args = FluentArgs::new();
                args.set("view", view_str.as_str());
                let msg = self.i18n.t_args("status-view-set", &args);
                self.set_status(&msg);
                self.settings.persist(&self.i18n.lang);
            }
            2 => {
                // Open the language picker popup; apply only on Enter.
                let pos = crate::i18n::LANGUAGES
                    .iter()
                    .position(|&l| l == self.i18n.lang)
                    .unwrap_or(0);
                self.overlay.lang_picker_cursor = pos;
                self.overlay.showing_lang_picker = true;
            }
            3 => {
                self.settings.theme_mode = match self.settings.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Auto,
                    ThemeMode::Auto => ThemeMode::Dark,
                };
                let mode_key = match self.settings.theme_mode {
                    ThemeMode::Dark => "settings-theme-mode-dark",
                    ThemeMode::Light => "settings-theme-mode-light",
                    ThemeMode::Auto => "settings-theme-mode-auto",
                };
                let mode_str = self.i18n.t(mode_key);
                let mut args = FluentArgs::new();
                args.set("mode", mode_str.as_str());
                let msg = self.i18n.t_args("status-theme-mode-set", &args);
                self.set_status(&msg);
                self.settings.persist(&self.i18n.lang);
            }
            _ => {}
        }
    }

    /// Called from `update(Message::LangPickerApply)`.
    pub(super) fn apply_lang_picker(&mut self) {
        if let Some(&code) = crate::i18n::LANGUAGES.get(self.overlay.lang_picker_cursor) {
            self.i18n = crate::i18n::I18n::new(code);
            let name = self
                .i18n
                .map_text_for_web(self.i18n.lang_meta.native_name.as_str());
            let mut args = FluentArgs::new();
            args.set("name", name.as_str());
            let msg = self.i18n.t_args("status-lang-set", &args);
            self.set_status(&msg);
            self.settings.persist(&self.i18n.lang);
        }
        self.overlay.showing_lang_picker = false;
    }

    /// Called from `update(Message::SettingsEditCommit)`.
    pub(super) fn commit_refresh_interval(&mut self) {
        let parsed: u64 = self
            .overlay
            .settings_edit_buf
            .parse()
            .unwrap_or(self.settings.auto_refresh_secs);
        self.settings.auto_refresh_secs = parsed.clamp(REFRESH_MIN, REFRESH_MAX);
        self.overlay.settings_edit_mode = false;
        self.overlay.settings_edit_buf.clear();
        self.settings.persist(&self.i18n.lang);
        let mut args = FluentArgs::new();
        args.set("seconds", self.settings.auto_refresh_secs as i64);
        let msg = self.i18n.t_args("status-interval-set", &args);
        self.set_status(&msg);
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::message::Message;
    use std::sync::mpsc;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        App::new_test(tx)
    }

    #[test]
    fn s_key_closes_settings() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.update(Message::CloseSettings);
        assert!(!app.overlay.showing_settings);
    }

    #[test]
    fn esc_closes_settings() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.update(Message::CloseSettings);
        assert!(!app.overlay.showing_settings);
    }

    #[test]
    fn j_moves_cursor_down() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = 0;
        app.update(Message::SettingsDown);
        assert_eq!(app.overlay.settings_cursor, 1);
    }

    #[test]
    fn k_moves_cursor_up() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = 2;
        app.update(Message::SettingsUp);
        assert_eq!(app.overlay.settings_cursor, 1);
    }

    #[test]
    fn j_at_bottom_stays() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = super::super::SETTINGS_ROW_COUNT - 1;
        app.update(Message::SettingsDown);
        assert_eq!(
            app.overlay.settings_cursor,
            super::super::SETTINGS_ROW_COUNT - 1
        );
    }

    #[test]
    fn k_at_top_stays() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = 0;
        app.update(Message::SettingsUp);
        assert_eq!(app.overlay.settings_cursor, 0);
    }

    #[test]
    fn enter_on_interval_row_enters_edit_mode() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = 0;
        app.update(Message::SettingsActivateRow);
        assert!(app.overlay.settings_edit_mode);
        assert_eq!(
            app.overlay.settings_edit_buf,
            app.settings.auto_refresh_secs.to_string()
        );
    }

    #[test]
    fn enter_on_default_view_row_toggles() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = 1;
        let before = app.settings.default_fav_view;
        app.update(Message::SettingsActivateRow);
        assert_eq!(app.settings.default_fav_view, !before);
    }

    #[test]
    fn edit_mode_digit_appends() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = 0;
        app.overlay.settings_edit_mode = true;
        app.overlay.settings_edit_buf = "1".to_string();
        app.update(Message::SettingsEditChar('5'));
        assert_eq!(app.overlay.settings_edit_buf, "15");
    }

    #[test]
    fn edit_mode_backspace_removes() {
        let mut app = make_app();
        app.overlay.settings_edit_mode = true;
        app.overlay.settings_edit_buf = "30".to_string();
        app.update(Message::SettingsEditBackspace);
        assert_eq!(app.overlay.settings_edit_buf, "3");
    }

    #[test]
    fn edit_mode_esc_cancels_without_saving() {
        let mut app = make_app();
        let original = app.settings.auto_refresh_secs;
        app.overlay.settings_edit_mode = true;
        app.overlay.settings_edit_buf = "99".to_string();
        app.update(Message::SettingsEditCancel);
        assert!(!app.overlay.settings_edit_mode);
        assert_eq!(app.settings.auto_refresh_secs, original);
    }

    #[test]
    fn edit_mode_enter_commits_value() {
        let mut app = make_app();
        app.overlay.settings_edit_mode = true;
        app.overlay.settings_edit_buf = "60".to_string();
        app.update(Message::SettingsEditCommit);
        assert!(!app.overlay.settings_edit_mode);
        assert_eq!(app.settings.auto_refresh_secs, 60);
    }

    #[test]
    fn commit_clamps_below_minimum() {
        let mut app = make_app();
        app.overlay.settings_edit_mode = true;
        app.overlay.settings_edit_buf = "1".to_string();
        app.update(Message::SettingsEditCommit);
        assert_eq!(app.settings.auto_refresh_secs, super::REFRESH_MIN);
    }

    #[test]
    fn commit_clamps_above_maximum() {
        let mut app = make_app();
        app.overlay.settings_edit_mode = true;
        app.overlay.settings_edit_buf = "999".to_string();
        app.update(Message::SettingsEditCommit);
        assert_eq!(app.settings.auto_refresh_secs, super::REFRESH_MAX);
    }

    #[test]
    fn commit_empty_buf_keeps_current() {
        let mut app = make_app();
        let original = app.settings.auto_refresh_secs;
        app.overlay.settings_edit_mode = true;
        app.overlay.settings_edit_buf.clear();
        app.update(Message::SettingsEditCommit);
        assert_eq!(app.settings.auto_refresh_secs, original);
    }

    #[test]
    fn enter_on_language_row_opens_lang_picker() {
        let mut app = make_app();
        app.overlay.showing_settings = true;
        app.overlay.settings_cursor = 2;
        assert_eq!(app.i18n.lang, "en");
        app.update(Message::SettingsActivateRow);
        // Language must not change yet — picker is open, waiting for Enter.
        assert_eq!(app.i18n.lang, "en");
        assert!(app.overlay.showing_lang_picker);
        // LangPickerApply should now commit the selection.
        app.overlay.lang_picker_cursor = 1;
        app.update(Message::LangPickerApply);
        assert_eq!(app.i18n.lang, crate::i18n::LANGUAGES[1]);
        assert!(!app.overlay.showing_lang_picker);
    }
}
