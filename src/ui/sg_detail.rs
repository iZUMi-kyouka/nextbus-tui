use fluent::FluentArgs;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::{BusFeature, BusLoad, SgService};
use crate::theme::Palette;

use super::helpers::{
    bus_type_label, col_header, load_color, operator_abbr, pad_right, sg_fmt_arrival,
};

pub(super) fn render_sg_detail(frame: &mut Frame, area: Rect, app: &App) {
    let palette = &app.theme().palette;

    // No stop selected
    let Some(stop) = app.current_sg_stop() else {
        let msg = if app.sg_nav.stops_loading {
            app.i18n.t("sg-detail-loading")
        } else {
            app.i18n.t("detail-no-stops")
        };
        frame.render_widget(
            Paragraph::new(msg).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", app.i18n.t("detail-title"))),
            ),
            area,
        );
        return;
    };

    let stop_code = stop.code.clone();
    let stop_desc = stop.description.clone();
    let is_loading = app.sg_fetch.loading.contains(&stop_code);
    let cached = app.sg_fetch.cache.get(&stop_code);

    let spinner = if is_loading { " ..." } else { "" };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} \u{00b7} {}{} ", stop_code, stop_desc, spinner))
        .border_style(Style::default().fg(palette.detail_border));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    match cached {
        None if is_loading => {
            lines.push(Line::from(Span::styled(
                format!("  {}", app.i18n.t("sg-detail-loading")),
                Style::default().fg(palette.highlight),
            )));
        }
        None => {
            lines.push(Line::from(Span::styled(
                format!("  {}", app.i18n.t("sg-detail-no-data")),
                Style::default().fg(palette.dim),
            )));
        }
        Some(cached) => {
            if let Some(err) = &cached.error {
                let mut args = FluentArgs::new();
                args.set("message", err.as_str());
                lines.push(Line::from(Span::styled(
                    format!("  {}", app.i18n.t_args("sg-detail-error", &args)),
                    Style::default().fg(palette.error),
                )));
                lines.push(Line::from(""));
            }

            if cached.result.services.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", app.i18n.t("sg-detail-no-service")),
                    Style::default().fg(palette.dim),
                )));
            } else {
                render_arrival_table(&mut lines, &cached.result.services, app, palette);
            }

            // Footer
            lines.push(Line::from(""));
            let elapsed = cached.fetched_at.elapsed().as_secs();
            let footer_text = if is_loading {
                format!("  {}", app.i18n.t("detail-refreshing"))
            } else if let Some(secs) = app.sg_seconds_until_refresh() {
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

fn render_arrival_table(
    lines: &mut Vec<Line>,
    services: &[SgService],
    app: &App,
    palette: &Palette,
) {
    // Column widths: Bus=6, Opr=5, Next=8, 2nd=8, Load=4, Type=4
    let bus_w = 6usize;
    let opr_w = 5usize;
    let next_w = 8usize;
    let nd2_w = 8usize;
    let load_w = 4usize;
    let type_w = 4usize;
    let sep_len = bus_w + opr_w + next_w + nd2_w + load_w + type_w + 2;

    // Header
    let header = vec![
        col_header(&app.i18n.t("sg-col-bus"), bus_w),
        col_header(&app.i18n.t("sg-col-opr"), opr_w),
        col_header(&app.i18n.t("sg-col-next"), next_w),
        col_header(&app.i18n.t("sg-col-2nd"), nd2_w),
        col_header(&app.i18n.t("sg-col-load"), load_w),
        col_header(&app.i18n.t("sg-col-type"), type_w),
    ];
    lines.push(Line::from(header));
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(sep_len),
        Style::default().fg(palette.dim),
    )));

    let now = chrono::Local::now();

    for svc in services {
        let (next_text, next_style) = sg_fmt_arrival(&svc.next, now, palette);
        let (nd2_text, nd2_style) = sg_fmt_arrival(&svc.next2, now, palette);

        let lc = svc
            .next
            .as_ref()
            .map(|b| load_color(&b.load, palette))
            .unwrap_or(palette.dim);

        let load_text = svc
            .next
            .as_ref()
            .map(|b| match b.load {
                BusLoad::SeatsAvailable => "SEA",
                BusLoad::StandingAvailable => "SDA",
                BusLoad::LimitedStanding => "LSD",
                BusLoad::Unknown => "   ",
            })
            .unwrap_or("   ");

        let type_text = svc
            .next
            .as_ref()
            .map(|b| bus_type_label(&b.bus_type))
            .unwrap_or("  ");

        let wab_indicator = svc
            .next
            .as_ref()
            .filter(|b| b.feature == BusFeature::WheelchairAccessible)
            .map(|_| " \u{267f}") // ♿
            .unwrap_or("");

        let spans = vec![
            Span::raw(pad_right(&svc.service_no, bus_w)),
            Span::styled(
                pad_right(&operator_abbr(&svc.operator), opr_w),
                Style::default().fg(palette.dim),
            ),
            Span::styled(pad_right(&next_text, next_w), next_style),
            Span::styled(pad_right(&nd2_text, nd2_w), nd2_style),
            Span::styled(
                pad_right(load_text, load_w),
                Style::default().fg(lc).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}{}", type_text, wab_indicator)),
        ];
        lines.push(Line::from(spans));
    }
}
