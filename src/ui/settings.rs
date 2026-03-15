use fluent::FluentArgs;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::ThemeMode;

use super::helpers::{centered_popup, popup_block, render_hint_row};

pub(super) fn render_settings(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let palette = &app.theme().palette;

    // 4 content rows (interval / view / language / theme mode) + 1 hint + 2 borders = 7.
    // Add 1 blank row between content and hint → popup_h = 8.
    let popup = centered_popup(area, 64, 8);
    frame.render_widget(Clear, popup);
    let block = popup_block(format!(" {} ", app.i18n.t("settings-title")), palette);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // ── Build all value strings before borrowing them as &str ─────────────────

    let interval_display: String =
        if app.overlay.settings_edit_mode && app.overlay.settings_cursor == 0 {
            let mut args = FluentArgs::new();
            args.set("value", app.overlay.settings_edit_buf.as_str());
            app.i18n.t_args("settings-interval-editing", &args)
        } else {
            let mut args = FluentArgs::new();
            args.set("seconds", app.settings.auto_refresh_secs as i64);
            app.i18n.t_args("settings-interval-value", &args)
        };

    let view_display = if app.settings.default_fav_view {
        app.i18n.t("settings-view-favs")
    } else {
        app.i18n.t("settings-view-all")
    };

    let lang_display = {
        let mut args = FluentArgs::new();
        args.set("name", app.i18n.lang_meta.native_name.as_str());
        app.i18n.t_args("settings-lang-value", &args)
    };

    let theme_mode_display = match app.settings.theme_mode {
        ThemeMode::Dark => app.i18n.t("settings-theme-mode-dark"),
        ThemeMode::Light => app.i18n.t("settings-theme-mode-light"),
        ThemeMode::Auto => app.i18n.t("settings-theme-mode-auto"),
    };

    let interval_label = app.i18n.t("settings-interval-label");
    let view_label = app.i18n.t("settings-view-label");
    let lang_label = app.i18n.t("settings-lang-label");
    let theme_mode_label = app.i18n.t("settings-theme-mode-label");

    // (label, value, is_interactive)
    let rows: &[(&str, &str, bool)] = &[
        (&interval_label, &interval_display, true),
        (&view_label, &view_display, true),
        (&lang_label, &lang_display, true),
        (&theme_mode_label, &theme_mode_display, true),
    ];

    // ── Render rows ───────────────────────────────────────────────────────────

    for (i, (label, value, is_interactive)) in rows.iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.y + inner.height.saturating_sub(1) {
            break; // leave the last row for the hint
        }

        // Only interactive rows (0–2) can be selected by the cursor.
        let is_selected = *is_interactive && i == app.overlay.settings_cursor;
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

    let hint_key = if app.overlay.settings_edit_mode {
        "settings-hint-edit"
    } else {
        "settings-hint-nav"
    };
    render_hint_row(frame, inner, &app.i18n.t(hint_key), palette);
}
