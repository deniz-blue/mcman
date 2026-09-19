use std::io::{IsTerminal, Stderr, Write};
use std::ops::ControlFlow;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use ratatui::{
    backend::{Backend, ClearType, CrosstermBackend},
    buffer::Buffer,
    layout::{Position, Rect},
    text::Line,
    Terminal, TerminalOptions, Viewport,
};

use crate::output::{
    block, row,
    task::{Emphasis, Event},
    tree::Tree,
};

const FRAME: Duration = Duration::from_millis(80);
const DEFAULT_WIDTH: u16 = 72;
const MAX_WIDTH: u16 = 100;

pub fn run(incoming: Receiver<Event>) {
    let mut renderer = ProgressRenderer::open();
    let mut tree = Tree::default();
    let mut log = Vec::new();
    let started = Instant::now();

    while pump(&mut renderer, &mut tree, &mut log, &incoming).is_continue() {
        let tick = (started.elapsed().as_millis() / FRAME.as_millis()) as usize;
        let slots = block::slots(&tree);

        flush(&mut renderer, &mut log);
        renderer.draw(slots.len() as u16, |area, buf| {
            row::draw_slots(&slots, tick, area, buf)
        });
    }

    flush(&mut renderer, &mut log);
}

fn flush(renderer: &mut ProgressRenderer, log: &mut Vec<Line<'static>>) {
    renderer.print(log.len() as u16, |area, buf| {
        row::draw_log(log, area, buf);
    });

    log.clear();
}

fn pump(
    renderer: &mut ProgressRenderer,
    tree: &mut Tree,
    log: &mut Vec<Line<'static>>,
    incoming: &Receiver<Event>,
) -> ControlFlow<()> {
    match incoming.recv_timeout(FRAME) {
        Ok(event) => apply(renderer, tree, log, event)?,
        Err(RecvTimeoutError::Timeout) => return ControlFlow::Continue(()),
        Err(RecvTimeoutError::Disconnected) => return ControlFlow::Break(()),
    }

    while let Ok(event) = incoming.try_recv() {
        apply(renderer, tree, log, event)?;
    }

    ControlFlow::Continue(())
}

fn apply(
    renderer: &mut ProgressRenderer,
    tree: &mut Tree,
    log: &mut Vec<Line<'static>>,
    event: Event,
) -> ControlFlow<()> {
    match event {
        Event::Create { id, parent, name } => tree.create(id, parent, name),
        Event::Finish {
            id,
            tag,
            text,
            detail,
        } => {
            if let Some((name, output)) = tree.finish(id) {
                if tag.emphasis == Emphasis::Failure {
                    log.extend(output.iter().map(|text| row::output(&name, text)));
                }
            }

            log.push(row::outcome(tag, &text, &detail));
        }
        Event::Progress { id, progress } => tree.set_progress(id, progress),
        Event::Line { id, text } => tree.line(id, text),
        Event::Abandon { id } => tree.abandon(id),
        Event::Suspend => renderer.suspend(),
        Event::Resume => renderer.resume(),
        Event::Close => return ControlFlow::Break(()),
    }

    ControlFlow::Continue(())
}

struct ProgressRenderer {
    terminal: Option<Terminal<CrosstermBackend<Stderr>>>,
    requested: u16,
    live: bool,
    suspended: bool,
}

impl ProgressRenderer {
    fn open() -> Self {
        Self {
            terminal: None,
            requested: 0,
            live: std::io::stderr().is_terminal(),
            suspended: false,
        }
    }

    fn draw(&mut self, rows: u16, paint: impl FnOnce(Rect, &mut Buffer)) {
        self.fit(if self.suspended { 0 } else { rows });

        let width = self.width();
        let Some(terminal) = &mut self.terminal else {
            return;
        };

        let _ = terminal.draw(|frame| {
            let area = narrow(frame.area(), width);
            paint(area, frame.buffer_mut());
        });
    }

    fn print(&mut self, rows: u16, paint: impl FnOnce(Rect, &mut Buffer)) {
        if rows == 0 {
            return;
        }

        let width = self.width();

        let Some(terminal) = &mut self.terminal else {
            let area = Rect::new(0, 0, width, rows);
            let mut buffer = Buffer::empty(area);
            paint(area, &mut buffer);
            write_text(&buffer);
            return;
        };

        let _ = terminal.insert_before(rows, |buffer| {
            paint(narrow(buffer.area, width), buffer);
        });
    }

    fn suspend(&mut self) {
        self.suspended = true;
        self.teardown();
    }

    fn resume(&mut self) {
        self.suspended = false;
    }

    fn fit(&mut self, height: u16) {
        if self.requested == height && self.terminal.is_some() {
            return;
        }

        self.teardown();

        if height == 0 || !self.live {
            return;
        }

        let backend = CrosstermBackend::new(std::io::stderr());
        let options = TerminalOptions {
            viewport: Viewport::Inline(height),
        };

        let Ok(mut terminal) = Terminal::with_options(backend, options) else {
            self.live = false;
            return;
        };

        let _ = terminal.hide_cursor();
        self.requested = height;
        self.terminal = Some(terminal);
    }

    fn teardown(&mut self) {
        self.requested = 0;

        let Some(mut terminal) = self.terminal.take() else {
            return;
        };

        let origin = terminal.get_frame().area().y;
        let _ = terminal.set_cursor_position(Position::new(0, origin));
        let _ = terminal.backend_mut().clear_region(ClearType::AfterCursor);
        let _ = terminal.show_cursor();
        let _ = Backend::flush(terminal.backend_mut());
    }

    fn width(&self) -> u16 {
        self.terminal
            .as_ref()
            .and_then(|terminal| terminal.size().ok())
            .map_or(DEFAULT_WIDTH, |size| size.width)
            .min(MAX_WIDTH)
    }
}

impl Drop for ProgressRenderer {
    fn drop(&mut self) {
        self.teardown();
    }
}

fn write_text(buffer: &Buffer) {
    let mut stderr = std::io::stderr().lock();

    for line in buffer.area.rows() {
        let text: String = (0..line.width)
            .map(|column| buffer[(column, line.y)].symbol())
            .collect();

        let _ = writeln!(stderr, "{}", text.trim_end());
    }
}

fn narrow(area: Rect, width: u16) -> Rect {
    Rect {
        width: area.width.min(width),
        ..area
    }
}
