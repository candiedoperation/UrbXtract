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

use std::collections::HashMap;

use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::{StatefulWidget, Widget},
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

pub struct ShortcutsFooter {}

pub struct ShortcutsFooterState {
    pub shortcuts: Vec<String>,
}

impl StatefulWidget for ShortcutsFooter {
    type State = ShortcutsFooterState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut sp_string = String::from("|");
        state
            .shortcuts
            .iter()
            .for_each(|sh| sp_string += &format!(" {} |", sh));

        let shortcuts_panel = Line::from(sp_string).right_aligned();
        shortcuts_panel.render(area, buf);
    }
}
