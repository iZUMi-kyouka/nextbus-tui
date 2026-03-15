#![cfg(target_arch = "wasm32")]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;

use ratzilla::event::{KeyCode, KeyEvent};
use ratzilla::web_sys;
use ratzilla::{WebGl2Backend, WebRenderer};

use crate::app::App;
use crate::message::Message;
use crate::models::AppEvent;

pub fn start() {
    console_error_panic_hook::set_once();
    STARTED.with(|started| {
        if started.replace(true) {
            return;
        }
        start_when_dom_ready();
    });
}

thread_local! {
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

#[wasm_bindgen(start)]
pub fn wasm_start() {
    start();
}

fn start_when_dom_ready() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    if document.ready_state() == "loading" {
        let cb = Closure::wrap(Box::new(move || {
            bootstrap_runtime();
        }) as Box<dyn FnMut()>);
        let _ = document
            .add_event_listener_with_callback("DOMContentLoaded", cb.as_ref().unchecked_ref());
        cb.forget();
    } else {
        bootstrap_runtime();
    }
}

fn bootstrap_runtime() {
    let Some(window) = web_sys::window() else {
        return;
    };

    let backend = match WebGl2Backend::new() {
        Ok(b) => b,
        Err(_) => return,
    };
    let terminal = match ratzilla::ratatui::Terminal::new(backend) {
        Ok(t) => t,
        Err(_) => return,
    };

    let (tx, rx) = mpsc::channel::<AppEvent>();
    let app = Rc::new(RefCell::new(App::new(tx.clone())));
    let rx = Rc::new(RefCell::new(rx));
    {
        let mut app_ref = app.borrow_mut();
        app_ref.ensure_data();
    }

    terminal.on_key_event({
        let app_ref = Rc::clone(&app);
        move |key| {
            let msg = {
                let app_view = app_ref.borrow();
                key_to_message(key, &app_view)
            };
            if let Some(msg) = msg {
                app_ref.borrow_mut().update(msg);
            }
        }
    });

    {
        let tick_tx = tx.clone();
        let tick_cb = Closure::wrap(Box::new(move || {
            let _ = tick_tx.send(AppEvent::Tick);
        }) as Box<dyn FnMut()>);
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
            tick_cb.as_ref().unchecked_ref(),
            500,
        );
        tick_cb.forget();
    }

    {
        let app_ref = Rc::clone(&app);
        let rx_ref = Rc::clone(&rx);
        terminal.draw_web(move |f| {
            loop {
                let ev = { rx_ref.borrow_mut().try_recv().ok() };
                let Some(ev) = ev else { break };
                app_ref.borrow_mut().update(Message::from(ev));
            }
            let mut app = app_ref.borrow_mut();
            crate::ui::render(f, &mut app);
        });
    }
}

fn key_to_message(key: KeyEvent, app: &App) -> Option<Message> {
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

fn picker_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('X') => Some(Message::CloseThemePicker),
        KeyCode::Enter => Some(Message::ThemePickerApply),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::ThemePickerUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::ThemePickerDown),
        _ => None,
    }
}

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

fn normal_key(key: KeyEvent, app: &App) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('c') if key.ctrl => Some(Message::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::MoveUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::MoveDown),
        KeyCode::Home => Some(Message::GoFirst),
        KeyCode::End => Some(Message::GoLast),
        KeyCode::Char('g') if !key.shift => Some(Message::GoFirst),
        KeyCode::Char('G') | KeyCode::Char('g') if key.shift => Some(Message::GoLast),
        KeyCode::Char('f') if !key.shift => Some(Message::ToggleFavourite),
        KeyCode::Char('F') | KeyCode::Char('f') if key.shift => Some(Message::ToggleFavView),
        KeyCode::Char('r') => Some(Message::RefreshCurrent),
        KeyCode::Char('/') => Some(Message::OpenSearch),
        KeyCode::Char('x') if !key.shift => Some(Message::CycleTheme),
        KeyCode::Char('X') | KeyCode::Char('x') if key.shift => Some(Message::OpenThemePicker),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(Message::OpenSettings),
        KeyCode::Char(c) if c.is_ascii_digit() => Some(Message::JumpDigit(c)),
        KeyCode::Enter if !app.nav.jump_buf.is_empty() => Some(Message::CommitJump),
        _ => Some(Message::CancelJump),
    }
}
