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

use std::{io::Read, process::Command, ptr};
use crate::ostools;

use super::{PacketCaptureImpl, UrbXractHeader, UrbXractPacket};
use pcap::{Capture, Device};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

/* Define Constants, etc. */
type UsbdStatus = u32;
pub struct PacketCapture;
const USBPCAP_PATH: &str = r"C:\Program Files\Wireshark\extcap\USBPcapCMD.exe";

#[repr(C, packed)]
struct USBPcapBufferPktHeader {
    header_length: u16,
    irp_id: u64,            /* I/O Request Pkt. ID */
    status_code: UsbdStatus,
    urb_function: u16,      /* URB Function */
    request_info: u8,       /* I/O Request Info */
    bus_id: u16,
    device_id: u16,
    endpoint: u8,           /* Endpoint, Transfer Direction */
    xfer_type: u8,
    data_length: u32
}

#[repr(C, packed)]
struct USBPcapBufferIsoHeader {
    pkt_header: USBPcapBufferPktHeader,
    start_frame: u64,
    packet_count: u64,
    error_count: u64,
    offset: u64,
    length: u64,
    status: UsbdStatus
}

impl PacketCaptureImpl for PacketCapture {
    async fn capture_core(device_name: String, tx: tokio::sync::mpsc::Sender<super::UrbXractPacket>) {        
        /* Setup a Named Pipe */
        let capture_pipename = format!(r"\\.\pipe\urbxtract_{}", device_name);
        println!("{}", capture_pipename);
        
        /* Create a stream for USBPcap Communication */
        let mut capture_stream = ostools::windows::create_named_pipe(&capture_pipename, 65536).unwrap();
        
        /* Spawn the USBPcap Process, See: https://www.wireshark.org/docs/wsdg_html_chunked/ChCaptureExtcap.html */
        let mut usbpcap_proc = Command::new(USBPCAP_PATH)
            .args(vec!["--extcap-interface", &format!(r"\\.\{}", device_name), "--capture", "-A", "--fifo", &capture_pipename])
            .spawn()
            .expect("Failed to start USBPcapCMD Process");

        /* Wait for Subprocess to Connect */
        capture_stream.await_clientconnect();
        println!("USBPcapCMD Module Connected to Pipe");

        loop {
            let mut buffer = [0; 2048];
            match capture_stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => println!("{} bytes read", n),
                Err(e) => panic!("Error: {:?}", e),
            }
        }

        // /* Capture the Packets and URB Data from PCAP */
        // while let Ok(pcap_packet) = capture_stream.next_packet() {
        //     /* Transmit Packet using Tokio MPSC Channel */
        //     //tx.send(urb_payload).await.unwrap();
        // }

        /* Terminate the child, just in case! */
        usbpcap_proc.kill().unwrap();
    }
    
    fn get_devices_list() -> Vec<String> {
        let usbpcap_enumlist = Command::new(USBPCAP_PATH)
            .arg("--extcap-interfaces")
            .output()
            .unwrap();

        let devicelist_encoded = String::from_utf8_lossy(&usbpcap_enumlist.stdout);
        return vec![devicelist_encoded.to_string()];
    }
}