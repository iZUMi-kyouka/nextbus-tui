use fluent::FluentArgs;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

use super::helpers::ellipsis;

pub(super) fn render_list(frame: &mut Frame, area: Rect, app: &mut App) {
    // Track inner height for viewport offset management.
    app.nav.list_height = area.height.saturating_sub(2); // subtract top+bottom borders
                                                         // Clear the ratatui selection so it never auto-scrolls the offset.
    app.nav.list_state.select(None);

    let selected = app.nav.selected;
    let fav_view = app.nav.fav_view;
    let palette = &app.theme().palette;
    let fav_count = app.fav_count_in_list();

    let items: Vec<ListItem> = app
        .nav
        .sorted_indices
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            let stop = &app.domain.stops[idx];
            let is_fav = app.settings.favourites.contains(&stop.name);
            let is_loading = app.fetch.loading.contains(&stop.name);

            let star = if is_fav { "\u{2605} " } else { "  " };
            let spin = if is_loading { " ..." } else { "" };
            let caption_width = (area.width as usize).saturating_sub(10 + spin.len());
            let caption = ellipsis(&stop.caption, caption_width);
            let cursor = if pos == selected { " > " } else { "   " };
            let label = format!("{}{:>2} {}{}{}", cursor, pos + 1, star, caption, spin);

            let style = if pos == selected {
                Style::default()
                    .bg(palette.dim)
                    .add_modifier(Modifier::BOLD)
            } else if is_fav && !fav_view {
                Style::default().fg(palette.highlight)
            } else {
                Style::default()
            };

            let _ = fav_count;
            ListItem::new(label).style(style)
        })
        .collect();

    let count = app.nav.sorted_indices.len();
    let title = if app.nav.fav_view {
        let mut args = FluentArgs::new();
        args.set("count", count as i64);
        format!(" {} ", app.i18n.t_args("panel-favourites", &args))
    } else {
        let mut args = FluentArgs::new();
        args.set("count", count as i64);
        format!(" {} ", app.i18n.t_args("panel-bus-stops", &args))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(palette.border));

    let list = List::new(items).block(block);

    frame.render_stateful_widget(list, area, &mut app.nav.list_state);
}
