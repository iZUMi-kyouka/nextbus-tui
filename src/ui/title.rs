use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::models::AppMode;
use crate::theme::ThemeMode;

pub(super) fn render_title(frame: &mut Frame, area: Rect, app: &App) {
    let p = &app.theme().palette;
    // On light themes the border colour is typically a dark blue; using the
    // foreground (also dark) as badge text produces poor contrast. Use the
    // background colour instead — it is near-white on light themes and
    // near-black on dark themes, giving readable contrast in both modes.
    let badge_fg = match app.theme().mode {
        ThemeMode::Light => p.background,
        _ => p.foreground,
    };

    // Mode-specific title and switch hint
    let (app_name, switch_hint) = match app.mode {
        AppMode::NusCampus => (
            app.i18n.t("title-app-name"),
            format!(
                "  {}  {}",
                app.i18n.t("title-subtitle"),
                app.i18n.t("title-switch-hint-sg")
            ),
        ),
        AppMode::SgPublicBus => (
            app.i18n.t("title-mode-sg"),
            format!("  {}", app.i18n.t("title-switch-hint-nus")),
        ),
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", app_name),
            Style::default()
                .bg(p.border)
                .fg(badge_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(switch_hint, Style::default().fg(p.dim)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
