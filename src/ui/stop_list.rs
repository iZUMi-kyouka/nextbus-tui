use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

use super::helpers::ellipsis;

pub(super) fn render_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let fav_count = app.fav_count_in_list();

    let items: Vec<ListItem> = app
        .sorted_indices
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            let stop = &app.stops[idx];
            let is_fav = app.favourites.contains(&stop.name);
            let is_loading = app.loading.contains(&stop.name);

            let star = if is_fav { "\u{2605} " } else { "  " }; // ★ or spaces
            let spin = if is_loading { " ..." } else { "" };
            // 2 borders + 3 highlight symbol (" > ") + 2 position + 1 space + 2 star + spin
            let caption_width = (area.width as usize).saturating_sub(10 + spin.len());
            let caption = ellipsis(&stop.caption, caption_width);
            let label = format!("{:>2} {}{}{}", pos + 1, star, caption, spin);

            let style = if is_fav && !app.fav_view {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            // Suppress unused-variable warning from the intentionally empty separator block.
            let _ = fav_count;

            ListItem::new(label).style(style)
        })
        .collect();

    let title = if app.fav_view {
        format!(" \u{2605} Favourites ({}) ", app.sorted_indices.len())
    } else {
        format!(" Bus Stops ({}) ", app.sorted_indices.len())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Blue));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}
