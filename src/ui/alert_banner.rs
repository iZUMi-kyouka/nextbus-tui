use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

/// Amber highlight bar shown across the full width when an MRT disruption is active.
/// Dismissed by pressing `d`.
pub fn render_alert_banner(frame: &mut Frame, area: Rect, app: &App) {
    let bg = Color::Rgb(160, 90, 0);
    let fg = Color::Rgb(255, 240, 180);
    let accent = Color::Rgb(255, 220, 60);

    let icon_style = Style::default()
        .fg(accent)
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD);
    let body_style = Style::default().fg(fg).bg(bg);
    let hint_style = Style::default()
        .fg(accent)
        .bg(bg)
        .add_modifier(Modifier::BOLD);

    let spans = vec![
        Span::styled(" ⚠ ", icon_style),
        Span::styled("MRT DISRUPTION: ", label_style),
        Span::styled(app.train_alert.summary.as_str(), body_style),
        Span::styled("   [d] Dismiss ", hint_style),
    ];

    let para = Paragraph::new(Line::from(spans)).style(Style::default().bg(bg));
    frame.render_widget(para, area);
}
