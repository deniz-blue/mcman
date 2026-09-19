use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::line,
    text::{Line, Span},
    widgets::{Gauge, Widget},
};

use crate::output::{
    block::Slot,
    format::{self, columns},
    spinner::Spinner,
    task::{Emphasis, Progress, Tag},
    tree::Entry,
};

const GUTTER: u16 = 12;
const INDENT: u16 = 2;

const NAME_COLUMN: u16 = 34;
const BAR: u16 = 28;
const MIN_BAR: u16 = 8;
const DIVIDER: u16 = 2;

const WHITE: Color = Color::Rgb(0xEF, 0xE9, 0xE1);
const GRAY: Color = Color::Rgb(0x75, 0x6D, 0x63);
const DARK_GRAY: Color = Color::Rgb(0x4A, 0x44, 0x3D);
const ORANGE: Color = Color::Rgb(0xD9, 0xA1, 0x4E);
const RED: Color = Color::Rgb(0xD1, 0x68, 0x5F);

pub fn draw_slots(slots: &[Slot], tick: usize, area: Rect, buf: &mut Buffer) {
    for (line, slot) in area.rows().zip(slots) {
        match slot {
            Slot::Blank => {}
            Slot::Entry {
                entry,
                depth,
                counts,
            } => {
                let indicator = Indicator::new(entry, *counts, tick);

                if *depth == 1 {
                    draw_phase(entry, &indicator, line, buf);
                } else {
                    draw_entry(entry, *depth, &indicator, line, buf);
                }
            }
            Slot::Output { text, depth } => draw_output(text, *depth, line, buf),
        }
    }
}

pub fn outcome(tag: Tag, text: &str, detail: &str) -> Line<'static> {
    let mut spans = vec![
        Span::styled(gutter(tag.text), Style::new().fg(color_of(tag.emphasis))),
        Span::raw(" "),
        Span::styled(text.to_owned(), Style::new().fg(WHITE)),
    ];

    if !detail.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(detail.to_owned(), Style::new().fg(GRAY)));
    }

    Line::from(spans)
}

pub fn output(name: &str, text: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(gutter(&format::clip(name, GUTTER)), Style::new().fg(GRAY)),
        Span::styled(format!(" {} ", line::VERTICAL), Style::new().fg(DARK_GRAY)),
        Span::styled(text.to_owned(), Style::new().fg(GRAY)),
    ])
}

pub fn draw_log(lines: &[Line], area: Rect, buf: &mut Buffer) {
    for (row, line) in area.rows().zip(lines) {
        line.render(row, buf);
    }
}

fn gutter(text: &str) -> String {
    let pad = GUTTER.saturating_sub(columns(text)) as usize;

    format!("{:pad$}{text}", "")
}

fn draw_phase(entry: &Entry, indicator: &Indicator, area: Rect, buf: &mut Buffer) {
    let elapsed = format::duration(entry.started.elapsed());

    let [gutter, _, track, fill, right] = area.layout(&Layout::horizontal([
        Constraint::Length(GUTTER),
        Constraint::Length(1),
        Constraint::Max(BAR),
        Constraint::Fill(1),
        Constraint::Length(columns(&elapsed) + 1),
    ]));

    styled(&entry.name, ORANGE)
        .right_aligned()
        .render(gutter, buf);
    indicator.draw(track, fill, buf);
    styled(&elapsed, GRAY).right_aligned().render(right, buf);
}

fn draw_entry(entry: &Entry, depth: usize, indicator: &Indicator, area: Rect, buf: &mut Buffer) {
    let [field, _, track, fill] = area.layout(&Layout::horizontal([
        Constraint::Length(NAME_COLUMN),
        Constraint::Length(1),
        Constraint::Max(BAR),
        Constraint::Fill(1),
    ]));

    let [_, field] = field.layout(&Layout::horizontal([
        Constraint::Length(indent(depth)),
        Constraint::Fill(1),
    ]));

    let mut spans = Vec::new();
    if depth >= 3 {
        spans.push(Span::styled(
            format!("{} ", line::BOTTOM_LEFT),
            Style::new().fg(DARK_GRAY),
        ));
    }
    spans.push(Span::styled(&entry.name, Style::new().fg(WHITE)));
    Line::from(spans).render(field, buf);

    indicator.draw(track, fill, buf);
}

fn draw_output(text: &str, depth: usize, area: Rect, buf: &mut Buffer) {
    let [_, wall, body] = area.layout(&Layout::horizontal([
        Constraint::Length(indent(depth) + INDENT),
        Constraint::Length(DIVIDER),
        Constraint::Fill(1),
    ]));

    styled(&format!("{} ", line::VERTICAL), DARK_GRAY).render(wall, buf);
    styled(text, GRAY).render(body, buf);
}

struct Indicator {
    ratio: Option<f64>,
    text: String,
    tick: usize,
}

impl Indicator {
    fn new(entry: &Entry, counts: Option<(usize, usize)>, tick: usize) -> Self {
        let (ratio, text) = match (counts, &entry.progress) {
            (Some((done, total)), _) => {
                (Some(done as f64 / total as f64), format!("{done}/{total}"))
            }
            (
                None,
                Some(Progress::Bytes {
                    done,
                    total: Some(total),
                }),
            ) => (
                Some(format::ratio(*done, *total)),
                format::byte_fraction(*done, *total),
            ),
            (None, Some(Progress::Bytes { done, .. })) => (None, format::bytes(*done)),
            (None, Some(Progress::Note(note))) => (None, note.clone()),
            (None, None) => (None, String::new()),
        };

        Self { ratio, text, tick }
    }

    fn draw(&self, track: Rect, fill: Rect, buf: &mut Buffer) {
        let text_only = track.union(fill);

        let Some(ratio) = self.ratio else {
            let mut spinner = Spinner::new(self.tick).style(Style::new().fg(ORANGE));

            if !self.text.is_empty() {
                spinner = spinner.label(Span::styled(&self.text, Style::new().fg(GRAY)));
            }

            spinner.render(text_only, buf);
            return;
        };

        if track.width >= MIN_BAR {
            gauge(ratio, &self.text).render(track, buf);
        } else {
            styled(&self.text, WHITE)
                .right_aligned()
                .render(text_only, buf);
        }
    }
}

fn indent(depth: usize) -> u16 {
    INDENT + (depth as u16).saturating_sub(3) * 2
}

fn styled(text: &str, color: Color) -> Line<'_> {
    Line::from(Span::styled(text, Style::new().fg(color)))
}

fn gauge<'a>(ratio: f64, label: &'a str) -> Gauge<'a> {
    Gauge::default()
        .ratio(ratio.clamp(0.0, 1.0))
        .label(Span::styled(label, Style::new().fg(WHITE)))
        .use_unicode(true)
        .gauge_style(Style::new().fg(ORANGE).bg(DARK_GRAY))
}

fn color_of(emphasis: Emphasis) -> Color {
    match emphasis {
        Emphasis::Muted => GRAY,
        Emphasis::Accent => ORANGE,
        Emphasis::Failure => RED,
    }
}
