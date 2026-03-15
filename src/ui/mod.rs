pub(crate) mod detail;
pub(crate) mod footer;
pub(crate) mod helpers;
pub(crate) mod search;
pub(crate) mod settings;
pub(crate) mod stop_list;
pub(crate) mod theme_picker;
pub(crate) mod title;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
};

use crate::app::App;

use detail::render_detail;
use footer::render_footer;
use search::render_search_overlay;
use settings::render_settings;
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

    if app.showing_settings {
        render_settings(frame, app);
    }
}

fn render_panels(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    let narrow = area.width < 100;
    let very_narrow = area.width < 70;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if very_narrow {
            [Constraint::Percentage(50), Constraint::Percentage(50)]
        } else if narrow {
            [Constraint::Percentage(40), Constraint::Percentage(60)]
        } else {
            [Constraint::Percentage(33), Constraint::Percentage(67)]
        })
        .split(area);

    render_list(frame, cols[0], app);
    render_detail(frame, cols[1], app, !narrow);
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::app::App;
    use ratatui::{Terminal, backend::TestBackend};
    use std::sync::mpsc;

    fn make_app() -> App {
        let (tx, _rx) = mpsc::channel();
        let mut app = App::new(tx);
        app.favourites.clear();
        app.fav_view = false;
        app.theme_mode = crate::theme::ThemeMode::Dark;
        app.theme_idx = 0;
        app.i18n = crate::i18n::I18n::new("en");
        app.rebuild_list();
        app
    }

    fn make_terminal(w: u16, h: u16) -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(w, h)).unwrap()
    }

    /// Concatenate all cell symbols in the buffer into a single string.
    fn buf_text(terminal: &Terminal<TestBackend>) -> String {
        let buf = terminal.backend().buffer();
        let area = buf.area;
        (area.top()..area.bottom())
            .flat_map(|y| (area.left()..area.right()).map(move |x| (x, y)))
            .map(|(x, y)| buf[(x, y)].symbol().to_string())
            .collect()
    }

    #[test]
    fn render_does_not_panic() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        terminal.draw(|f| render(f, &mut app)).unwrap();
    }

    #[test]
    fn render_narrow_does_not_panic() {
        let mut terminal = make_terminal(60, 20);
        let mut app = make_app();
        terminal.draw(|f| render(f, &mut app)).unwrap();
    }

    #[test]
    fn render_contains_app_title() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let text = buf_text(&terminal);
        assert!(
            text.contains("NUS NextBus"),
            "Title should contain 'NUS NextBus'"
        );
    }

    #[test]
    fn render_contains_bus_stops_header() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let text = buf_text(&terminal);
        assert!(
            text.contains("Bus Stops"),
            "Should contain 'Bus Stops' header"
        );
    }

    #[test]
    fn render_search_overlay_when_searching() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        app.searching = true;
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let text = buf_text(&terminal);
        assert!(text.contains("Search"), "Search overlay should be visible");
    }

    #[test]
    fn render_theme_picker_shows_default_theme() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        app.showing_theme_picker = true;
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let text = buf_text(&terminal);
        assert!(
            text.contains("Default"),
            "Theme picker should list 'Default' theme"
        );
    }

    #[test]
    fn render_fav_view_shows_favourites_title() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        app.fav_view = true;
        app.rebuild_list();
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let text = buf_text(&terminal);
        assert!(
            text.contains("Favourites"),
            "Fav view should show 'Favourites' title"
        );
    }

    #[test]
    fn render_footer_quit_hint() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let text = buf_text(&terminal);
        assert!(text.contains("Quit"), "Footer should show 'Quit' hint");
    }

    #[test]
    fn render_footer_search_mode_cancel_hint() {
        let mut terminal = make_terminal(120, 30);
        let mut app = make_app();
        app.searching = true;
        terminal.draw(|f| render(f, &mut app)).unwrap();
        let text = buf_text(&terminal);
        assert!(
            text.contains("Cancel"),
            "Search footer should show 'Cancel'"
        );
    }

    #[test]
    fn render_all_themes_without_panic() {
        let (tx, _rx) = mpsc::channel();
        let mut app = App::new(tx);
        let n = app.themes.len();
        for i in 0..n {
            app.theme_idx = i;
            let mut terminal = make_terminal(120, 30);
            terminal.draw(|f| render(f, &mut app)).unwrap();
        }
    }
}
