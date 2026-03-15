#![cfg(target_arch = "wasm32")]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;

use ratzilla::event::{KeyCode, KeyEvent, MouseButton, MouseEventKind};
use ratzilla::web_sys;
use ratzilla::WebRenderer;

use crate::app::App;
use crate::message::Message;
use crate::models::AppEvent;
use crate::web_atlas;

const WEB_FONT_LINK_ID: &str = "nextbus-web-fonts";
const WEB_FONT_STYLESHEET_URL: &str = "https://fonts.googleapis.com/css2?family=Noto+Sans+Mono:wght@400;700&family=Noto+Sans+JP:wght@400;700&family=Noto+Sans+SC:wght@400;700&family=Noto+Sans+TC:wght@400;700&family=Noto+Sans+Tamil:wght@400;700&display=swap";
const WEB_FONT_STACK: &str = "'Noto Sans Mono', 'Noto Sans SC', 'Noto Sans TC', 'Noto Sans JP', 'Noto Sans Tamil', monospace";

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
    let Some(document) = window.document() else {
        return;
    };

    // Prevent the browser's native find-in-page dialog when "/" is pressed,
    // since it is used as the search shortcut in this app.
    let prevent_slash: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
        Closure::wrap(Box::new(|event: web_sys::KeyboardEvent| {
            if event.key() == "/" {
                event.prevent_default();
            }
        }));
    let _ = document
        .add_event_listener_with_callback("keydown", prevent_slash.as_ref().unchecked_ref());
    prevent_slash.forget();

    let cfg = crate::config::load();
    let backend = match web_atlas::create_backend_for_lang(&cfg.language) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
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

    apply_web_font_css();

    // Shared terminal size (cols, rows) updated every draw frame so that
    // mouse/wheel/touch handlers can convert pixel coords to cell coords.
    let term_size: Rc<Cell<(u16, u16)>> = Rc::new(Cell::new((80, 24)));

    // Mouse click — left button press only.
    terminal.on_mouse_event({
        let app_ref = Rc::clone(&app);
        let term_size = Rc::clone(&term_size);
        move |mouse_event| {
            if mouse_event.event != MouseEventKind::Pressed
                || mouse_event.button != MouseButton::Left
            {
                return;
            }
            let (term_w, term_h) = term_size.get();
            let Some((col, row)) = pixel_to_cell(mouse_event.x, mouse_event.y, term_w, term_h)
            else {
                return;
            };
            let msg = {
                let app_view = app_ref.borrow();
                web_left_click_message(col, row, &app_view, term_w, term_h)
            };
            if let Some(msg) = msg {
                app_ref.borrow_mut().update(msg);
            }
        }
    });

    // Wheel / trackpad scroll — scroll the stop list when over the list panel.
    {
        let app_ref = Rc::clone(&app);
        let term_size = Rc::clone(&term_size);
        let wheel_cb: Closure<dyn FnMut(web_sys::WheelEvent)> =
            Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
                let (term_w, term_h) = term_size.get();
                let Some((col, _)) = pixel_to_cell(
                    event.client_x() as u32,
                    event.client_y() as u32,
                    term_w,
                    term_h,
                ) else {
                    return;
                };
                if col >= crate::layout::list_x_end(term_w) {
                    return;
                }
                let msg = if event.delta_y() < 0.0 {
                    Message::ScrollListUp
                } else {
                    Message::ScrollListDown
                };
                event.prevent_default();
                app_ref.borrow_mut().update(msg);
            }));
        let _ =
            document.add_event_listener_with_callback("wheel", wheel_cb.as_ref().unchecked_ref());
        wheel_cb.forget();
    }

    // Touch — tap acts as a click; vertical swipe scrolls the stop list.
    {
        let touch_start: Rc<Cell<(f32, f32)>> = Rc::new(Cell::new((0.0, 0.0)));

        {
            let ts = Rc::clone(&touch_start);
            let touchstart_cb: Closure<dyn FnMut(web_sys::TouchEvent)> =
                Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
                    if let Some(touch) = event.touches().get(0) {
                        ts.set((touch.client_x() as f32, touch.client_y() as f32));
                    }
                }));
            let _ = document.add_event_listener_with_callback(
                "touchstart",
                touchstart_cb.as_ref().unchecked_ref(),
            );
            touchstart_cb.forget();
        }

        {
            let app_ref = Rc::clone(&app);
            let ts = Rc::clone(&touch_start);
            let term_size = Rc::clone(&term_size);
            let touchend_cb: Closure<dyn FnMut(web_sys::TouchEvent)> =
                Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
                    let Some(touch) = event.changed_touches().get(0) else {
                        return;
                    };
                    let (start_x, start_y) = ts.get();
                    let end_x = touch.client_x() as f32;
                    let end_y = touch.client_y() as f32;
                    let dx = end_x - start_x;
                    let dy = end_y - start_y;
                    let (term_w, term_h) = term_size.get();

                    if dx.abs() < 10.0 && dy.abs() < 10.0 {
                        // Tap → treat as a left click at the release position.
                        let Some((col, row)) =
                            pixel_to_cell(end_x as u32, end_y as u32, term_w, term_h)
                        else {
                            return;
                        };
                        let msg = {
                            let app_view = app_ref.borrow();
                            web_left_click_message(col, row, &app_view, term_w, term_h)
                        };
                        if let Some(msg) = msg {
                            app_ref.borrow_mut().update(msg);
                        }
                        return;
                    }

                    // Vertical swipe in the list panel → scroll.
                    if dy.abs() > dx.abs() && dy.abs() > 30.0 {
                        let Some((col, _)) = pixel_to_cell(start_x as u32, 0, term_w, 1) else {
                            return;
                        };
                        if col < crate::layout::list_x_end(term_w) {
                            // Swipe up (finger up) → show items further down the list.
                            // Swipe down (finger down) → show items further up the list.
                            let msg = if dy < 0.0 {
                                Message::ScrollListDown
                            } else {
                                Message::ScrollListUp
                            };
                            event.prevent_default();
                            app_ref.borrow_mut().update(msg);
                        }
                    }
                }));
            let _ = document
                .add_event_listener_with_callback("touchend", touchend_cb.as_ref().unchecked_ref());
            touchend_cb.forget();
        }
    }

    terminal.on_key_event({
        let app_ref = Rc::clone(&app);
        move |key| {
            let msg = {
                let app_view = app_ref.borrow();
                key_to_message(key, &app_view)
            };
            if let Some(msg) = msg {
                let before_lang = app_ref.borrow().i18n.lang.clone();
                app_ref.borrow_mut().update(msg);
                let after_lang = app_ref.borrow().i18n.lang.clone();
                apply_web_font_css();
                if before_lang != after_lang {
                    reload_page();
                }
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
            // Keep the shared terminal size current for pointer event handlers.
            let area = f.area();
            term_size.set((area.width, area.height));

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

fn apply_web_font_css() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    ensure_web_fonts_loaded(&document);

    // Keep CSS font stack on document and canvas in sync for non-WebGL text contexts.
    if let Some(body) = document.body() {
        append_font_family_style(&body, WEB_FONT_STACK);
    }
    if let Ok(Some(canvas)) = document.query_selector("canvas") {
        if let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
            append_font_family_style(&canvas, WEB_FONT_STACK);
            apply_canvas_pointer_css(canvas.as_ref());
        }
    }
}

/// Disable browser default touch/scroll behaviour on the canvas so swipe and
/// tap events reach our handlers without interference.
fn apply_canvas_pointer_css(element: &web_sys::Element) {
    let existing = element.get_attribute("style").unwrap_or_default();
    if existing.contains("touch-action") {
        return;
    }
    let addition = "touch-action: none; user-select: none;";
    let style = if existing.trim().is_empty() {
        addition.to_string()
    } else {
        format!("{} {addition}", existing.trim_end())
    };
    let _ = element.set_attribute("style", &style);
}

fn append_font_family_style<T>(element: &T, font_stack: &str)
where
    T: AsRef<web_sys::Element>,
{
    let element = element.as_ref();
    let existing = element.get_attribute("style").unwrap_or_default();
    if existing.contains("font-family") {
        return;
    }
    let style = if existing.trim().is_empty() {
        format!("font-family: {font_stack};")
    } else {
        format!("{} font-family: {font_stack};", existing.trim_end())
    };
    let _ = element.set_attribute("style", &style);
}

fn ensure_web_fonts_loaded(document: &web_sys::Document) {
    if document.get_element_by_id(WEB_FONT_LINK_ID).is_some() {
        return;
    }

    let Ok(link) = document.create_element("link") else {
        return;
    };
    let _ = link.set_attribute("id", WEB_FONT_LINK_ID);
    let _ = link.set_attribute("rel", "stylesheet");
    let _ = link.set_attribute("href", WEB_FONT_STYLESHEET_URL);

    if let Some(body) = document.body() {
        let _ = body.append_child(&link);
    }
}

// ── Pointer helpers ───────────────────────────────────────────────────────────

/// Convert pixel coords to terminal cell (col, row).
/// Returns `None` if the canvas is not found or has zero size.
fn pixel_to_cell(px: u32, py: u32, term_w: u16, term_h: u16) -> Option<(u16, u16)> {
    if term_w == 0 || term_h == 0 {
        return None;
    }
    let document = web_sys::window()?.document()?;
    let canvas = document
        .query_selector("canvas")
        .ok()??
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .ok()?;
    let cw = canvas.client_width() as u32;
    let ch = canvas.client_height() as u32;
    if cw == 0 || ch == 0 {
        return None;
    }
    let col = ((px * term_w as u32) / cw).min(term_w as u32 - 1) as u16;
    let row = ((py * term_h as u32) / ch).min(term_h as u32 - 1) as u16;
    Some((col, row))
}

fn web_left_click_message(
    col: u16,
    row: u16,
    app: &App,
    term_w: u16,
    term_h: u16,
) -> Option<Message> {
    let footer_y = term_h.saturating_sub(1);

    if row == footer_y {
        return web_footer_click_message(col, app.nav.searching);
    }

    // Click outside footer while search overlay is open → confirm and close.
    if app.nav.searching {
        return Some(Message::CloseSearch { keep_filter: true });
    }

    if col < crate::layout::list_x_end(term_w) {
        web_list_click_message(row, app, term_h)
    } else {
        Some(Message::CancelJump)
    }
}

fn web_list_click_message(row: u16, app: &App, term_h: u16) -> Option<Message> {
    // Row 0 = title bar, row 1 = list top border; last two rows = border + footer.
    let inner_top: u16 = 2;
    let inner_bot: u16 = term_h.saturating_sub(2);

    if row < inner_top || row >= inner_bot {
        return Some(Message::CancelJump);
    }

    let visual_row = (row - inner_top) as usize;
    let target = app.nav.list_state.offset() + visual_row;

    if target < app.nav.sorted_indices.len() {
        Some(Message::ListClick(target))
    } else {
        Some(Message::CancelJump)
    }
}

fn web_footer_click_message(col: u16, searching: bool) -> Option<Message> {
    // Column ranges mirror those in app/mouse.rs `footer_hit()`.
    if searching {
        match col {
            35..=45 => Some(Message::CloseSearch { keep_filter: true }),
            49..=60 => Some(Message::CloseSearch { keep_filter: false }),
            _ => Some(Message::CancelJump),
        }
    } else {
        match col {
            18..=30 => Some(Message::ToggleFavourite),
            34..=44 => Some(Message::RefreshCurrent),
            48..=57 => Some(Message::OpenSearch),
            61..=69 => Some(Message::GoFirst),
            73..=80 => Some(Message::Quit),
            _ => Some(Message::CancelJump),
        }
    }
}

fn reload_page() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let _ = window.location().reload();
}

fn key_to_message(key: KeyEvent, app: &App) -> Option<Message> {
    if app.overlay.showing_lang_picker {
        lang_picker_key(key)
    } else if app.overlay.showing_settings {
        settings_key(key, app)
    } else if app.overlay.showing_theme_picker {
        picker_key(key)
    } else if app.nav.searching {
        search_key(key)
    } else {
        normal_key(key, app)
    }
}

fn lang_picker_key(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc => Some(Message::CloseLangPicker),
        KeyCode::Enter => Some(Message::LangPickerApply),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::LangPickerUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::LangPickerDown),
        _ => None,
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
