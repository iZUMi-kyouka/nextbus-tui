use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

pub(super) fn render_theme_picker(frame: &mut Frame, app: &App) {
    let n = app.themes.len();
    let area = frame.area();
    let palette = &app.theme().palette;

    // Popup dimensions: enough rows for all themes + 1 blank + 1 hint + 2 borders.
    let popup_h = (n as u16 + 4).min(area.height.saturating_sub(2));
    let popup_w = 52u16.min(area.width.saturating_sub(2));
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" \u{1F3A8} Themes ")
        .border_style(
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    // Scrollable entry list — keep cursor visible.
    let entry_rows = (inner.height as usize).saturating_sub(2); // reserve blank + hint
    let offset = if app.theme_picker_cursor >= entry_rows {
        app.theme_picker_cursor - entry_rows + 1
    } else {
        0
    };

    for (i, theme) in app.themes.iter().enumerate().skip(offset).take(entry_rows) {
        let row_y = inner.y + (i - offset) as u16;

        let is_selected = i == app.theme_picker_cursor;
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
    let hint_y = inner.y + inner.height.saturating_sub(1);
    let hint_area = Rect {
        x: inner.x,
        y: hint_y,
        width: inner.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [\u{2191}\u{2193}/j/k] Navigate   [\u{21B5}] Apply   [Esc] Close",
            Style::default().fg(palette.dim),
        ))),
        hint_area,
    );
}
