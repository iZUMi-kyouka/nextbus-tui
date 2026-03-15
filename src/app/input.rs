#![cfg(not(target_arch = "wasm32"))]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::App;
use crate::message::Message;

/// Pure function — reads app state, returns the intent. No mutation.
pub fn key_to_message(key: KeyEvent, app: &App) -> Option<Message> {
    if app.overlay.showing_settings {
        settings_key(key, app)
    } else if app.overlay.showing_theme_picker {
        picker_key(key)
    } else if app.nav.searching {
        search_key(key)
    } else {
        normal_key(key, app)
    }
}

// ── Settings mode ─────────────────────────────────────────────────────────────

fn settings_key(key: KeyEvent, app: &App) -> Option<Message> {
    if app.overlay.settings_edit_mode {
        settings_edit_key(key)
    } else {
        settings_nav_key(key)
    }
}

fn settings_nav_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('S') => Some(Message::CloseSettings),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::SettingsUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::SettingsDown),
        KeyCode::Enter | KeyCode::Char(' ') => Some(Message::SettingsActivateRow),
        _ => None,
    }
}

fn settings_edit_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc => Some(Message::SettingsEditCancel),
        KeyCode::Enter => Some(Message::SettingsEditCommit),
        KeyCode::Backspace => Some(Message::SettingsEditBackspace),
        KeyCode::Char(c) if c.is_ascii_digit() => Some(Message::SettingsEditChar(c)),
        _ => None,
    }
}

// ── Theme picker mode ─────────────────────────────────────────────────────────

fn picker_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('X') => Some(Message::CloseThemePicker),
        KeyCode::Enter => Some(Message::ThemePickerApply),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::ThemePickerUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::ThemePickerDown),
        _ => None,
    }
}

// ── Search mode ───────────────────────────────────────────────────────────────

fn search_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc => Some(Message::CloseSearch { keep_filter: false }),
        KeyCode::Enter => Some(Message::CloseSearch { keep_filter: true }),
        KeyCode::Backspace => Some(Message::SearchBackspace),
        KeyCode::Char(c) => Some(Message::SearchChar(c)),
        KeyCode::Up => Some(Message::MoveUp),
        KeyCode::Down => Some(Message::MoveDown),
        _ => None,
    }
}

// ── Normal mode ───────────────────────────────────────────────────────────────

fn normal_key(key: KeyEvent, app: &App) -> Option<Message> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            Some(Message::Quit)
        }
        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => Some(Message::MoveUp),
        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => Some(Message::MoveDown),
        (KeyCode::Char('g'), KeyModifiers::NONE) => Some(Message::GoFirst),
        (KeyCode::Char('G'), _) => Some(Message::GoLast),
        (KeyCode::Home, _) => Some(Message::GoFirst),
        (KeyCode::End, _) => Some(Message::GoLast),
        (KeyCode::Char('f'), _) => Some(Message::ToggleFavourite),
        (KeyCode::Char('F'), _) => Some(Message::ToggleFavView),
        (KeyCode::Char('r'), _) => Some(Message::RefreshCurrent),
        (KeyCode::Char('/'), _) => Some(Message::OpenSearch),
        (KeyCode::Char('x'), _) => Some(Message::CycleTheme),
        (KeyCode::Char('X'), _) => Some(Message::OpenThemePicker),
        (KeyCode::Char('s'), _) | (KeyCode::Char('S'), _) => Some(Message::OpenSettings),
        (KeyCode::Char(c), _) if c.is_ascii_digit() => Some(Message::JumpDigit(c)),
        (KeyCode::Enter, _) if !app.nav.jump_buf.is_empty() => Some(Message::CommitJump),
        _ => Some(Message::CancelJump),
    }
}

#[cfg(test)]
mod tests {
    use super::{App, key_to_message};
    use crate::message::Message;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::mpsc;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        App::new_test(tx)
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn apply(app: &mut App, code: KeyCode) {
        if let Some(msg) = key_to_message(key(code), app) {
            app.update(msg);
        }
    }

    fn apply_ctrl(app: &mut App, code: KeyCode) {
        if let Some(msg) = key_to_message(ctrl(code), app) {
            app.update(msg);
        }
    }

    // ── Normal mode ────────────────────────────────────────────────────────────

    #[test]
    fn q_sets_should_quit() {
        let mut app = make_app();
        apply(&mut app, KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn ctrl_c_sets_should_quit() {
        let mut app = make_app();
        apply_ctrl(&mut app, KeyCode::Char('c'));
        assert!(app.should_quit);
    }

    #[test]
    fn down_moves_selection() {
        let mut app = make_app();
        apply(&mut app, KeyCode::Down);
        assert_eq!(app.nav.selected, 1);
    }

    #[test]
    fn j_moves_selection_down() {
        let mut app = make_app();
        apply(&mut app, KeyCode::Char('j'));
        assert_eq!(app.nav.selected, 1);
    }

    #[test]
    fn up_at_top_stays() {
        let mut app = make_app();
        apply(&mut app, KeyCode::Up);
        assert_eq!(app.nav.selected, 0);
    }

    #[test]
    fn k_at_top_stays() {
        let mut app = make_app();
        apply(&mut app, KeyCode::Char('k'));
        assert_eq!(app.nav.selected, 0);
    }

    #[test]
    fn g_goes_to_first() {
        let mut app = make_app();
        app.nav.selected = 15;
        app.nav.list_state.select(Some(15));
        apply(&mut app, KeyCode::Char('g'));
        assert_eq!(app.nav.selected, 0);
    }

    #[test]
    fn shift_g_goes_to_last() {
        let mut app = make_app();
        if let Some(msg) =
            key_to_message(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE), &app)
        {
            app.update(msg);
        }
        assert_eq!(app.nav.selected, app.nav.sorted_indices.len() - 1);
    }

    #[test]
    fn f_toggles_favourite() {
        let mut app = make_app();
        let name = app.current_stop().unwrap().name.clone();
        apply(&mut app, KeyCode::Char('f'));
        assert!(app.settings.favourites.contains(&name));
    }

    #[test]
    fn shift_f_enables_fav_view() {
        let mut app = make_app();
        assert!(!app.nav.fav_view);
        if let Some(msg) =
            key_to_message(KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE), &app)
        {
            app.update(msg);
        }
        assert!(app.nav.fav_view);
    }

    #[test]
    fn shift_f_twice_toggles_back() {
        let mut app = make_app();
        for _ in 0..2 {
            if let Some(msg) =
                key_to_message(KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE), &app)
            {
                app.update(msg);
            }
        }
        assert!(!app.nav.fav_view);
    }

    #[test]
    fn shift_f_clears_search_query() {
        let mut app = make_app();
        app.nav.search_query = "COM".to_string();
        if let Some(msg) =
            key_to_message(KeyEvent::new(KeyCode::Char('F'), KeyModifiers::NONE), &app)
        {
            app.update(msg);
        }
        assert!(app.nav.search_query.is_empty());
    }

    #[test]
    fn slash_opens_search_with_empty_query() {
        let mut app = make_app();
        app.nav.search_query = "old".to_string();
        apply(&mut app, KeyCode::Char('/'));
        assert!(app.nav.searching);
        assert!(app.nav.search_query.is_empty());
    }

    #[test]
    fn x_cycles_theme() {
        let mut app = make_app();
        let indices = app.picker_theme_indices();
        let initial = app.settings.theme_idx;
        let pos = indices.iter().position(|&i| i == initial).unwrap_or(0);
        let expected = indices[(pos + 1) % indices.len()];
        apply(&mut app, KeyCode::Char('x'));
        assert_eq!(app.settings.theme_idx, expected);
    }

    #[test]
    fn x_wraps_theme_at_end() {
        let mut app = make_app();
        let indices = app.picker_theme_indices();
        app.settings.theme_idx = *indices.last().unwrap();
        apply(&mut app, KeyCode::Char('x'));
        assert_eq!(app.settings.theme_idx, indices[0]);
    }

    #[test]
    fn shift_x_opens_theme_picker() {
        let mut app = make_app();
        if let Some(msg) =
            key_to_message(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE), &app)
        {
            app.update(msg);
        }
        assert!(app.overlay.showing_theme_picker);
    }

    #[test]
    fn shift_x_preselects_current_theme() {
        let mut app = make_app();
        // Pick the 4th entry in the filtered picker list and set it as active.
        let indices = app.picker_theme_indices();
        let target_pos = 3.min(indices.len() - 1);
        app.settings.theme_idx = indices[target_pos];
        if let Some(msg) =
            key_to_message(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE), &app)
        {
            app.update(msg);
        }
        assert_eq!(app.overlay.theme_picker_cursor, target_pos);
    }

    #[test]
    fn digit_buffers_jump_in_large_list() {
        let mut app = make_app();
        apply(&mut app, KeyCode::Char('5'));
        assert_eq!(app.nav.jump_buf, "5");
    }

    // ── Search mode ────────────────────────────────────────────────────────────

    #[test]
    fn search_char_appends_to_query() {
        let mut app = make_app();
        app.nav.searching = true;
        if let Some(msg) = key_to_message(key(KeyCode::Char('C')), &app) {
            app.update(msg);
        }
        assert_eq!(app.nav.search_query, "C");
    }

    #[test]
    fn search_backspace_removes_char() {
        let mut app = make_app();
        app.nav.searching = true;
        app.nav.search_query = "COM".to_string();
        app.rebuild_list();
        if let Some(msg) = key_to_message(key(KeyCode::Backspace), &app) {
            app.update(msg);
        }
        assert_eq!(app.nav.search_query, "CO");
    }

    #[test]
    fn search_esc_closes_and_clears() {
        let mut app = make_app();
        app.nav.searching = true;
        app.nav.search_query = "COM".to_string();
        if let Some(msg) = key_to_message(key(KeyCode::Esc), &app) {
            app.update(msg);
        }
        assert!(!app.nav.searching);
        assert!(app.nav.search_query.is_empty());
    }

    #[test]
    fn search_enter_closes_keeps_filter() {
        let mut app = make_app();
        app.nav.searching = true;
        app.nav.search_query = "COM".to_string();
        app.rebuild_list();
        let n = app.nav.sorted_indices.len();
        if let Some(msg) = key_to_message(key(KeyCode::Enter), &app) {
            app.update(msg);
        }
        assert!(!app.nav.searching);
        assert_eq!(app.nav.search_query, "COM");
        assert_eq!(app.nav.sorted_indices.len(), n);
    }

    // ── Theme picker mode ─────────────────────────────────────────────────────

    #[test]
    fn theme_picker_esc_closes() {
        let mut app = make_app();
        app.overlay.showing_theme_picker = true;
        if let Some(msg) = key_to_message(key(KeyCode::Esc), &app) {
            app.update(msg);
        }
        assert!(!app.overlay.showing_theme_picker);
    }

    #[test]
    fn theme_picker_shift_x_closes() {
        let mut app = make_app();
        app.overlay.showing_theme_picker = true;
        if let Some(msg) =
            key_to_message(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE), &app)
        {
            app.update(msg);
        }
        assert!(!app.overlay.showing_theme_picker);
    }

    #[test]
    fn theme_picker_enter_applies_and_closes() {
        let mut app = make_app();
        app.overlay.showing_theme_picker = true;
        app.overlay.theme_picker_cursor = 2;
        // Resolve which global theme index the picker cursor maps to.
        let expected_idx = app.picker_theme_indices()[2];
        if let Some(msg) = key_to_message(key(KeyCode::Enter), &app) {
            app.update(msg);
        }
        assert_eq!(app.settings.theme_idx, expected_idx);
        assert!(!app.overlay.showing_theme_picker);
    }

    #[test]
    fn theme_picker_j_moves_cursor_down() {
        let mut app = make_app();
        app.overlay.showing_theme_picker = true;
        app.overlay.theme_picker_cursor = 0;
        if let Some(msg) = key_to_message(key(KeyCode::Char('j')), &app) {
            app.update(msg);
        }
        assert_eq!(app.overlay.theme_picker_cursor, 1);
    }

    #[test]
    fn theme_picker_k_moves_cursor_up() {
        let mut app = make_app();
        app.overlay.showing_theme_picker = true;
        app.overlay.theme_picker_cursor = 3;
        if let Some(msg) = key_to_message(key(KeyCode::Char('k')), &app) {
            app.update(msg);
        }
        assert_eq!(app.overlay.theme_picker_cursor, 2);
    }

    #[test]
    fn theme_picker_k_at_top_stays() {
        let mut app = make_app();
        app.overlay.showing_theme_picker = true;
        app.overlay.theme_picker_cursor = 0;
        if let Some(msg) = key_to_message(key(KeyCode::Char('k')), &app) {
            app.update(msg);
        }
        assert_eq!(app.overlay.theme_picker_cursor, 0);
    }

    #[test]
    fn theme_picker_j_at_bottom_stays() {
        let mut app = make_app();
        app.overlay.showing_theme_picker = true;
        let bottom = app.picker_theme_indices().len() - 1;
        app.overlay.theme_picker_cursor = bottom;
        if let Some(msg) = key_to_message(key(KeyCode::Char('j')), &app) {
            app.update(msg);
        }
        assert_eq!(app.overlay.theme_picker_cursor, bottom);
    }
}
