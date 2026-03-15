use fluent::FluentArgs;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use crate::i18n::I18n;
use crate::models::Shuttle;
use crate::theme::Palette;

use super::helpers::{arrival_style, col_header, fmt_arrival, pad_right, route_color};

pub(super) fn render_detail(frame: &mut Frame, area: Rect, app: &App, show_plate: bool) {
    let palette = &app.theme().palette;

    let Some(stop) = app.current_stop() else {
        frame.render_widget(
            Paragraph::new(app.i18n.t("detail-no-stops")).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", app.i18n.t("detail-title"))),
            ),
            area,
        );
        return;
    };

    let stop_name = stop.name.clone();
    let caption = stop.caption.clone();
    let is_loading = app.fetch.loading.contains(&stop_name);
    let cached = app.fetch.cache.get(&stop_name);

    let spinner = if is_loading { " ..." } else { "" };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {}{} ", caption, spinner))
        .border_style(Style::default().fg(palette.detail_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    match cached {
        None if is_loading => {
            lines.push(Line::from(Span::styled(
                format!("  {}", app.i18n.t("detail-loading")),
                Style::default().fg(palette.highlight),
            )));
        }
        None => {
            lines.push(Line::from(Span::styled(
                format!("  {}", app.i18n.t("detail-no-data")),
                Style::default().fg(palette.dim),
            )));
        }
        Some(cached) => {
            if let Some(err) = &cached.error {
                let mut args = FluentArgs::new();
                args.set("message", err.as_str());
                lines.push(Line::from(Span::styled(
                    format!("  {}", app.i18n.t_args("detail-error", &args)),
                    Style::default().fg(palette.error),
                )));
                lines.push(Line::from(""));
            }

            if cached.result.shuttles.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", app.i18n.t("detail-no-buses")),
                    Style::default().fg(palette.dim),
                )));
            } else {
                render_shuttle_table(
                    &mut lines,
                    &cached.result.shuttles,
                    app,
                    show_plate,
                    palette,
                );
            }

            // Refresh countdown footer.
            lines.push(Line::from(""));
            let elapsed = cached.fetched_at.elapsed().as_secs();
            let footer_text = if is_loading {
                format!("  {}", app.i18n.t("detail-refreshing"))
            } else if let Some(secs) = app.seconds_until_refresh() {
                let mut args = FluentArgs::new();
                args.set("elapsed", elapsed as i64);
                args.set("remaining", secs as i64);
                args.set("total", app.settings.auto_refresh_secs as i64);
                format!("  {}", app.i18n.t_args("detail-last-refreshed", &args))
            } else {
                let mut args = FluentArgs::new();
                args.set("elapsed", elapsed as i64);
                format!("  {}", app.i18n.t_args("detail-last-fetched", &args))
            };
            lines.push(Line::from(Span::styled(
                footer_text,
                Style::default().fg(palette.dim),
            )));
        }
    }

    frame.render_widget(
        Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
        inner,
    );
}

// ── Shuttle table ─────────────────────────────────────────────────────────────

fn render_shuttle_table(
    lines: &mut Vec<Line>,
    shuttles: &[Shuttle],
    app: &App,
    show_plate: bool,
    palette: &Palette,
) {
    // Compact column widths when narrow (no plate column):
    //   Next      = 9  ("Arriving" + 1)
    //   Following = 10 ("Following" header + 1)
    // Full widths when wide (plate shown):
    //   Next = Following = Plate = 12
    let (next_w, foll_w) = if show_plate { (12, 12) } else { (9, 10) };
    let sep_len = 10 + next_w + foll_w + if show_plate { 12 } else { 0 };

    // Column header row.
    let mut header = vec![
        col_header(&app.i18n.t("col-bus"), 10),
        col_header(&app.i18n.t("col-next"), next_w),
        col_header(&app.i18n.t("col-following"), foll_w),
    ];
    if show_plate {
        header.push(col_header(&app.i18n.t("col-plate"), 12));
    }
    lines.push(Line::from(header));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(sep_len),
        Style::default().fg(palette.dim),
    )));

    // Deduplicate: the API returns one entry per direction for routes that serve
    // a stop in both directions (e.g. D1 as COM3-D1-E and COM3-D1-S).
    // Prefer the departing entry (busstopcode ends with "-S").
    let mut deduped: Vec<&Shuttle> = Vec::new();
    for s in shuttles {
        let is_departing = s
            .busstopcode
            .as_deref()
            .map(|c| c.ends_with("-S"))
            .unwrap_or(false);
        if let Some(existing) = deduped.iter_mut().find(|e| e.name == s.name) {
            if is_departing {
                *existing = s;
            }
        } else {
            deduped.push(s);
        }
    }

    for s in deduped {
        lines.push(shuttle_row(s, app, show_plate, next_w, foll_w, palette));
    }
}

fn shuttle_row(
    s: &Shuttle,
    app: &App,
    show_plate: bool,
    next_w: usize,
    foll_w: usize,
    palette: &Palette,
) -> Line<'static> {
    let i18n: &I18n = &app.i18n;
    let next_text = fmt_arrival(&s.arrival_time, i18n);
    let following_text = fmt_arrival(&s.next_arrival_time, i18n);
    let plate = s.arrival_plate.as_deref().unwrap_or("-");
    let display_name = s.name.strip_prefix("PUB:").unwrap_or(&s.name);

    let name_spans: [Span; 2] = match route_color(&s.name, &app.domain.routes) {
        Some(color) => [
            Span::styled(
                format!("{:<5}", display_name),
                Style::default()
                    .bg(color)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("     "),
        ],
        None => [Span::raw(format!("{:<10}", display_name)), Span::raw("")],
    };

    let mut spans = vec![
        name_spans[0].clone(),
        name_spans[1].clone(),
        Span::styled(
            pad_right(&next_text, next_w),
            arrival_style(&s.arrival_time, palette),
        ),
        Span::styled(
            pad_right(&following_text, foll_w),
            arrival_style(&s.next_arrival_time, palette),
        ),
    ];

    if show_plate {
        spans.push(Span::styled(
            format!("{:<12}", plate),
            Style::default().fg(palette.dim),
        ));
    }

    Line::from(spans)
}
