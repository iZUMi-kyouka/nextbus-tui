use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

pub(super) fn render_settings(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let palette = &app.theme().palette;

    // 3 content rows + 1 blank spacer + 1 hint + 2 borders = 7 rows minimum.
    let popup_h = 7u16.min(area.height.saturating_sub(2));
    let popup_w = 64u16.min(area.width.saturating_sub(4));
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" \u{2699} Settings ")
        .border_style(
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // ── Build value strings before borrowing them as &str ─────────────────────

    let interval_display: String = if app.settings_edit_mode && app.settings_cursor == 0 {
        format!("[{}\u{2588}]", app.settings_edit_buf)
    } else {
        format!("[{}s]", app.auto_refresh_secs)
    };

    let view_display: &str = if app.default_fav_view {
        "[Favourites]"
    } else {
        "[All stops]"
    };

    // (label, value, is_stub)
    let rows: &[(&str, &str, bool)] = &[
        ("Auto-refresh interval:  ", &interval_display, false),
        ("Default view:           ", view_display, false),
        ("Language:               ", "[English]", true),
    ];

    // ── Render rows ───────────────────────────────────────────────────────────

    for (i, (label, value, is_stub)) in rows.iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.y + inner.height.saturating_sub(1) {
            break; // leave room for hint row
        }

        let is_selected = i == app.settings_cursor && !*is_stub;
        let cursor_str = if is_selected { " > " } else { "   " };

        let cursor_span = Span::styled(
            cursor_str,
            if is_selected {
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        );

        let label_style = if *is_stub {
            Style::default().fg(palette.dim)
        } else {
            Style::default()
        };

        let value_style = if *is_stub {
            Style::default().fg(palette.dim)
        } else if is_selected {
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let mut spans = vec![
            cursor_span,
            Span::styled(*label, label_style),
            Span::styled(*value, value_style),
        ];

        if *is_stub {
            spans.push(Span::styled(
                "  (coming soon)",
                Style::default().fg(palette.dim),
            ));
        }

        let row_bg = if is_selected {
            Style::default().bg(palette.dim)
        } else {
            Style::default()
        };

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(row_bg),
            Rect {
                x: inner.x,
                y: row_y,
                width: inner.width,
                height: 1,
            },
        );
    }

    // ── Hint row ──────────────────────────────────────────────────────────────

    let hint_text = if app.settings_edit_mode {
        " [0-9] Type   [\u{232B}] Delete   [\u{21B5}] Confirm   [Esc] Cancel"
    } else {
        " [\u{2191}\u{2193}/j/k] Navigate   [\u{21B5}/Space] Edit/Toggle   [Esc/s] Close"
    };

    let hint_y = inner.y + inner.height.saturating_sub(1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            hint_text,
            Style::default().fg(palette.dim),
        ))),
        Rect {
            x: inner.x,
            y: hint_y,
            width: inner.width,
            height: 1,
        },
    );
}
