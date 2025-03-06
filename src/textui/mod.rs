use std::time::Duration;
use crossterm::{event::{self, Event, EventStream, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}};
use futures::{FutureExt, StreamExt};
use ratatui::{prelude::Backend, text::Line, Frame, Terminal};
use tokio::{sync::mpsc::Receiver, time::Instant};
use crate::reconstructor::ReconstructedTransmission;

pub struct UserInterface {
    app_title: String,
    consume_rx: Receiver<ReconstructedTransmission>
}

impl UserInterface {
    fn render(&mut self, frame: &mut Frame) {
        let rndr_area = frame.area();
        frame.render_widget(Line::from(self.app_title.clone()), rndr_area);
    }
    
    pub fn new(mut consume_rx: Receiver<ReconstructedTransmission>) -> Self {
        UserInterface { 
            app_title: String::from("UrbXtract 0.0.1"),
            consume_rx,
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

                    println!("Got Reconstructed Pkt: {:?}", transmission);
                }
            }
        }

        /* Clean Exit? Save Capture? */
        disable_raw_mode().unwrap();
    }
}