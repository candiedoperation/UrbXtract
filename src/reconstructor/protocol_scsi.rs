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

use tokio::sync::mpsc::Sender;
use super::{ReconstructedTransmission, ReconstructionModule};

pub struct Reconstructor {
    module_tx: Sender<ReconstructedTransmission>
}

impl ReconstructionModule for Reconstructor {    
    fn new(module_tx: Sender<ReconstructedTransmission>) -> Self {
        Self {
            module_tx
        }
    }

    async fn consume_packet(&mut self, urb_packet: crate::sniffer::UrbXractPacket) {
        let transmission = ReconstructedTransmission {
            urbx_header: urb_packet.header,
            combined_payload: String::from("(Indentified SCSI Packet: Parsing Not Implemented)"),
            sources: vec![],
        };

        /* Transmit Packet */
        self.module_tx.send(transmission).await.unwrap();
    }
}