mod components;

use std::time::Duration;
use components::tables::VirtualizedTable;
use crossterm::{event::{Event, EventStream, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}};
use futures::{FutureExt, StreamExt};
use ratatui::{layout::{Constraint, Direction, Layout}, prelude::Backend, text::Line, widgets::{Row, TableState}, Frame, Terminal};
use tokio::{sync::mpsc::Receiver, time::Instant};
use crate::reconstructor::ReconstructedTransmission;

pub struct UserInterface<'a> {
    app_title: String,
    rows: Vec<Row<'a>>,
    table_state: TableState,
    consume_rx: Receiver<ReconstructedTransmission>
}

impl<'a> UserInterface<'a> {
    fn render(&mut self, frame: &mut Frame) {
        let rndr_area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(2), Constraint::Percentage(98)].as_ref())
            .split(rndr_area);
        
        let table = VirtualizedTable {
            rows: self.rows.clone(),
            widths: vec![Constraint::Percentage(100)],
        };

        frame.render_widget(Line::from(self.app_title.clone()), chunks[0]);
        frame.render_stateful_widget(table, chunks[1], &mut (self.table_state));
    }
    
    pub fn new(mut consume_rx: Receiver<ReconstructedTransmission>) -> Self {
        UserInterface { 
            app_title: String::from("UrbXtract 0.0.1"),
            consume_rx,
            rows: vec![Row::new(vec!["Cell1", "Cell2", "Cell3"])],
            table_state: TableState::default()
        }
    }

    pub async fn run(&mut self, mut terminal: Terminal<impl Backend>) {
        let render_interval = Duration::from_millis(100);
        let mut last_render = Instant::now();
        let mut event_handler = EventStream::new();
        enable_raw_mode().unwrap();

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
                            }
                        },

                        _ => {},
                    }
                }

                /* Consume Packets as Sniffer captures them */
                Some(transmission) = self.consume_rx.recv() => {
                    // if (urb_packet_header.bus_id == 3 && urb_packet_header.device == 4 && urb_packet_header.data_length > 0) {
                    //     print!("{}", String::from_utf8_lossy(urb_packet_data));
                    // }
                }
            }
        }

        /* Clean Exit? Save Capture? */
        disable_raw_mode().unwrap();
    }
}