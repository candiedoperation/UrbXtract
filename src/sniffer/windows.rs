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
use pcap_parser::{traits::PcapReaderIterator, LegacyPcapReader, PcapError};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

/* Define Constants, etc. */
type UsbdStatus = u32;
pub struct PacketCapture;
const USBPCAP_PATH: &str = r"C:\Program Files\Wireshark\extcap\USBPcapCMD.exe";

#[allow(dead_code)]
#[repr(C, packed)]
#[derive(Debug)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[repr(C, packed)]
struct USBPcapBufferControlHeader {
    pkt_header: USBPcapBufferPktHeader,
    stage: u8
}

fn get_buffer_pkt_header(data: &[u8]) -> USBPcapBufferPktHeader {
    unsafe {
        let mut pkt_header = ptr::read_unaligned(data.as_ptr() as *const USBPcapBufferPktHeader);
        /* Return Header */
        pkt_header
    }
}

impl PacketCaptureImpl for PacketCapture {
    async fn capture_core(device_name: String, tx: tokio::sync::mpsc::Sender<super::UrbXractPacket>) {        
        /* Setup a Named Pipe */
        let capture_pipename = format!(r"\\.\pipe\urbxtract_{}", device_name);
        let capture_syspipe = ostools::windows::create_named_pipe(&capture_pipename, 65536).unwrap();
        
        /* Spawn the USBPcap Process, See: https://www.wireshark.org/docs/wsdg_html_chunked/ChCaptureExtcap.html */
        let mut _usbpcap_proc = Command::new(USBPCAP_PATH)
            .args(vec!["--extcap-interface", &format!(r"\\.\{}", device_name), "--capture", "-A", "--fifo", &capture_pipename])
            .spawn()
            .expect("Failed to start USBPcapCMD Process");

        /* Wait for Subprocess to Connect */
        capture_syspipe.await_clientconnect();
        println!("USBPcapCMD Module Connected to Pipe");

        /* Setup PCAP Stream Parser, See: https://docs.rs/pcap-parser/latest/pcap_parser/pcap/struct.LegacyPcapReader.html#example */
        let mut pcap_stream = LegacyPcapReader::new(65536, capture_syspipe).unwrap();
        loop {
            match pcap_stream.next() {
                Err(PcapError::Eof) => break,
                Err(PcapError::Incomplete(_)) => { pcap_stream.refill().unwrap(); },
                Err(e) => panic!("Error while reading: {:?}", e),

                Ok((offset, block)) => {
                    match block {
                        pcap_parser::PcapBlockOwned::LegacyHeader(_pcap_header) => { },
                        pcap_parser::PcapBlockOwned::NG(_block) => { },

                        pcap_parser::PcapBlockOwned::Legacy(legacy_pcap_block) => {
                            let end_index = size_of::<USBPcapBufferPktHeader>();
                            let urb_header = get_buffer_pkt_header(&legacy_pcap_block.data[0..end_index]);
                            
                            let urb_data = 
                                if urb_header.data_length < 1 { None }
                                else { 
                                    Some(legacy_pcap_block.data[
                                        end_index..(end_index + urb_header.data_length as usize)
                                    ].to_vec()) 
                                };
                            
                            /* Construct UrbXtractHeader */
                            let urbx_header = UrbXractHeader {
                                bus_id: urb_header.bus_id,
                                device_id: urb_header.device_id,
                            };

                            /* Construct UrbXtractPacket */
                            let urbx_packet = UrbXractPacket {
                                header: urbx_header,
                                data: urb_data,
                            };

                            println!("URB Info:\n{:?}\n", urbx_packet);
                            //tx.send(urb_payload).await.unwrap();
                        },
                    }                  
                    
                    /* Consume the Block */
                    pcap_stream.consume(offset);
                },
            }
        }
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