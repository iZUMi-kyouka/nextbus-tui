pub(crate) mod detail;
pub(crate) mod footer;
pub(crate) mod helpers;
pub(crate) mod search;
pub(crate) mod stop_list;
pub(crate) mod theme_picker;
pub(crate) mod title;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    Frame,
};

use crate::app::App;

use detail::render_detail;
use footer::render_footer;
use search::render_search_overlay;
use stop_list::render_list;
use theme_picker::render_theme_picker;
use title::render_title;

pub fn render(frame: &mut Frame, app: &mut App) {
    // Fill the entire terminal with the active theme's background and foreground.
    // Widgets that use Style::default() (no explicit bg/fg) will inherit these.
    {
        let p = &app.theme().palette;
        let base = Style::default().fg(p.foreground).bg(p.background);
        let area = frame.area();
        frame.buffer_mut().set_style(area, base);
    }

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(0),    // main panels
            Constraint::Length(1), // status / key hints
        ])
        .split(frame.area());

    render_title(frame, root[0], app);
    render_panels(frame, root[1], app);
    render_footer(frame, root[2], app);

    if app.searching {
        render_search_overlay(frame, app);
    }

    if app.showing_theme_picker {
        render_theme_picker(frame, app);
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
