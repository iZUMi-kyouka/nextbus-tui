use fluent::FluentArgs;
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

    // 4 content rows (interval / view / language / font) + 1 hint + 2 borders = 7.
    // Add 1 blank row between content and hint → popup_h = 8.
    let popup_h = 8u16.min(area.height.saturating_sub(2));
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
        .title(format!(" {} ", app.i18n.t("settings-title")))
        .border_style(
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // ── Build all value strings before borrowing them as &str ─────────────────

    let interval_display: String = if app.settings_edit_mode && app.settings_cursor == 0 {
        let mut args = FluentArgs::new();
        args.set("value", app.settings_edit_buf.as_str());
        app.i18n.t_args("settings-interval-editing", &args)
    } else {
        let mut args = FluentArgs::new();
        args.set("seconds", app.auto_refresh_secs as i64);
        app.i18n.t_args("settings-interval-value", &args)
    };

    let view_display = if app.default_fav_view {
        app.i18n.t("settings-view-favs")
    } else {
        app.i18n.t("settings-view-all")
    };

    let lang_display = {
        let mut args = FluentArgs::new();
        args.set("name", app.i18n.lang_meta.native_name.as_str());
        app.i18n.t_args("settings-lang-value", &args)
    };

    let font_display = {
        let mut args = FluentArgs::new();
        args.set("font", app.i18n.lang_meta.font.as_str());
        app.i18n.t_args("settings-font-value", &args)
    };

    let interval_label = app.i18n.t("settings-interval-label");
    let view_label = app.i18n.t("settings-view-label");
    let lang_label = app.i18n.t("settings-lang-label");
    let font_label = app.i18n.t("settings-font-label");

    // (label, value, is_interactive)
    // The font row is display-only (is_interactive = false, always dim).
    let rows: &[(&str, &str, bool)] = &[
        (&interval_label, &interval_display, true),
        (&view_label, &view_display, true),
        (&lang_label, &lang_display, true),
        (&font_label, &font_display, false), // display-only
    ];

    // ── Render rows ───────────────────────────────────────────────────────────

    for (i, (label, value, is_interactive)) in rows.iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.y + inner.height.saturating_sub(1) {
            break; // leave the last row for the hint
        }

        // Only interactive rows (0–2) can be selected by the cursor.
        let is_selected = *is_interactive && i == app.settings_cursor;
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

        let label_style = if !is_interactive {
            Style::default().fg(palette.dim)
        } else {
            Style::default()
        };

        let value_style = if !is_interactive {
            Style::default().fg(palette.dim)
        } else if is_selected {
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let spans = vec![
            cursor_span,
            Span::styled(*label, label_style),
            Span::raw(" "),
            Span::styled(*value, value_style),
        ];

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

    let hint_key = if app.settings_edit_mode {
        "settings-hint-edit"
    } else {
        "settings-hint-nav"
    };
    let hint_y = inner.y + inner.height.saturating_sub(1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {}", app.i18n.t(hint_key)),
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
