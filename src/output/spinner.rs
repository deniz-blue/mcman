use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Default)]
pub struct Spinner<'a> {
    tick: usize,
    label: Option<Line<'a>>,
    style: Style,
}

impl<'a> Spinner<'a> {
    pub fn new(tick: usize) -> Self {
        Self {
            tick,
            label: None,
            style: Style::new(),
        }
    }

    pub fn label(mut self, label: impl Into<Line<'a>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for Spinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame = Span::styled(FRAMES[self.tick % FRAMES.len()], self.style);

        let line = match self.label {
            Some(label) => {
                let mut spans = vec![frame, Span::raw(" ")];
                spans.extend(label.spans);
                Line::from(spans)
            }
            None => Line::from(frame),
        };

        line.render(area, buf);
    }
}
