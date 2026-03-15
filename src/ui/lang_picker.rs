use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::i18n::I18n;

use super::helpers::{centered_popup, popup_block, render_hint_row};

pub(super) fn render_lang_picker(frame: &mut Frame, app: &App) {
    let langs = I18n::all_native_names();
    let n = langs.len();
    let area = frame.area();
    let palette = &app.theme().palette;

    // Narrow popup: 38 cols is enough for the longest entry (native name + code).
    let popup = centered_popup(area, 38, n as u16 + 4);
    frame.render_widget(Clear, popup);
    let block = popup_block(format!(" {} ", app.i18n.t("lang-picker-title")), palette);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // Scrollable entry list — keep the cursor visible.
    let entry_rows = (inner.height as usize).saturating_sub(2); // reserve blank + hint
    let offset = if app.overlay.lang_picker_cursor >= entry_rows {
        app.overlay.lang_picker_cursor - entry_rows + 1
    } else {
        0
    };

    for (i, (code, native_name)) in langs.iter().enumerate().skip(offset).take(entry_rows) {
        let row_y = inner.y + (i - offset) as u16;
        let is_selected = i == app.overlay.lang_picker_cursor;
        let cursor_str = if is_selected { " > " } else { "   " };
        let display_name = app.i18n.map_text_for_web(native_name);
        let label = format!("{display_name} ({code})");

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
        let name_span = Span::styled(
            label,
            if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        );
        let row_style = if is_selected {
            Style::default().bg(palette.dim)
        } else {
            Style::default()
        };

        frame.render_widget(
            Paragraph::new(Line::from(vec![cursor_span, name_span])).style(row_style),
            Rect {
                x: inner.x,
                y: row_y,
                width: inner.width,
                height: 1,
            },
        );
    }

    // Reuse the same hint as the theme picker (navigate / apply / close).
    render_hint_row(frame, inner, &app.i18n.t("footer-theme-picker"), palette);
}
