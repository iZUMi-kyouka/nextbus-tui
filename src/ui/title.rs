use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub(super) fn render_title(frame: &mut Frame, area: Rect, app: &App) {
    let p = &app.theme().palette;
    let line = Line::from(vec![
        Span::styled(
            " NUS NextBus TUI ",
            Style::default()
                .bg(p.border)
                .fg(p.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  NUS Internal Shuttle Service",
            Style::default().fg(p.dim),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
