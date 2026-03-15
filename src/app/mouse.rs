use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use super::App;

// ── Footer button hit regions (normal mode) ───────────────────────────────────
//
// The normal-mode footer string is (display columns, 0-indexed):
//   "  [↑↓/j/k] Move   [f] Favourite   [r] Refresh   [/] Search   [g/G] ⇱/⇲   [q] Quit"
//
//    0- 1  leading spaces
//    2-14  [↑↓/j/k] Move      — direction hint, intentionally not a click target
//   15-17  separator
//   18-30  [f] Favourite
//   31-33  separator
//   34-44  [r] Refresh
//   45-47  separator
//   48-57  [/] Search
//   58-60  separator
//   61-69  [g/G] ⇱/⇲
//   70-72  separator
//   73-80  [q] Quit
//
// The search-mode footer string:
//   "  Type to filter   [↑↓] Navigate   [↵] Confirm   [Esc] Cancel"
//   35-45  [↵] Confirm
//   49-60  [Esc] Cancel

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

// ── Layout helpers ────────────────────────────────────────────────────────────

/// X coordinate of the right edge (exclusive) of the stop list panel.
fn list_x_end(term_w: u16) -> u16 {
    if term_w < 100 {
        term_w * 40 / 100
    } else {
        term_w * 33 / 100
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

impl App {
    /// Dispatch a crossterm mouse event.
    /// `term_w`/`term_h` are the current terminal dimensions, used to reconstruct
    /// the layout without storing any UI state in App.
    pub fn handle_mouse(&mut self, event: MouseEvent, term_w: u16, term_h: u16) {
        match event.kind {
            MouseEventKind::ScrollUp => self.on_scroll(event.column, term_w, true),
            MouseEventKind::ScrollDown => self.on_scroll(event.column, term_w, false),
            MouseEventKind::Down(MouseButton::Left) => {
                self.on_left_click(event.column, event.row, term_w, term_h)
            }
            _ => {}
        }
    }

    // ── Private handlers ──────────────────────────────────────────────────────

    /// Scroll wheel — restricted to the list panel so scrolling over the detail
    /// panel or footer is a no-op (prevents accidental navigation).
    fn on_scroll(&mut self, col: u16, term_w: u16, up: bool) {
        if col < list_x_end(term_w) {
            if up {
                self.move_up();
            } else {
                self.move_down();
            }
        }
    }

    fn on_left_click(&mut self, col: u16, row: u16, term_w: u16, term_h: u16) {
        // Any click cancels a pending number-jump, same as non-digit key presses do.
        self.cancel_jump();

        let footer_y = term_h.saturating_sub(1);

        if row == footer_y {
            self.on_footer_click(col);
            return;
        }

        // If the search overlay is open, any click outside the footer closes it
        // (keeping the current filter text so the user can review the filtered list).
        if self.searching {
            self.searching = false;
            return;
        }

        if col < list_x_end(term_w) {
            self.on_list_click(row, term_h);
        }
    }

    /// Map a click inside the stop list panel to a stop selection.
    ///
    /// Layout (absolute rows):
    ///   0          title bar
    ///   1          list top border
    ///   2..=h-3    list inner items  ← clickable
    ///   h-2        list bottom border
    ///   h-1        footer
    fn on_list_click(&mut self, row: u16, term_h: u16) {
        let inner_top: u16 = 2;
        let inner_bot: u16 = term_h.saturating_sub(2); // exclusive

        if row < inner_top || row >= inner_bot {
            return;
        }

        let visual_row = (row - inner_top) as usize;
        let target = self.list_state.offset() + visual_row;

        if target < self.sorted_indices.len() {
            self.selected = target;
            self.list_state.select(Some(target));
            self.ensure_data();
        }
    }

    fn on_footer_click(&mut self, col: u16) {
        match footer_hit(col, self.searching) {
            Some(FooterHit::Favourite) => self.toggle_favourite(),
            Some(FooterHit::Refresh) => self.refresh_current(),
            Some(FooterHit::Search) => self.searching = true,
            Some(FooterHit::GoFirst) => self.go_first(),
            Some(FooterHit::Quit) => self.should_quit = true,
            Some(FooterHit::ConfirmSearch) => {
                self.searching = false;
                self.ensure_data();
            }
            Some(FooterHit::CancelSearch) => {
                self.searching = false;
                self.search_query.clear();
                self.rebuild_list();
                self.ensure_data();
            }
            None => {}
        }
    }
}
