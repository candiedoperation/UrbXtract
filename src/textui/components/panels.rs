use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

pub struct TitleBar {
    pub title: String,
}

impl Widget for TitleBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_line = Line::from(self.title).centered();
        title_line.render(area, buf);
    }
}
