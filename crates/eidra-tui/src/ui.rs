use std::collections::HashMap;
use std::f64::consts::PI;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Circle, Line as CanvasLine, Points, Rectangle};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::app::TuiApp;
use crate::event::{RequestAction, RequestEntry};

pub fn render(frame: &mut Frame, app: &TuiApp) {
    frame.render_widget(
        Block::default().style(Style::default().bg(bg())),
        frame.area(),
    );

    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(26),
            Constraint::Percentage(46),
            Constraint::Percentage(28),
        ])
        .split(shell[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(12)])
        .split(main[0]);

    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(16), Constraint::Length(9)])
        .split(main[1]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(13), Constraint::Min(11)])
        .split(main[2]);

    let rail = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(30),
            Constraint::Percentage(36),
        ])
        .split(shell[2]);

    render_header(frame, app, shell[0]);
    render_trust_summary(frame, app, left[0]);
    render_channel_ledger(frame, app, left[1]);
    render_boundary_field(frame, app, center[0]);
    render_decision_flow(frame, app, center[1]);
    render_guardian_radar(frame, app, right[0]);
    render_live_stream(frame, app, right[1]);
    render_sensitive_surface(frame, app, rail[0]);
    render_response_channels(frame, app, rail[1]);
    render_policy_matrix(frame, app, rail[2]);
    render_footer(frame, app, shell[3]);
}

fn render_header(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let containment = percentage(containment_ratio(app));
    let local_routes = percentage(route_ratio(app));
    let pressure = pressure_score(app);
    let latency_label = if has_signal(&app.latency_history) {
        format!("{:>3}ms", average_latency_ms(app))
    } else {
        "idle".to_string()
    };
    let mode = if app.stats.total_requests == 0 {
        "LISTEN"
    } else if containment > 55 {
        "HARDEN"
    } else {
        "WATCH"
    };

    let line = Line::from(vec![
        Span::styled(
            " EIDRA ",
            Style::default()
                .fg(bg())
                .bg(accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            "LOCAL TRUST BOUNDARY",
            Style::default().fg(text()).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        pill("mode", mode, accent()),
        Span::raw(" "),
        pill(
            "requests",
            format!("{:04}", app.stats.total_requests),
            text_soft(),
        ),
        Span::raw(" "),
        pill("containment", format!("{containment:>3}%"), route_teal()),
        Span::raw(" "),
        pill("local", format!("{local_routes:>3}%"), route_blue()),
        Span::raw(" "),
        pill("pressure", format!("{pressure:>3}"), warm_gold()),
        Span::raw(" "),
        pill("latency", latency_label, accent_soft()),
    ]);

    frame.render_widget(Paragraph::new(line).block(panel(None, accent_soft())), area);
}

fn render_trust_summary(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let last = app.entries.last();
    let last_provider = last
        .map(|entry| shorten(&entry.provider, 18))
        .unwrap_or_else(|| "awaiting traffic".to_string());
    let last_action = last
        .map(|entry| action_short(&entry.action).to_string())
        .unwrap_or_else(|| "NONE".to_string());

    let lines = vec![
        Line::from(vec![span_dim("NODE"), span_value(" localhost/edge-01")]),
        Line::from(vec![
            span_dim("UPTIME"),
            span_value(format!(" {}s", app.uptime_secs)),
        ]),
        Line::from(vec![
            span_dim("REQUESTS"),
            span_metric(format!(" {}", app.stats.total_requests)),
        ]),
        Line::from(vec![
            span_dim("CONTAINED"),
            span_metric(format!(" {}%", percentage(containment_ratio(app)))),
        ]),
        Line::from(vec![
            span_dim("LOCAL ROUTE"),
            Span::styled(
                format!(" {}%", percentage(route_ratio(app))),
                Style::default()
                    .fg(route_blue())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            span_dim("BLOCKED"),
            Span::styled(
                format!(" {}", app.stats.blocked),
                Style::default()
                    .fg(action_color(&RequestAction::Block))
                    .add_modifier(Modifier::BOLD),
            ),
            span_dim("  ALERTS "),
            Span::styled(
                app.stats.escalated.to_string(),
                Style::default()
                    .fg(action_color(&RequestAction::Escalate))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            span_dim("LAST"),
            span_value(format!(" {last_action}")),
        ]),
        Line::from(vec![
            span_dim("TARGET"),
            Span::styled(format!(" {last_provider}"), Style::default().fg(text())),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lines).block(panel(Some(" TRUST SUMMARY "), accent_soft())),
        area,
    );
}

fn render_channel_ledger(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let provider_counts = provider_counts(app);
    let categories = category_counts(app);
    let total = provider_counts
        .first()
        .map(|(_, count)| *count)
        .unwrap_or(1)
        .max(1);

    let mut lines = vec![Line::from(vec![span_dim("PROVIDER FLOW")])];
    if provider_counts.is_empty() {
        lines.push(Line::from(vec![span_dim(" no live providers yet")]));
        lines.push(Line::from(vec![
            span_dim(" point SDKs to "),
            Span::styled("127.0.0.1:8080", Style::default().fg(accent())),
        ]));
    } else {
        for (provider, count) in provider_counts.iter().take(5) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>3}", count),
                    Style::default().fg(accent()).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    block_bar(*count, total, 10),
                    Style::default().fg(route_blue()),
                ),
                Span::styled(
                    format!(" {}", shorten(provider, 14)),
                    Style::default().fg(text()),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![span_dim("SENSITIVE HOTSPOTS")]));
    if categories.is_empty() {
        lines.push(Line::from(vec![span_dim(" waiting for findings")]));
    } else {
        for (category, count) in categories.into_iter().take(4) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>3}", count),
                    Style::default()
                        .fg(warm_gold())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(shorten(&category, 18), Style::default().fg(text_soft())),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).block(panel(Some(" TRUST CHANNELS "), accent_soft())),
        area,
    );
}

fn render_boundary_field(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let outer = panel(Some(" TRUST BOUNDARY "), accent());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let labels = Line::from(vec![
        stage_label(" ingress ", accent_dim()),
        stage_label(" inspect ", warm_gold()),
        stage_label(" decide ", accent()),
        stage_label(" local / cloud ", route_blue()),
    ]);
    frame.render_widget(Paragraph::new(labels), sections[0]);

    let phase = app.frame_tick as f64 / 10.0;
    let canvas = Canvas::default()
        .background_color(bg())
        .marker(Marker::Braille)
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(|ctx| {
            for lane in [18.0, 34.0, 50.0, 66.0, 82.0] {
                ctx.draw(&CanvasLine {
                    x1: 4.0,
                    y1: lane,
                    x2: 96.0,
                    y2: lane,
                    color: grid(),
                });
            }

            for gate in [26.0, 50.0, 74.0] {
                ctx.draw(&CanvasLine {
                    x1: gate,
                    y1: 10.0,
                    x2: gate,
                    y2: 90.0,
                    color: grid(),
                });
            }

            ctx.draw(&Rectangle {
                x: 46.0,
                y: 12.0,
                width: 8.0,
                height: 76.0,
                color: accent_dim(),
            });

            for radius in [8.0, 16.0, 24.0] {
                ctx.draw(&Circle {
                    x: 50.0,
                    y: 50.0,
                    radius,
                    color: if radius == 16.0 {
                        accent_soft()
                    } else {
                        grid()
                    },
                });
            }

            let pulse_radius = 11.0 + phase.sin().abs() * 3.0;
            ctx.draw(&Circle {
                x: 50.0,
                y: 50.0,
                radius: pulse_radius,
                color: glow(),
            });

            ctx.draw(&Circle {
                x: 87.0,
                y: 76.0,
                radius: 8.0,
                color: route_teal(),
            });
            ctx.draw(&Circle {
                x: 87.0,
                y: 26.0,
                radius: 8.0,
                color: accent_soft(),
            });

            ctx.draw(&CanvasLine {
                x1: 74.0,
                y1: 76.0,
                x2: 87.0,
                y2: 76.0,
                color: route_teal(),
            });
            ctx.draw(&CanvasLine {
                x1: 74.0,
                y1: 26.0,
                x2: 87.0,
                y2: 26.0,
                color: accent_dim(),
            });

            for (idx, lane) in [18.0, 34.0, 50.0, 66.0, 82.0].iter().enumerate() {
                let mut ambient = Vec::with_capacity(32);
                for step in 0..32 {
                    let x = 4.0 + step as f64 / 31.0 * 90.0;
                    let y = lane + ((x / 8.5) + phase + idx as f64 * 0.7).sin() * 0.7;
                    ambient.push((x, y));
                }
                draw_polyline(ctx, &ambient, grid());
            }

            let entries: Vec<&RequestEntry> = app.entries.iter().rev().take(18).collect();
            let mut heads = Vec::new();
            for (idx, entry) in entries.iter().enumerate() {
                let lane = 80.0 - (idx % 5) as f64 * 15.0;
                let speed = 1.1 + (idx % 4) as f64 * 0.12;
                let head = ((app.frame_tick as f64 * speed) + idx as f64 * 9.0) % 118.0 - 8.0;
                let mut trail = Vec::new();
                for step in 0..7 {
                    let progress = head - step as f64 * 3.4;
                    if !(0.0..=100.0).contains(&progress) {
                        continue;
                    }
                    let point = packet_position(entry, lane, progress, phase + idx as f64 * 0.35);
                    trail.push(point);
                }

                let color = action_color(&entry.action);
                if trail.len() > 1 {
                    draw_polyline(ctx, &trail, color);
                }
                if let Some(head_point) = trail.first().copied() {
                    heads.push(head_point);
                }
            }

            if !heads.is_empty() {
                ctx.draw(&Points {
                    coords: &heads,
                    color: accent_bright(),
                });
            }
        });

    frame.render_widget(canvas, sections[1]);
}

fn render_decision_flow(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let outer = panel(Some(" DECISION FLOW "), accent_soft());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(inner);

    if has_signal(&app.payload_history) {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                span_dim("ingress  "),
                Span::styled(
                    sparkbars(&app.payload_history),
                    Style::default().fg(text_soft()),
                ),
                Span::raw(" "),
                span_metric(format!("{:>4.0}KB", average(&app.payload_history))),
            ])),
            rows[0],
        );
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                span_dim("ingress  "),
                Span::styled("idle", Style::default().fg(text_soft())),
                span_dim("  awaiting traffic"),
            ])),
            rows[0],
        );
    }

    if has_signal(&app.findings_history) {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                span_dim("findings "),
                Span::styled(
                    sparkbars(&app.findings_history),
                    Style::default().fg(warm_gold()),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:>3}", app.stats.total_findings),
                    Style::default().fg(accent()).add_modifier(Modifier::BOLD),
                ),
            ])),
            rows[1],
        );
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                span_dim("findings "),
                Span::styled("idle", Style::default().fg(text_soft())),
                span_dim("  no classifications yet"),
            ])),
            rows[1],
        );
    }

    if has_signal(&app.latency_history) {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                span_dim("latency  "),
                Span::styled(
                    sparkbars(&app.latency_history),
                    Style::default().fg(route_blue()),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:>3}ms", average_latency_ms(app)),
                    Style::default()
                        .fg(accent_bright())
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            rows[2],
        );
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                span_dim("latency  "),
                Span::styled("idle", Style::default().fg(text_soft())),
                span_dim("  no upstream timings yet"),
            ])),
            rows[2],
        );
    }

    let footer = Line::from(vec![
        span_dim("principle "),
        Span::styled(
            "inspect before egress",
            Style::default().fg(text()).add_modifier(Modifier::BOLD),
        ),
        span_dim("  vector "),
        span_metric(format!("{:03}", app.frame_tick % 360)),
    ]);
    frame.render_widget(Paragraph::new(footer), rows[3]);
}

fn render_guardian_radar(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let outer = panel(Some(" GUARDIAN SWEEP "), accent_soft());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let label = Line::from(vec![
        stage_label(" quiet zone ", route_teal()),
        stage_label(" inspect ", accent()),
        stage_label(" alert arc ", action_color(&RequestAction::Block)),
    ]);
    frame.render_widget(Paragraph::new(label), sections[0]);

    let sweep = app.frame_tick as f64 / 14.0;
    let canvas = Canvas::default()
        .background_color(bg())
        .marker(Marker::Braille)
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(|ctx| {
            for radius in [16.0, 30.0, 44.0] {
                ctx.draw(&Circle {
                    x: 50.0,
                    y: 50.0,
                    radius,
                    color: grid(),
                });
            }

            for spoke in 0..8 {
                let angle = spoke as f64 / 8.0 * PI * 2.0;
                ctx.draw(&CanvasLine {
                    x1: 50.0,
                    y1: 50.0,
                    x2: 50.0 + angle.cos() * 46.0,
                    y2: 50.0 + angle.sin() * 46.0,
                    color: grid(),
                });
            }

            for beam in 0..8 {
                let angle = sweep - beam as f64 * 0.08;
                ctx.draw(&CanvasLine {
                    x1: 50.0,
                    y1: 50.0,
                    x2: 50.0 + angle.cos() * 46.0,
                    y2: 50.0 + angle.sin() * 46.0,
                    color: if beam == 0 {
                        route_blue()
                    } else if beam < 3 {
                        route_teal()
                    } else {
                        accent_dim()
                    },
                });
            }

            let mut echoes = Vec::new();
            for (idx, entry) in app.entries.iter().rev().take(16).enumerate() {
                let seed = hash_seed(&entry.provider) as f64;
                let angle = (seed / 19.0 + idx as f64 * 0.53) % (PI * 2.0);
                let radius = match entry.action {
                    RequestAction::Allow => 18.0,
                    RequestAction::Route => 24.0,
                    RequestAction::Mask => 31.0,
                    RequestAction::Block => 40.0,
                    RequestAction::Escalate => 44.0,
                } + (entry.findings_count as f64).min(6.0);
                echoes.push((
                    50.0 + angle.cos() * radius.min(46.0),
                    50.0 + angle.sin() * radius.min(46.0),
                ));
            }
            if !echoes.is_empty() {
                ctx.draw(&Points {
                    coords: &echoes,
                    color: accent_bright(),
                });
            }
        });

    frame.render_widget(canvas, sections[1]);
}

fn render_live_stream(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let outer = panel(Some(" LIVE DECISIONS "), accent_soft());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(3)])
        .split(inner);

    let visible = sections[0].height as usize;
    let total = app.entries.len();
    let mut lines = Vec::new();
    if total == 0 {
        lines.push(Line::from(vec![span_dim(" waiting for live requests")]));
        lines.push(Line::from(vec![
            span_dim(" export "),
            Span::styled("HTTPS_PROXY", Style::default().fg(route_blue())),
            span_dim(" to point through Eidra"),
        ]));
    } else {
        let offset = app.scroll_offset.min(total.saturating_sub(1));
        let end = total.saturating_sub(offset);
        let start = end.saturating_sub(visible.max(1));
        for entry in app.entries[start..end].iter().rev() {
            lines.push(event_line(entry));
        }
    }
    frame.render_widget(Paragraph::new(lines), sections[0]);

    let summary = if let Some(entry) = app.entries.last() {
        Line::from(vec![
            span_dim("latest "),
            Span::styled(
                action_short(&entry.action),
                Style::default()
                    .fg(action_color(&entry.action))
                    .add_modifier(Modifier::BOLD),
            ),
            span_dim("  categories "),
            Span::styled(latest_categories(entry), Style::default().fg(text_soft())),
        ])
    } else {
        Line::from(vec![
            span_dim("use "),
            Span::styled("j / k", Style::default().fg(accent())),
            span_dim(" to scroll once traffic arrives"),
        ])
    };
    frame.render_widget(Paragraph::new(summary), sections[1]);
}

fn render_sensitive_surface(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let outer = panel(Some(" SENSITIVE SURFACE "), accent_soft());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    let categories = category_counts(app);
    let max = categories
        .first()
        .map(|(_, count)| *count)
        .unwrap_or(1)
        .max(1);

    let mut lines = Vec::new();
    if categories.is_empty() {
        lines.push(Line::from(vec![span_dim(
            " no sensitive categories detected yet",
        )]));
    } else {
        for (idx, (category, count)) in categories.iter().take(4).enumerate() {
            let color = match idx {
                0 => accent_bright(),
                1 => warm_gold(),
                _ => accent(),
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>3}", count),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(block_bar(*count, max, 12), Style::default().fg(color)),
                Span::styled(
                    format!(" {}", shorten(category, 18)),
                    Style::default().fg(text()),
                ),
            ]));
        }
    }
    frame.render_widget(Paragraph::new(lines), sections[0]);

    let footer = Line::from(vec![
        span_dim("risk "),
        Span::styled(
            if has_signal(&app.risk_history) {
                sparkbars(&app.risk_history)
            } else {
                "idle".to_string()
            },
            Style::default().fg(route_blue()),
        ),
        Span::raw(" "),
        span_metric(format!("{:>3}", pressure_score(app))),
    ]);
    frame.render_widget(Paragraph::new(footer), sections[1]);
}

fn render_response_channels(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let outer = panel(Some(" RESPONSE CHANNELS "), accent_soft());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(inner);

    let contain = containment_ratio(app);
    let local = route_ratio(app);
    frame.render_widget(
        Gauge::default()
            .ratio(contain.clamp(0.0, 1.0))
            .label(format!("containment {:>3}%", percentage(contain)))
            .use_unicode(true)
            .style(Style::default().bg(bg()))
            .gauge_style(Style::default().fg(route_teal()))
            .block(panel(None, accent_dim())),
        rows[0],
    );
    frame.render_widget(
        Gauge::default()
            .ratio(local.clamp(0.0, 1.0))
            .label(format!("local route {:>3}%", percentage(local)))
            .use_unicode(true)
            .style(Style::default().bg(bg()))
            .gauge_style(Style::default().fg(route_blue()))
            .block(panel(None, accent_dim())),
        rows[1],
    );
    let latency_label = if has_signal(&app.latency_history) {
        format!("avg latency {:>3}ms", average_latency_ms(app))
    } else {
        "avg latency idle".to_string()
    };
    frame.render_widget(
        Gauge::default()
            .ratio(latency_budget_ratio(app))
            .label(latency_label)
            .use_unicode(true)
            .style(Style::default().bg(bg()))
            .gauge_style(Style::default().fg(warm_gold()))
            .block(panel(None, accent_dim())),
        rows[2],
    );

    let footer = Line::from(vec![
        span_dim("routes "),
        Span::styled(
            format!("{}", app.stats.routed),
            Style::default()
                .fg(route_blue())
                .add_modifier(Modifier::BOLD),
        ),
        span_dim("  blocks "),
        Span::styled(
            format!("{}", app.stats.blocked),
            Style::default()
                .fg(action_color(&RequestAction::Block))
                .add_modifier(Modifier::BOLD),
        ),
        span_dim("  alerts "),
        Span::styled(
            format!("{}", app.stats.escalated),
            Style::default()
                .fg(action_color(&RequestAction::Escalate))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(footer), rows[3]);
}

fn render_policy_matrix(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let total = app.stats.total_requests.max(1);
    let lines = vec![
        policy_line("allow ", app.stats.allowed, total, &RequestAction::Allow),
        policy_line("route ", app.stats.routed, total, &RequestAction::Route),
        policy_line("mask  ", app.stats.masked, total, &RequestAction::Mask),
        policy_line("block ", app.stats.blocked, total, &RequestAction::Block),
        policy_line(
            "alert ",
            app.stats.escalated,
            total,
            &RequestAction::Escalate,
        ),
        Line::from(""),
        Line::from(vec![
            span_dim("findings "),
            span_metric(format!("{:>4}", app.stats.total_findings)),
            span_dim("  hotspots "),
            span_metric(category_counts(app).len().to_string()),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lines).block(panel(Some(" POLICY MATRIX "), accent_soft())),
        area,
    );
}

fn render_footer(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            " j/k ",
            Style::default().fg(accent()).add_modifier(Modifier::BOLD),
        ),
        span_dim("scroll  "),
        Span::styled(
            " q ",
            Style::default().fg(accent()).add_modifier(Modifier::BOLD),
        ),
        span_dim("close  "),
        span_dim("frame "),
        span_metric(format!("{:05}", app.frame_tick)),
        span_dim("  "),
        Span::styled(
            "inspect -> decide -> contain",
            Style::default().fg(text_soft()),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn event_line(entry: &RequestEntry) -> Line<'static> {
    let time = entry.timestamp.format("%H:%M:%S").to_string();
    let findings = if entry.findings_count == 0 {
        "clean".to_string()
    } else {
        format!("{} hit", entry.findings_count)
    };
    let latency = if entry.latency_ms == 0 {
        "--".to_string()
    } else {
        format!("{}ms", entry.latency_ms)
    };

    Line::from(vec![
        Span::styled(format!(" {} ", time), Style::default().fg(accent_dim())),
        Span::styled(
            format!("{:<5}", action_short(&entry.action)),
            Style::default()
                .fg(action_color(&entry.action))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {:<11}", shorten(&entry.provider, 11)),
            Style::default().fg(text()),
        ),
        Span::styled(
            format!(" {:>3}", entry.status_code),
            Style::default().fg(route_blue()),
        ),
        Span::styled(format!(" {:>5}", latency), Style::default().fg(text_soft())),
        Span::styled(
            format!(" {:>7}", findings),
            Style::default().fg(text_soft()),
        ),
    ])
}

fn policy_line(label: &str, value: u64, total: u64, action: &RequestAction) -> Line<'static> {
    Line::from(vec![
        span_dim(label),
        Span::styled(
            block_bar(value, total, 14),
            Style::default().fg(action_color(action)),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>3}", value),
            Style::default()
                .fg(action_color(action))
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn packet_position(entry: &RequestEntry, lane: f64, progress: f64, phase: f64) -> (f64, f64) {
    let wave = ((progress / 9.0) + phase).sin() * 1.2;
    match &entry.action {
        RequestAction::Allow => {
            let y = if progress <= 74.0 {
                lane + wave
            } else {
                let t = ((progress - 74.0) / 18.0).clamp(0.0, 1.0);
                lane + wave + (26.0 - lane) * t
            };
            (progress, y)
        }
        RequestAction::Route => {
            let y = if progress <= 60.0 {
                lane + wave
            } else {
                let t = ((progress - 60.0) / 22.0).clamp(0.0, 1.0);
                lane + wave + (76.0 - lane) * t
            };
            (progress, y)
        }
        RequestAction::Mask => {
            let y = if progress <= 52.0 {
                lane + wave
            } else {
                let t = ((progress - 52.0) / 34.0).clamp(0.0, 1.0);
                lane + wave + (34.0 - lane) * t
            };
            (progress, y)
        }
        RequestAction::Block => {
            let stop = progress.min(52.0);
            (stop, lane + wave + (phase * 2.4).sin() * 0.6)
        }
        RequestAction::Escalate => {
            let stop = progress.min(58.0);
            let t = ((stop - 42.0) / 16.0).clamp(0.0, 1.0);
            (stop, lane + wave + (50.0 - lane) * t)
        }
    }
}

fn panel<'a>(title: Option<&'a str>, border: Color) -> Block<'a> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(bg()));

    if let Some(title) = title {
        block = block.title(Span::styled(
            title,
            Style::default().fg(accent()).add_modifier(Modifier::BOLD),
        ));
    }

    block
}

fn draw_polyline(ctx: &mut ratatui::widgets::canvas::Context, points: &[(f64, f64)], color: Color) {
    for window in points.windows(2) {
        let (x1, y1) = window[0];
        let (x2, y2) = window[1];
        ctx.draw(&CanvasLine {
            x1,
            y1,
            x2,
            y2,
            color,
        });
    }
}

fn provider_counts(app: &TuiApp) -> Vec<(String, u64)> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for entry in app.entries.iter().rev().take(24) {
        *counts.entry(entry.provider.clone()).or_insert(0) += 1;
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    pairs
}

fn category_counts(app: &TuiApp) -> Vec<(String, u64)> {
    let mut pairs: Vec<_> = app
        .stats
        .categories
        .iter()
        .map(|(category, count)| (category.clone(), *count))
        .collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1));
    pairs
}

fn latest_categories(entry: &RequestEntry) -> String {
    if entry.categories.is_empty() {
        "none".to_string()
    } else {
        entry
            .categories
            .iter()
            .take(3)
            .map(|category| shorten(category, 10))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn has_signal(values: &[u64]) -> bool {
    values.iter().any(|value| *value > 0)
}

fn sparkbars(values: &[u64]) -> String {
    let bars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let max = values.iter().copied().max().unwrap_or(1).max(1);
    values
        .iter()
        .map(|value| {
            let idx = ((*value * (bars.len() as u64 - 1)) / max) as usize;
            bars[idx]
        })
        .collect::<Vec<_>>()
        .join("")
}

fn block_bar(value: u64, max: u64, width: usize) -> String {
    let filled = ((value as f64 / max.max(1) as f64) * width as f64).round() as usize;
    format!(
        "{}{}",
        "█".repeat(filled.min(width)),
        "·".repeat(width.saturating_sub(filled.min(width)))
    )
}

fn average(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<u64>() as f64 / values.len() as f64
}

fn containment_ratio(app: &TuiApp) -> f64 {
    let total = app.stats.total_requests.max(1) as f64;
    (app.stats.blocked + app.stats.masked + app.stats.routed + app.stats.escalated) as f64 / total
}

fn route_ratio(app: &TuiApp) -> f64 {
    let total = app.stats.total_requests.max(1) as f64;
    app.stats.routed as f64 / total
}

fn pressure_score(app: &TuiApp) -> u64 {
    ((average(&app.findings_history) * 0.45) + (average(&app.risk_history) * 0.55)).round() as u64
}

fn average_latency_ms(app: &TuiApp) -> u64 {
    average(&app.latency_history).round() as u64
}

fn latency_budget_ratio(app: &TuiApp) -> f64 {
    if !has_signal(&app.latency_history) {
        return 0.0;
    }
    (average_latency_ms(app) as f64 / 300.0).clamp(0.0, 1.0)
}

fn percentage(value: f64) -> u64 {
    (value.clamp(0.0, 1.0) * 100.0).round() as u64
}

fn hash_seed(value: &str) -> u64 {
    value.bytes().fold(0u64, |acc, byte| {
        acc.wrapping_mul(33).wrapping_add(byte as u64)
    })
}

fn shorten(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    value
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>()
        + "…"
}

fn action_short(action: &RequestAction) -> &'static str {
    match action {
        RequestAction::Allow => "ALLOW",
        RequestAction::Route => "ROUTE",
        RequestAction::Mask => "MASK ",
        RequestAction::Block => "BLOCK",
        RequestAction::Escalate => "ALERT",
    }
}

fn action_color(action: &RequestAction) -> Color {
    match action {
        RequestAction::Allow => route_teal(),
        RequestAction::Route => route_blue(),
        RequestAction::Mask => warm_gold(),
        RequestAction::Block => Color::Rgb(228, 117, 98),
        RequestAction::Escalate => Color::Rgb(192, 150, 221),
    }
}

fn pill<T: Into<String>>(label: &str, value: T, color: Color) -> Span<'static> {
    Span::styled(
        format!(" {} {} ", label.to_uppercase(), value.into()),
        Style::default()
            .fg(bg())
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn stage_label<T: Into<String>>(value: T, color: Color) -> Span<'static> {
    Span::styled(
        value.into(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn span_dim<T: Into<String>>(value: T) -> Span<'static> {
    Span::styled(value.into(), Style::default().fg(accent_dim()))
}

fn span_metric<T: Into<String>>(value: T) -> Span<'static> {
    Span::styled(
        value.into(),
        Style::default().fg(accent()).add_modifier(Modifier::BOLD),
    )
}

fn span_value<T: Into<String>>(value: T) -> Span<'static> {
    Span::styled(value.into(), Style::default().fg(text()))
}

fn bg() -> Color {
    Color::Rgb(10, 11, 14)
}

fn text() -> Color {
    Color::Rgb(232, 226, 214)
}

fn text_soft() -> Color {
    Color::Rgb(191, 184, 171)
}

fn accent() -> Color {
    Color::Rgb(228, 188, 125)
}

fn accent_soft() -> Color {
    Color::Rgb(155, 125, 85)
}

fn accent_dim() -> Color {
    Color::Rgb(102, 85, 62)
}

fn accent_bright() -> Color {
    Color::Rgb(252, 236, 206)
}

fn warm_gold() -> Color {
    Color::Rgb(209, 168, 96)
}

fn route_teal() -> Color {
    Color::Rgb(123, 184, 170)
}

fn route_blue() -> Color {
    Color::Rgb(112, 171, 203)
}

fn grid() -> Color {
    Color::Rgb(48, 43, 37)
}

fn glow() -> Color {
    Color::Rgb(192, 152, 94)
}
