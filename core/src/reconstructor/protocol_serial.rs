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
use tokio::sync::mpsc::Sender;
use crate::sniffer::UrbXractPacket;

use super::{ReconstructedTransmission, ReconstructionModule};

pub struct Reconstructor {
    module_tx: Sender<ReconstructedTransmission>,
    datastore: HashMap<String, ReconstructedTransmission>, /* DeviceId, DataStore */
}

impl Reconstructor {
    async fn dispatch_packet(&mut self, device_id: String) {
        /* Dispatch the Constructed Datastore */
        match self.datastore.remove(&device_id) {
            None => return,
            Some(dispatch_data) => {
                /* Transmit the Data */
                self.module_tx.send(dispatch_data).await.unwrap();
            }
        }
    }
}

impl ReconstructionModule for Reconstructor {
    fn new(module_tx: Sender<ReconstructedTransmission>) -> Self {
        Self {
            module_tx,
            datastore: HashMap::new(),
        }
    }

    async fn consume_packet(&mut self, urb_packet: crate::sniffer::UrbXractPacket) {
        let urb_header = &urb_packet.header;
        let urb_data = urb_packet.data.as_ref().unwrap();
        let strbuild_result = String::from_utf8(urb_data.to_vec());

        /* Construct Datastore */
        let device_id = &format!("{}:{}:{}", urb_header.bus_id, urb_header.device_id, (urb_header.endpoint_info & 0b10000000 == 0));
        let datastore = self.datastore.get(device_id);
        
        if strbuild_result.is_err() {
            if datastore.is_some() {
                /* We have data from previous packets, dispatch it */
                self.dispatch_packet(String::from(device_id)).await;
            } else {
                /* Build a new Packet */
                self.datastore.insert(
                    String::from(device_id),
                    ReconstructedTransmission {
                        urbx_header: urb_packet.header,
                        combined_payload: String::from("(Non-UTF8 Binary Data)"),
                        sources: vec![urb_packet], /* Should have URB Packet? */
                    }
                );

                /* Dispatch the Packet */
                self.dispatch_packet(String::from(device_id)).await;
            }
        } else {
            /* We have a valid UTF8 Serial String */
            let parsed_strdata = strbuild_result.unwrap();

            /* Check Datastore, Create one if it doesn't exist */
            if datastore.is_none() {
                self.datastore.insert(
                    String::from(device_id), 
                    ReconstructedTransmission {
                        urbx_header: urb_packet.header,
                        combined_payload: parsed_strdata.clone(),
                        sources: vec![
                            UrbXractPacket { 
                                header: urb_packet.header, 
                                data: None 
                            }
                        ],
                    }
                );
            } else {
                /* Expand the Previous String */
                let datastore_mut = self.datastore.get_mut(device_id).unwrap();
                datastore_mut.combined_payload += &parsed_strdata;
                datastore_mut.sources.push(
                    UrbXractPacket {
                        header: urb_packet.header,
                        data: None
                    }
                );
            }

            /* If \r\n or \n, dispatch the packet */
            if parsed_strdata.ends_with("\n") {
                self.dispatch_packet(String::from(device_id)).await;
            }
        }
    }
}
