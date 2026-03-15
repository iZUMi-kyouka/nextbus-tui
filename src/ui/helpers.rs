use fluent::FluentArgs;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::i18n::I18n;
use crate::models::Route;
use crate::theme::Palette;

/// Center a popup of at most `max_w × max_h` within `area`, clamping to terminal edges.
pub(super) fn centered_popup(area: Rect, max_w: u16, max_h: u16) -> Rect {
    let w = max_w.min(area.width.saturating_sub(4));
    let h = max_h.min(area.height.saturating_sub(2));
    Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    }
}

/// Build the standard popup Block (highlighted bold border, theme background/foreground).
/// The caller is responsible for `frame.render_widget(Clear, popup)` before rendering.
pub(super) fn popup_block(title: String, palette: &Palette) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        )
        .style(
            Style::default()
                .bg(palette.background)
                .fg(palette.foreground),
        )
}

/// Render a dim hint line at the very last row of `inner`.
pub(super) fn render_hint_row(frame: &mut Frame, inner: Rect, text: &str, palette: &Palette) {
    let y = inner.y + inner.height.saturating_sub(1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {}", text),
            Style::default().fg(palette.dim),
        ))),
        Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        },
    );
}

/// Pad `s` to exactly `display_width` terminal columns using spaces.
/// Unlike `format!("{:<width$}", s)`, this counts Unicode display width
/// rather than codepoint count, so CJK double-width characters are handled
/// correctly.
pub(super) fn pad_right(s: &str, display_width: usize) -> String {
    let w = s.width();
    if w >= display_width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(display_width - w))
    }
}

pub(super) fn col_header(label: &str, width: usize) -> Span<'static> {
    Span::styled(
        pad_right(label, width),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )
}

/// Format an arrival time string for display, translating special values.
pub(super) fn fmt_arrival(t: &str, i18n: &I18n) -> String {
    match t {
        "Arr" => i18n.t("arrival-arriving"),
        "-" | "N.A." | "" => "-".into(),
        t => t
            .parse::<u32>()
            .map(|n| {
                let mut args = FluentArgs::new();
                args.set("minutes", n as i64);
                i18n.t_args("arrival-minutes", &args)
            })
            .unwrap_or_else(|_| t.into()),
    }
}

pub(super) fn arrival_style(t: &str, palette: &Palette) -> Style {
    match t {
        "Arr" => Style::default()
            .fg(palette.success)
            .add_modifier(Modifier::BOLD),
        "-" | "N.A." | "" => Style::default().fg(palette.dim),
        t => {
            if t.parse::<u32>().map(|n| n <= 3).unwrap_or(false) {
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        }
    }
}

pub(super) fn route_color(name: &str, routes: &[Route]) -> Option<ratatui::style::Color> {
    use ratatui::style::Color;
    let hex = &routes.iter().find(|r| r.name == name)?.color;
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

/// Truncate `s` to at most `max_cols` terminal columns, appending `…` if cut.
/// Uses Unicode display width, so CJK double-width characters count as 2.
pub(super) fn ellipsis(s: &str, max_cols: usize) -> String {
    if s.width() <= max_cols {
        return s.to_string();
    }
    if max_cols <= 1 {
        return "\u{2026}".to_string();
    }
    let target = max_cols - 1; // reserve 1 col for '…'
    let mut w = 0usize;
    let mut result = String::new();
    for g in s.graphemes(true) {
        let gw = g.width();
        if w + gw > target {
            break;
        }
        result.push_str(g);
        w += gw;
    }
    format!("{}\u{2026}", result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::I18n;
    use crate::models::Route;
    use crate::theme::Palette;
    use ratatui::style::{Color, Modifier};

    fn en() -> I18n {
        I18n::new("en")
    }

    fn test_palette() -> Palette {
        Palette {
            background: Color::Black,
            foreground: Color::White,
            border: Color::Blue,
            detail_border: Color::Cyan,
            highlight: Color::Yellow,
            success: Color::Green,
            error: Color::Red,
            dim: Color::DarkGray,
            jump: Color::Cyan,
            swatch: [
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue,
                Color::Magenta,
            ],
        }
    }

    fn make_route(name: &str, color: &str) -> Route {
        Route {
            name: name.to_string(),
            color: color.to_string(),
            stops: vec![],
        }
    }

    // ── fmt_arrival ────────────────────────────────────────────────────────────

    #[test]
    fn fmt_arrival_arr() {
        assert_eq!(fmt_arrival("Arr", &en()), "Arriving");
    }

    #[test]
    fn fmt_arrival_dash() {
        assert_eq!(fmt_arrival("-", &en()), "-");
    }

    #[test]
    fn fmt_arrival_na() {
        assert_eq!(fmt_arrival("N.A.", &en()), "-");
    }

    #[test]
    fn fmt_arrival_empty() {
        assert_eq!(fmt_arrival("", &en()), "-");
    }

    #[test]
    fn fmt_arrival_numeric() {
        assert_eq!(fmt_arrival("5", &en()), "5 min");
    }

    #[test]
    fn fmt_arrival_unknown_string() {
        assert_eq!(fmt_arrival("soon", &en()), "soon");
    }

    #[test]
    fn fmt_arrival_japanese_arr() {
        let ja = I18n::new("ja");
        assert_eq!(fmt_arrival("Arr", &ja), "まもなく");
    }

    #[test]
    fn fmt_arrival_japanese_minutes() {
        let ja = I18n::new("ja");
        assert_eq!(fmt_arrival("5", &ja), "5分");
    }

    // ── ellipsis ───────────────────────────────────────────────────────────────

    #[test]
    fn ellipsis_within_width() {
        assert_eq!(ellipsis("hello", 10), "hello");
    }

    #[test]
    fn ellipsis_exact_width() {
        assert_eq!(ellipsis("hello", 5), "hello");
    }

    #[test]
    fn ellipsis_truncates() {
        assert_eq!(ellipsis("hello world", 6), "hello\u{2026}");
    }

    #[test]
    fn ellipsis_max_width_one() {
        assert_eq!(ellipsis("hello", 1), "\u{2026}");
    }

    #[test]
    fn ellipsis_max_width_zero() {
        assert_eq!(ellipsis("hello", 0), "\u{2026}");
    }

    #[test]
    fn ellipsis_empty_input() {
        assert_eq!(ellipsis("", 5), "");
    }

    #[test]
    fn ellipsis_cjk_double_width() {
        // Each kanji = 2 display cols; "日本語" = 6 cols.
        // max_cols=5: target=4, "日"(2)+"本"(2)=4 fits, "語" would exceed → "日本…"
        assert_eq!(ellipsis("日本語", 5), "日本\u{2026}");
    }

    #[test]
    fn ellipsis_cjk_tight_boundary() {
        // max_cols=4: target=3; "日"(2)≤3 fits, "日本"(4)>3 → only "日" fits → "日…"
        assert_eq!(ellipsis("日本語", 4), "日\u{2026}");
    }

    #[test]
    fn ellipsis_tamil_graphemes() {
        // Width-aware truncation should preserve the full string when it fits.
        assert_eq!(ellipsis("தமிழ்", 4), "தமிழ்");
    }

    #[test]
    fn ellipsis_tamil_tight_boundary() {
        assert_eq!(ellipsis("தமிழ்", 3), "த\u{2026}");
    }

    // ── pad_right ──────────────────────────────────────────────────────────────

    #[test]
    fn pad_right_ascii() {
        assert_eq!(pad_right("Bus", 10), "Bus       ");
    }

    #[test]
    fn pad_right_cjk() {
        // "バス" = 4 display cols → pad to 10 → 6 spaces appended
        let result = pad_right("バス", 10);
        assert_eq!(result.width(), 10);
    }

    #[test]
    fn pad_right_already_wide() {
        assert_eq!(pad_right("hello", 3), "hello");
    }

    // ── route_color ────────────────────────────────────────────────────────────

    #[test]
    fn route_color_found() {
        let routes = vec![make_route("D1", "#fe0000")];
        assert_eq!(
            route_color("D1", &routes),
            Some(Color::Rgb(0xfe, 0x00, 0x00))
        );
    }

    #[test]
    fn route_color_not_found() {
        let routes = vec![make_route("D1", "#fe0000")];
        assert_eq!(route_color("A2", &routes), None);
    }

    #[test]
    fn route_color_empty_routes() {
        assert_eq!(route_color("D1", &[]), None);
    }

    // ── arrival_style ──────────────────────────────────────────────────────────

    #[test]
    fn arrival_style_arr_bold_success() {
        let p = test_palette();
        let s = arrival_style("Arr", &p);
        assert_eq!(s.fg, Some(Color::Green));
        assert!(s.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn arrival_style_dash_dim() {
        let p = test_palette();
        let s = arrival_style("-", &p);
        assert_eq!(s.fg, Some(Color::DarkGray));
    }

    #[test]
    fn arrival_style_urgent_leq3_bold_highlight() {
        let p = test_palette();
        let s = arrival_style("3", &p);
        assert_eq!(s.fg, Some(Color::Yellow));
        assert!(s.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn arrival_style_normal_gt3_no_override() {
        let p = test_palette();
        let s = arrival_style("10", &p);
        assert!(!s.add_modifier.contains(Modifier::BOLD));
        assert_eq!(s.fg, None);
    }
}
