/*
    UrbXtract
    Copyright (C) 2025  Atheesh Thirumalairajan

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>. 
*/

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