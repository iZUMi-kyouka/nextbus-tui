use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use super::App;
use crate::message::Message;

// ── Footer button hit regions ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum FooterHit {
    Favourite,
    Refresh,
    Search,
    GoFirst,
    Quit,
    ConfirmSearch,
    CancelSearch,
}

fn footer_hit(x: u16, searching: bool) -> Option<FooterHit> {
    if searching {
        match x {
            35..=45 => Some(FooterHit::ConfirmSearch),
            49..=60 => Some(FooterHit::CancelSearch),
            _ => None,
        }
    } else {
        match x {
            18..=30 => Some(FooterHit::Favourite),
            34..=44 => Some(FooterHit::Refresh),
            48..=57 => Some(FooterHit::Search),
            61..=69 => Some(FooterHit::GoFirst),
            73..=80 => Some(FooterHit::Quit),
            _ => None,
        }
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Pure function — reads app state and event, returns intent. No mutation.
pub fn mouse_to_message(event: MouseEvent, app: &App, term_w: u16, term_h: u16) -> Option<Message> {
    match event.kind {
        MouseEventKind::ScrollUp => {
            if event.column < crate::layout::list_x_end(term_w) {
                Some(Message::MoveUp)
            } else {
                None
            }
        }
        MouseEventKind::ScrollDown => {
            if event.column < crate::layout::list_x_end(term_w) {
                Some(Message::MoveDown)
            } else {
                None
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            left_click_message(event.column, event.row, app, term_w, term_h)
        }
        _ => None,
    }
}

fn left_click_message(col: u16, row: u16, app: &App, term_w: u16, term_h: u16) -> Option<Message> {
    let footer_y = term_h.saturating_sub(1);

    if row == footer_y {
        return footer_click_message(col, app.nav.searching);
    }

    // Any click outside footer cancels jump.
    // If search overlay is open, click outside footer closes it.
    if app.nav.searching {
        return Some(Message::CloseSearch { keep_filter: true });
    }

    if col < crate::layout::list_x_end(term_w) {
        list_click_message(row, app, term_h)
    } else {
        Some(Message::CancelJump)
    }
}

fn list_click_message(row: u16, app: &App, term_h: u16) -> Option<Message> {
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

fn footer_click_message(col: u16, searching: bool) -> Option<Message> {
    match footer_hit(col, searching) {
        Some(FooterHit::Favourite) => Some(Message::ToggleFavourite),
        Some(FooterHit::Refresh) => Some(Message::RefreshCurrent),
        Some(FooterHit::Search) => Some(Message::OpenSearch),
        Some(FooterHit::GoFirst) => Some(Message::GoFirst),
        Some(FooterHit::Quit) => Some(Message::Quit),
        Some(FooterHit::ConfirmSearch) => Some(Message::CloseSearch { keep_filter: true }),
        Some(FooterHit::CancelSearch) => Some(Message::CloseSearch { keep_filter: false }),
        None => Some(Message::CancelJump),
    }
}
