use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

use crate::models::Route;

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

pub(super) fn arrival_style(t: &str) -> Style {
    match t {
        "Arr" => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        "-" | "N.A." | "" => Style::default().fg(Color::DarkGray),
        t => {
            if t.parse::<u32>().map(|n| n <= 3).unwrap_or(false) {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        }
    }
}

pub(super) fn route_color(name: &str, routes: &[Route]) -> Option<Color> {
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
        format!("{}\u{2026}", s.chars().take(max_width - 1).collect::<String>())
    }
}
