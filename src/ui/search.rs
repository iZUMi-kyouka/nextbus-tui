use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

pub(super) fn render_search_overlay(frame: &mut Frame, app: &App) {
    let p = &app.theme().palette;
    let area = frame.area();
    let width = (area.width * 50 / 100)
        .max(40)
        .min(area.width.saturating_sub(4));
    let popup = Rect {
        x: (area.width - width) / 2,
        y: area.height / 4,
        width,
        height: 3,
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" \u{1F50D} Search ")
        .border_style(
            Style::default()
                .fg(p.highlight)
                .add_modifier(Modifier::BOLD),
        );

    let input = Paragraph::new(Line::from(vec![
        Span::raw(&app.search_query[..]),
        Span::styled("\u{2588}", Style::default().fg(p.highlight)),
    ]))
    .block(block);

    frame.render_widget(input, popup);
}
