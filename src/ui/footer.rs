use fluent::FluentArgs;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub(super) fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let p = &app.theme().palette;

    let span = if !app.jump_buf.is_empty() {
        let mut args = FluentArgs::new();
        args.set("digits", app.jump_buf.as_str());
        Span::styled(
            format!("  {}", app.i18n.t_args("footer-jump", &args)),
            Style::default().fg(p.jump).add_modifier(Modifier::BOLD),
        )
    } else if let Some((msg, _)) = &app.status_msg {
        Span::styled(
            format!("  {} ", msg),
            Style::default().fg(p.success).add_modifier(Modifier::BOLD),
        )
    } else if app.showing_settings {
        let hint_key = if app.settings_edit_mode {
            "footer-settings-edit"
        } else {
            "footer-settings-nav"
        };
        Span::styled(
            format!("  {}", app.i18n.t(hint_key)),
            Style::default().fg(p.highlight),
        )
    } else if app.searching {
        Span::styled(
            format!("  {}", app.i18n.t("footer-search")),
            Style::default().fg(p.highlight),
        )
    } else {
        Span::styled(
            format!("  {}", app.i18n.t("footer-normal")),
            Style::default().fg(p.dim),
        )
    };

    frame.render_widget(Paragraph::new(Line::from(span)), area);
}
