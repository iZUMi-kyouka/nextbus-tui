use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use crate::app::App;

use super::helpers::{centered_popup, popup_block, render_hint_row};

pub(super) fn render_theme_picker(frame: &mut Frame, app: &App) {
    let filtered = app.picker_theme_indices();
    let n = filtered.len();
    let area = frame.area();
    let palette = &app.theme().palette;

    // Popup dimensions: enough rows for all visible themes + 1 blank + 1 hint + 2 borders.
    let popup = centered_popup(area, 52, n as u16 + 4);
    frame.render_widget(Clear, popup);
    let block = popup_block(format!(" {} ", app.i18n.t("theme-title")), palette);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // Scrollable entry list — keep cursor visible.
    let entry_rows = (inner.height as usize).saturating_sub(2); // reserve blank + hint
    let offset = if app.overlay.theme_picker_cursor >= entry_rows {
        app.overlay.theme_picker_cursor - entry_rows + 1
    } else {
        0
    };

    for (list_pos, &theme_global_idx) in filtered.iter().enumerate().skip(offset).take(entry_rows) {
        let theme = &app.domain.themes[theme_global_idx];
        let row_y = inner.y + (list_pos - offset) as u16;

        let is_selected = list_pos == app.overlay.theme_picker_cursor;
        let cursor_str = if is_selected { " > " } else { "   " };

        let mut spans = vec![Span::styled(
            cursor_str,
            if is_selected {
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        )];

        // Colour swatches: five █ blocks in the theme's own swatch colours.
        for &color in &theme.palette.swatch {
            spans.push(Span::styled("\u{2588}", Style::default().fg(color)));
            spans.push(Span::raw(" "));
        }

        let name_style = if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        spans.push(Span::styled(format!(" {}", theme.name), name_style));

        let row_style = if is_selected {
            Style::default().bg(palette.dim)
        } else {
            Style::default()
        };
        let row_area = Rect {
            x: inner.x,
            y: row_y,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(Paragraph::new(Line::from(spans)).style(row_style), row_area);
    }

    // Hint row at the bottom of the inner area.
    render_hint_row(frame, inner, &app.i18n.t("footer-theme-picker"), palette);
}
