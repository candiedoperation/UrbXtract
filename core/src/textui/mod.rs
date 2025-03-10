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

mod components;

use std::time::Duration;
use components::{panels::{ShortcutsFooter, ShortcutsFooterState, TitleBar}, tables::VirtualizedTable};
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{layout::{Constraint, Direction, Layout}, prelude::Backend, widgets::{Row, TableState}, Frame, Terminal};
use tokio::{sync::mpsc::Receiver, time::Instant};
use crate::reconstructor::ReconstructedTransmission;

enum UIPage {
    MainTableView
}

pub struct UserInterface<'a> {
    /* Main Interface Config */
    app_title: String,
    active_page: UIPage,
    shortcutspnl_state: ShortcutsFooterState,
    
    /* Table Options */
    rows: Vec<Row<'a>>,
    table_state: TableState,
    table_auto_scroll: bool,
    
    /* Data consumer */
    consume_rx: Receiver<ReconstructedTransmission>
}

/* Define Constants */
const STATIC_ROW_WIDTH: u16 = 35; // Update if you change column lengths

fn sanitize_ansi_escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '\x1B' => "\\e".to_string(), // Escape sequence start
            c if c.is_ascii_control() => format!("\\x{:02X}", c as u8),
            c => c.to_string(),
        })
        .collect()
}

impl UIPage {
    pub fn get_pagename(&self) -> String {
        match self {
            UIPage::MainTableView => String::from("Packet Capture"),
        }
    }
    
    pub fn get_apptitle(&self) -> String {
        let page_name = self.get_pagename();
        return format!("UrbXtract 0.0.1 > {}", page_name);
    }
}

impl<'a> UserInterface<'a> {
    fn render(&mut self, frame: &mut Frame) {
        let rndr_area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(2), Constraint::Percentage(96), Constraint::Percentage(2)].as_ref())
            .split(rndr_area);

        /* Create Table */
        let table = VirtualizedTable {
            rows: self.rows.clone(),
            widths: vec![
                Constraint::Length(8), /* Packet # */
                Constraint::Length(8), /* Bus ID */
                Constraint::Length(8), /* Dev ID */
                Constraint::Length(11), /* Pkt Src */
                Constraint::Min(65)    /* Payload Preview */
            ],

            header: Row::new(vec![
                "#",
                "Bus ID",
                "Dev ID",
                "Direction",
                "Payload Preview"
            ])
        };

        /* Create Title Bar */
        let title_bar = TitleBar {
            title: self.app_title.clone(),
        };

        frame.render_widget(title_bar, chunks[0]);
        frame.render_stateful_widget(table, chunks[1], &mut (self.table_state));
        frame.render_stateful_widget(ShortcutsFooter {}, chunks[2], &mut (self.shortcutspnl_state));
    }
    
    pub fn new(consume_rx: Receiver<ReconstructedTransmission>) -> Self {
        UserInterface { 
            app_title: UIPage::MainTableView.get_apptitle(),
            active_page: UIPage::MainTableView,
            consume_rx,
            rows: vec![],
            table_state: TableState::default(),
            table_auto_scroll: true,
            shortcutspnl_state: ShortcutsFooterState {
                shortcuts: vec![
                    String::from("More Info (↵)"),
                    String::from("To Top (Shift + Up)"),
                    String::from("To Bottom (Shift + Down)"),
                    String::from("Quit (q)")
                ]
            },
        }
    }

    fn handle_terminal_event(&mut self, event: Event) {
        if let Event::Key(key_event) = event {
            if key_event.kind == crossterm::event::KeyEventKind::Press {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Up, KeyModifiers::SHIFT) => {
                        self.table_state.select_first();
                        self.table_auto_scroll = false;
                    },
                    (KeyCode::Up, _) => {
                        self.table_state.select_previous();
                        self.table_auto_scroll = false;
                    },
                    (KeyCode::Down, KeyModifiers::SHIFT) => {
                        self.table_state.select_last();
                        self.table_auto_scroll = true;
                    },
                    (KeyCode::Down, _) => {
                        self.table_state.select_next();
                        self.table_auto_scroll = false;
                    },
                    _ => {}
                }
            }
        }
    }

    pub async fn run(&mut self, mut terminal: Terminal<impl Backend>) {
        let render_interval = Duration::from_millis(50);
        let mut last_render = Instant::now();
        let mut event_handler = EventStream::new();
        
        loop {
            tokio::select! {
                /* Re-render if interval elapsed */
                _ = tokio::time::sleep_until(last_render + render_interval) => {
                    terminal.draw(|frame| self.render(frame)).unwrap();
                    last_render = Instant::now();
                }

                /* Handle User Input Events */
                Some(Ok(event)) = event_handler.next().fuse() => {
                    match event {
                        Event::Key(key_event) => {
                            if key_event.code == KeyCode::Char('q') {
                                /* Exit the App! */
                                break;
                            } else {
                                self.handle_terminal_event(event);
                            }
                        },

                        _ => {
                            self.handle_terminal_event(event);
                        },
                    }
                }

                /* Consume Packets as Sniffer captures them */
                Some(transmission) = self.consume_rx.recv() => {
                    let (t_width, _) = crossterm::terminal::size().unwrap(); /* Get Terminal Size */
                    self.rows.push(Row::new(vec![
                        (self.rows.len() + 1).to_string(),
                        format!("{:03}", transmission.urbx_header.bus_id),
                        format!("{:03}", transmission.urbx_header.device_id),
                        
                        /* Transmission Direction */
                        if (transmission.urbx_header.endpoint_info & 0b10000000) == 0 {
                            String::from("To Device")
                        } else {
                            String::from("To Host")
                        },

                        /* Preview Data */
                        sanitize_ansi_escape(&transmission.combined_payload[0..std::cmp::min(transmission.combined_payload.len(), (t_width - STATIC_ROW_WIDTH) as usize - 15)]) + 
                        if transmission.combined_payload.len() > ((t_width - STATIC_ROW_WIDTH) as usize - 15) { "..." } else { "" },
                    ]));

                    /* Auto Scrolling */
                    if self.table_auto_scroll {
                        self.table_state.select_last();
                    }
                }
            }
        }

        /* Clean Exit? Save Capture? */
        //disable_raw_mode().unwrap();
        // process::exit(0);
    }
}