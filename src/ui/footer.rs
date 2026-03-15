use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub(super) fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let p = &app.theme().palette;

    let span = if !app.jump_buf.is_empty() {
        Span::styled(
            format!("  Jump: {}_", app.jump_buf),
            Style::default().fg(p.jump).add_modifier(Modifier::BOLD),
        )
    } else if let Some((msg, _)) = &app.status_msg {
        Span::styled(
            format!("  {} ", msg),
            Style::default().fg(p.success).add_modifier(Modifier::BOLD),
        )
    } else if app.showing_settings {
        Span::styled(
            if app.settings_edit_mode {
                "  [0-9] Type   [\u{232B}] Delete   [\u{21B5}] Confirm   [Esc] Cancel"
            } else {
                "  [\u{2191}\u{2193}/j/k] Navigate   [\u{21B5}/Space] Edit/Toggle   [Esc/s] Close"
            },
            Style::default().fg(p.highlight),
        )
    } else if app.searching {
        Span::styled(
            "  Type to filter   [\u{2191}\u{2193}] Navigate   [\u{21B5}] Confirm   [Esc] Cancel",
            Style::default().fg(p.highlight),
        )
    } else {
        Span::styled(
            "  [\u{2191}\u{2193}/j/k] Move   [f] Favourite   [r] Refresh   [/] Search   [s] Settings   [q] Quit",
            Style::default().fg(p.dim),
        )
    };

    frame.render_widget(Paragraph::new(Line::from(span)), area);
}
