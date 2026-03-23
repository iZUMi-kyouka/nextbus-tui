use fluent::FluentArgs;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

use super::helpers::ellipsis;

pub(super) fn render_sg_list(frame: &mut Frame, area: Rect, app: &mut App) {
    app.sg_nav.list_height = area.height.saturating_sub(2);
    // Clear the ratatui selection so it never auto-scrolls the offset.
    app.sg_nav.list_state.select(None);

    let palette = &app.theme().palette;

    // Show loading state when stops aren't available yet
    if app.sg_nav.stops_loading && app.domain.sg_stops.is_empty() {
        let progress = app.sg_nav.stops_load_progress;
        let mut args = FluentArgs::new();
        args.set("count", progress as i64);
        let msg = app.i18n.t_args("sg-stops-loading", &args);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", app.i18n.t("sg-panel-stops-title")))
            .border_style(Style::default().fg(palette.border));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("  {}", msg),
                Style::default().fg(palette.highlight),
            ))),
            Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 1,
            },
        );
        return;
    }

    if let Some(err) = &app.sg_nav.stops_error.clone() {
        let mut args = FluentArgs::new();
        args.set("message", err.as_str());
        let msg = app.i18n.t_args("sg-stops-error", &args);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", app.i18n.t("sg-panel-stops-title")))
            .border_style(Style::default().fg(palette.border));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("  {}", msg),
                Style::default().fg(palette.error),
            ))),
            Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 1,
            },
        );
        return;
    }

    let selected = app.sg_nav.selected;
    let fav_view = app.sg_nav.fav_view;

    let items: Vec<ListItem> = app
        .sg_nav
        .sorted_indices
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            let stop = &app.domain.sg_stops[idx];
            let is_fav = app.settings.sg_favourites.contains(&stop.code);
            let is_loading = app.sg_fetch.loading.contains(&stop.code);

            let star = if is_fav { "\u{2605} " } else { "  " };
            let spin = if is_loading { " ..." } else { "" };
            let desc_width = (area.width as usize).saturating_sub(14 + spin.len());
            let desc = ellipsis(&stop.description, desc_width);
            let cursor = if pos == selected { " > " } else { "   " };
            let label = format!(
                "{}{:>3} {} {:5} {}{}",
                cursor,
                pos + 1,
                star,
                stop.code,
                desc,
                spin
            );

            let style = if pos == selected {
                Style::default()
                    .bg(palette.dim)
                    .add_modifier(Modifier::BOLD)
            } else if is_fav && !fav_view {
                Style::default().fg(palette.highlight)
            } else {
                Style::default()
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let count = app.sg_nav.sorted_indices.len();
    let title = if app.sg_nav.fav_view {
        let mut args = FluentArgs::new();
        args.set("count", count as i64);
        format!(" {} ", app.i18n.t_args("sg-panel-favs", &args))
    } else {
        let mut args = FluentArgs::new();
        args.set("count", count as i64);
        format!(" {} ", app.i18n.t_args("sg-panel-stops", &args))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(palette.border));

    let list = List::new(items).block(block);

    frame.render_stateful_widget(list, area, &mut app.sg_nav.list_state);
}
