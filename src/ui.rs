use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AUTO_REFRESH_SECS};

// ── Entry point ───────────────────────────────────────────────────────────────

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

// ── Title bar ─────────────────────────────────────────────────────────────────

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            " NUS NextBus TUI ",
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  NUS Internal Shuttle Service",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

// ── Two-panel layout ──────────────────────────────────────────────────────────

fn render_panels(frame: &mut Frame, area: Rect, app: &mut App) {
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

// ── Stop list (left panel) ────────────────────────────────────────────────────

fn render_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let fav_count = app.fav_count_in_list();

    let items: Vec<ListItem> = app
        .sorted_indices
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            let stop = &app.stops[idx];
            let is_fav = app.favourites.contains(&stop.name);
            let is_loading = app.loading.contains(&stop.name);

            // Visual separator line between favourites and the rest
            if pos == fav_count && fav_count > 0 {
                // This item is the first non-favourite after some favourites.
                // We handle this by inserting spacing via label padding; a true
                // separator would need a custom widget. Keep it simple.
            }

            let star = if is_fav { "\u{2605} " } else { "  " }; // ★ or spaces
            let spin = if is_loading { " ..." } else { "" };
            // 2 borders + 2 highlight symbol + 2 star + spin length
            let caption_width = (area.width as usize)
                .saturating_sub(6 + spin.len());
            let caption = ellipsis(&stop.caption, caption_width);
            let label = format!("{}{}{}", star, caption, spin);

            let style = if is_fav {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let title = format!(" Bus Stops ({}) ", app.sorted_indices.len());

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
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

// ── Stop detail (right panel) ─────────────────────────────────────────────────

fn render_detail(frame: &mut Frame, area: Rect, app: &App, show_plate: bool) {
    let Some(stop) = app.current_stop() else {
        frame.render_widget(
            Paragraph::new("No stops to display.")
                .block(Block::default().borders(Borders::ALL).title(" Details ")),
            area,
        );
        return;
    };

    let stop_name = stop.name.clone();
    let caption = stop.caption.clone();
    let is_loading = app.loading.contains(&stop_name);
    let cached = app.cache.get(&stop_name);

    let spinner = if is_loading { " ..." } else { "" };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {}{} ", caption, spinner))
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    match cached {
        None if is_loading => {
            lines.push(Line::from(Span::styled(
                "  Loading...",
                Style::default().fg(Color::Yellow),
            )));
        }
        None => {
            lines.push(Line::from(Span::styled(
                "  No data yet.  Press [r] to fetch.",
                Style::default().fg(Color::DarkGray),
            )));
        }
        Some(cached) => {
            // Error banner (shown even when we have stale data underneath)
            if let Some(err) = &cached.error {
                lines.push(Line::from(Span::styled(
                    format!("  ! {}", err),
                    Style::default().fg(Color::Red),
                )));
                lines.push(Line::from(""));
            }

            if cached.result.shuttles.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  No buses currently in service.",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                // ── Column header ──
                let mut header = vec![
                    col_header("Bus", 10),
                    col_header("Next", 12),
                    col_header("Following", 12),
                ];
                if show_plate {
                    header.push(col_header("Plate", 12));
                }
                lines.push(Line::from(header));
                lines.push(Line::from(Span::styled(
                    "\u{2500}".repeat(if show_plate { 46 } else { 34 }),
                    Style::default().fg(Color::DarkGray),
                )));

                // ── One row per shuttle entry ──
                // The API returns one entry per direction for routes that serve a stop
                // in both directions (e.g. D1 as COM3-D1-E and COM3-D1-S). When
                // duplicates exist, prefer the departing entry (busstopcode ends with "-S").
                let mut deduped: Vec<&crate::models::Shuttle> = Vec::new();
                for s in &cached.result.shuttles {
                    let is_departing = s.busstopcode.as_deref().map(|c| c.ends_with("-S")).unwrap_or(false);
                    if let Some(existing) = deduped.iter_mut().find(|e| e.name == s.name) {
                        if is_departing {
                            *existing = s;
                        }
                    } else {
                        deduped.push(s);
                    }
                }
                for s in deduped {
                    let next_text = fmt_arrival(&s.arrival_time);
                    let following_text = fmt_arrival(&s.next_arrival_time);
                    let plate = s.arrival_plate.as_deref().unwrap_or("-");

                    let name_spans: [Span; 2] = match route_color(&s.name, &app.routes) {
                        Some(color) => [
                            Span::styled(
                                format!("{:<5}", s.name),
                                Style::default().bg(color).fg(Color::White).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw("     "),
                        ],
                        None => [Span::raw(format!("{:<10}", s.name)), Span::raw("")],
                    };

                    let mut row = vec![
                        name_spans[0].clone(),
                        name_spans[1].clone(),
                        Span::styled(
                            format!("{:<12}", next_text),
                            arrival_style(&s.arrival_time),
                        ),
                        Span::styled(
                            format!("{:<12}", following_text),
                            arrival_style(&s.next_arrival_time),
                        ),
                    ];
                    if show_plate {
                        row.push(Span::styled(
                            format!("{:<12}", plate),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    lines.push(Line::from(row));
                }
            }

            // ── Refresh countdown ──
            lines.push(Line::from(""));
            let elapsed = cached.fetched_at.elapsed().as_secs();
            let footer_text = if is_loading {
                "  Refreshing...".to_string()
            } else if let Some(secs) = app.seconds_until_refresh() {
                format!(
                    "  Last: {}s ago   Auto-refresh in: {}s / {}s",
                    elapsed, secs, AUTO_REFRESH_SECS
                )
            } else {
                format!("  Last fetched: {}s ago", elapsed)
            };
            lines.push(Line::from(Span::styled(
                footer_text,
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    frame.render_widget(
        Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
        inner,
    );
}

// ── Search overlay (Spotlight-style) ─────────────────────────────────────────

fn render_search_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let width = (area.width * 50 / 100).max(40).min(area.width.saturating_sub(4));
    let popup = Rect {
        x: (area.width - width) / 2,
        y: area.height / 4,
        width,
        height: 3,
    };
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🔍 Search ")
        .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    let input = Paragraph::new(Line::from(vec![
        Span::raw(&app.search_query[..]),
        Span::styled("█", Style::default().fg(Color::Yellow)),
    ]))
    .block(block);
    frame.render_widget(input, popup);
}

// ── Status / key-hint bar ─────────────────────────────────────────────────────

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let span = if let Some((msg, _)) = &app.status_msg {
        Span::styled(
            format!("  {} ", msg),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )
    } else if app.searching {
        Span::styled(
            "  [Type] filter   [Up/Down] navigate   [Enter] confirm   [Esc] cancel",
            Style::default().fg(Color::Yellow),
        )
    } else {
        Span::styled(
            "  [j/k] Move   [f] Favourite   [r] Refresh   [/] Search   [g/G] Top/Bot   [q] Quit",
            Style::default().fg(Color::DarkGray),
        )
    };

    frame.render_widget(Paragraph::new(Line::from(span)), area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn col_header(label: &str, width: usize) -> Span<'static> {
    Span::styled(
        format!("{:<width$}", label),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )
}

fn fmt_arrival(t: &str) -> String {
    match t {
        "Arr" => "Arriving".into(),
        "-" | "N.A." | "" => "-".into(),
        t => t
            .parse::<u32>()
            .map(|n| format!("{n} min"))
            .unwrap_or_else(|_| t.into()),
    }
}

fn route_color(name: &str, routes: &[crate::models::Route]) -> Option<Color> {
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

fn ellipsis(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else if max_width <= 1 {
        "…".to_string()
    } else {
        format!("{}…", s.chars().take(max_width - 1).collect::<String>())
    }
}

fn arrival_style(t: &str) -> Style {
    match t {
        "Arr" => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        "-" | "N.A." | "" => Style::default().fg(Color::DarkGray),
        t => {
            if t.parse::<u32>().map(|n| n <= 3).unwrap_or(false) {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        }
    }
}
