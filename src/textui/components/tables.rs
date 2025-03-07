use ratatui::{layout::Constraint, style::{Color, Modifier, Style, Stylize}, widgets::{Block, Borders, Row, StatefulWidget, Table, TableState}};

pub struct VirtualizedTable<'a> {
    pub rows: Vec<Row<'a>>,
    pub header: Row<'a>,
    pub widths: Vec<Constraint>,
}

impl<'a> StatefulWidget for VirtualizedTable<'a> {
    type State = TableState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let visible_rows = area.height as usize;
        let start_index = state.offset();
        let end_index = (start_index + visible_rows).min(self.rows.len());
    
        /* Construct Vector of Visible Rows */
        let visible_rows = self.rows;
    
        /* Render only Visible Rows (Virtualization) */
        let table = Table::new(visible_rows, self.widths)
            .block(Block::default().borders(Borders::ALL))
            .row_highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
            .header(self.header.style(Style::default()).add_modifier(Modifier::REVERSED));
    
        /* Render the Table */
        StatefulWidget::render(table, area, buf, state);
    }
}