mod detail;
mod footer;
mod helpers;
mod search;
mod stop_list;
mod title;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

use detail::render_detail;
use footer::render_footer;
use search::render_search_overlay;
use stop_list::render_list;
use title::render_title;

pub fn render(frame: &mut Frame, app: &mut App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(0),    // main panels
            Constraint::Length(1), // status / key hints
        ])
        .split(frame.area());

    render_title(frame, root[0]);
    render_panels(frame, root[1], app);
    render_footer(frame, root[2], app);

    if app.searching {
        render_search_overlay(frame, app);
    }
}

fn render_panels(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let narrow = area.width < 100;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if narrow {
            [Constraint::Percentage(40), Constraint::Percentage(60)]
        } else {
            [Constraint::Percentage(33), Constraint::Percentage(67)]
        })
        .split(area);

    render_list(frame, cols[0], app);
    render_detail(frame, cols[1], app, !narrow);
}
