use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub(super) fn render_title(frame: &mut Frame, area: Rect, app: &App) {
    let p = &app.theme().palette;
    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.i18n.t("title-app-name")),
            Style::default()
                .bg(p.border)
                .fg(p.foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}", app.i18n.t("title-subtitle")),
            Style::default().fg(p.dim),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
