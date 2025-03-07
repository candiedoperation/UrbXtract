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

use crate::sniffer::UrbPacket;
use tokio::sync::mpsc::Sender;

use super::{ReconstructedTransmission, ReconstructionModule};

struct SerialDatastore {
    construct_sources: Vec<UrbPacket>,
    combined_payload: String,
    pkt_direction: bool,
    dev_id: u8,
    bus_id: u16,
}

pub struct Reconstructor {
    module_tx: Sender<ReconstructedTransmission>,
    datastore: HashMap<String, SerialDatastore>, /* DeviceId, DataStore */
}

impl Reconstructor {
    async fn dispatch_packet(&mut self, device_id: String) {
        /* Dispatch the Constructed Datastore */
        match self.datastore.get(&device_id) {
            None => return,
            Some(dispatch_data) => {
                let send_request = self.module_tx.send(ReconstructedTransmission {
                    combined_payload: dispatch_data.combined_payload.clone(),
                    bus_id: dispatch_data.bus_id,
                    dev_id: dispatch_data.dev_id,
                    pkt_direction: dispatch_data.pkt_direction, 
                    sources: vec![], /* For now, we'd prolly need Box<> for efficiency */
                });

                /* Wait for the request to succeed */
                send_request.await.unwrap();
                self.datastore.insert(
                    String::from(device_id),
                    SerialDatastore {
                        construct_sources: vec![],
                        combined_payload: String::from(""),
                        pkt_direction: false,
                        dev_id: 0,
                        bus_id: 0,
                    }
                );
            }
        }
    }
}

fn get_endpoint_direction(endpoint: &u8) -> bool {
    return endpoint & 0x0F != 0;
}

impl ReconstructionModule for Reconstructor {
    fn new(module_tx: Sender<ReconstructedTransmission>) -> Self {
        Self {
            module_tx,
            datastore: HashMap::new(),
        }
    }

    async fn consume_packet(&mut self, urb_packet: crate::sniffer::UrbPacket) {
        let urb_header = &urb_packet.header;
        let urb_data = urb_packet.data.as_ref().unwrap();
        let strbuild_result = String::from_utf8(urb_data.to_vec());

        /* Construct Datastore */
        let device_id = &format!("{}:{}", urb_header.bus_id, urb_header.device);
        if !self.datastore.contains_key(device_id) {
            self.datastore.insert(
                String::from(device_id),
                SerialDatastore {
                    construct_sources: vec![],
                    combined_payload: String::from(""),
                    pkt_direction: get_endpoint_direction(&urb_header.endpoint),
                    dev_id: urb_header.device,
                    bus_id: urb_header.bus_id,
                },
            );
        }

        let datastore = self.datastore.get(device_id).unwrap();
        if strbuild_result.is_err() {
            if datastore.combined_payload != "" {
                /* We have data from previous packets, dispatch it */
                self.dispatch_packet(String::from(device_id)).await;
            }

            /* Construct New datastore */
            self.datastore.insert(
                String::from(device_id),
                SerialDatastore {
                    pkt_direction: get_endpoint_direction(&urb_header.endpoint),
                    dev_id: urb_header.device,
                    bus_id: urb_header.bus_id,
                    construct_sources: vec![urb_packet],
                    combined_payload: String::from("Non-UTF8 Binary Data"),
                },
            );

            /* Dispatch the Packet */
            self.dispatch_packet(String::from(device_id)).await;
        } else {
            /* We have a valid UTF8 Serial String */
            let parsed_strdata = strbuild_result.unwrap();
            
            /* Expand existing datastore */
            let datastore = self.datastore.get_mut(device_id).unwrap();
            datastore.combined_payload += &parsed_strdata;
            datastore.construct_sources.push(urb_packet);
            
            /* If \r\n or \n, dispatch the packet */
            if parsed_strdata.ends_with("\n") {
                self.dispatch_packet(String::from(device_id)).await;
            }
        }
    }
}
