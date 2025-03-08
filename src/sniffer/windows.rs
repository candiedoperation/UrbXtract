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

use std::ptr;
use super::{PacketCaptureImpl, UrbXractHeader, UrbXractPacket};
use pcap::{Capture, Device};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

/* Define Constants, etc. */
pub struct PacketCapture;

impl PacketCaptureImpl for PacketCapture {
    async fn capture_core(device_name: String, tx: tokio::sync::mpsc::Sender<super::UrbXractPacket>) {
         /* Get the Capture Device */
        let device_list = Device::list().unwrap();
        let device = device_list.into_iter()
            .find(|dev| dev.name == device_name)
            .expect(format!("Device {} does not exist!", device_name).as_str());

        /* Configure the Capture */
        let mut capture_stream = 
            Capture::from_device(device)
            .unwrap()
            .promisc(true)
            .open().unwrap();

        /* Capture the Packets and URB Data from PCAP */
        while let Ok(pcap_packet) = capture_stream.next_packet() {
            /* Transmit Packet using Tokio MPSC Channel */
            //tx.send(urb_payload).await.unwrap();
        }
    }
}