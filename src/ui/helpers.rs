use ratatui::{
    style::{Modifier, Style},
    text::Span,
};

use crate::models::Route;
use crate::theme::Palette;

pub(super) fn col_header(label: &str, width: usize) -> Span<'static> {
    Span::styled(
        format!("{:<width$}", label),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )
}

pub(super) fn fmt_arrival(t: &str) -> String {
    match t {
        "Arr" => "Arriving".into(),
        "-" | "N.A." | "" => "-".into(),
        t => t
            .parse::<u32>()
            .map(|n| format!("{n} min"))
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

pub(super) fn ellipsis(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else if max_width <= 1 {
        "\u{2026}".to_string()
    } else {
        format!(
            "{}\u{2026}",
            s.chars().take(max_width - 1).collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Route;
    use crate::theme::Palette;
    use ratatui::style::{Color, Modifier};

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
        assert_eq!(fmt_arrival("Arr"), "Arriving");
    }

    #[test]
    fn fmt_arrival_dash() {
        assert_eq!(fmt_arrival("-"), "-");
    }

    #[test]
    fn fmt_arrival_na() {
        assert_eq!(fmt_arrival("N.A."), "-");
    }

    #[test]
    fn fmt_arrival_empty() {
        assert_eq!(fmt_arrival(""), "-");
    }

    #[test]
    fn fmt_arrival_numeric() {
        assert_eq!(fmt_arrival("5"), "5 min");
    }

    #[test]
    fn fmt_arrival_unknown_string() {
        assert_eq!(fmt_arrival("soon"), "soon");
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
