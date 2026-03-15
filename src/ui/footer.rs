use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub(super) fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let span = if !app.jump_buf.is_empty() {
        Span::styled(
            format!("  Jump: {}_", app.jump_buf),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else if let Some((msg, _)) = &app.status_msg {
        Span::styled(
            format!("  {} ", msg),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else if app.searching {
        Span::styled(
            "  Type to filter   [\u{2191}\u{2193}] Navigate   [\u{21B5}] Confirm   [Esc] Cancel",
            Style::default().fg(Color::Yellow),
        )
    } else {
        Span::styled(
            "  [\u{2191}\u{2193}/j/k] Move   [f] Favourite   [r] Refresh   [/] Search   [g/G] \u{21B1}/\u{21B3}   [q] Quit",
            Style::default().fg(Color::DarkGray),
        )
    };

    frame.render_widget(Paragraph::new(Line::from(span)), area);
}
