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

use std::{process::Command, ptr};

use super::{PacketCaptureImpl, UrbXractHeader, UrbXractPacket};
use pcap_parser::{traits::PcapReaderIterator, LegacyPcapReader, PcapError};
use regex::Regex;
use tokio::net::windows::named_pipe::ServerOptions;
use tokio_util::io::SyncIoBridge;

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

fn get_struct_frombytes<T>(data: &[u8]) -> T {
    unsafe {
        let pkt_header: T = ptr::read_unaligned(data.as_ptr() as *const T);
        pkt_header
    }
}

impl PacketCaptureImpl for PacketCapture {
    async fn capture_core(device_name: String, tx: tokio::sync::mpsc::Sender<super::UrbXractPacket>) {        
        /* Setup a Named Pipe */
        let capture_pipename = format!(r"\\.\pipe\urbxtract_{}", device_name);
        let capture_syspipe = 
            ServerOptions::new()
            .in_buffer_size(65536)
            .create(&capture_pipename)
            .unwrap();
        
        /* Spawn the USBPcap Process, See: https://www.wireshark.org/docs/wsdg_html_chunked/ChCaptureExtcap.html */
        let mut _usbpcap_proc = Command::new(USBPCAP_PATH)
            .args(vec!["--extcap-interface", &format!(r"\\.\{}", device_name), "--capture", "-A", "--fifo", &capture_pipename])
            .spawn()
            .expect("Failed to start USBPcapCMD Process");

        /* Wait for Subprocess to Connect, Spawn Tokio Task */
        capture_syspipe.connect().await.unwrap();
        tokio::task::spawn_blocking(move || {
            /* Setup PCAP Stream Parser, See: https://docs.rs/pcap-parser/latest/pcap_parser/pcap/struct.LegacyPcapReader.html#example */
            let capture_syncreader = SyncIoBridge::new(capture_syspipe);
            let mut pcap_stream = LegacyPcapReader::new(65536, capture_syncreader).unwrap();

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
                                let mut end_index = size_of::<USBPcapBufferPktHeader>();
                                let urb_header = get_struct_frombytes::<USBPcapBufferPktHeader>(&legacy_pcap_block.data[0..end_index]);
                                
                                /* Match Transfer Types */
                                match urb_header.xfer_type {
                                    0 => {
                                        /* ISOCHRONOUS Transfer */
                                        end_index = size_of::<USBPcapBufferIsoHeader>(); 
                                    },

                                    2 => {
                                        /* CONTROL Transfer */
                                        end_index = size_of::<USBPcapBufferControlHeader>();
                                    },

                                    _ => { /* Bulk, Interrupt, Invalid Transfer */ },
                                }

                                /* Get URB Payload Data */
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
                                    endpoint_info: urb_header.endpoint
                                };

                                /* Construct UrbXtractPacket */
                                let urbx_packet = UrbXractPacket {
                                    header: urbx_header,
                                    data: urb_data,
                                };

                                tx.blocking_send(urbx_packet).unwrap();
                            },
                        }                  
                        
                        /* Consume the Block */
                        pcap_stream.consume(offset);
                    },
                }
            }
        })
        .await
        .unwrap();
    }
    
    fn get_connected_devices_list(device_name: String) -> Vec<String> {
        let usbpcap_devlist = Command::new(USBPCAP_PATH)
            .arg(format!("--extcap-interface {}", device_name))
            .arg("--extcap-config")
            .output()
            .unwrap();

        // Regex to match lines with parent and extract the display field
        // let re = Regex::new(r"\{display=([^}]+)\}").unwrap();
        // let devices: Vec<String> = re
        //     .captures_iter(&String::from_utf8_lossy(&usbpcap_devlist.stdout))
        //     .map(|cap| cap[1].to_string())
        //     .collect();

        // // Print the result
        // println!("{:?}", devices);
        return vec![]
    }

    fn get_devices_list() -> Vec<String> {
        let usbpcap_enumlist = Command::new(USBPCAP_PATH)
            .arg("--extcap-interfaces")
            .output()
            .unwrap();

        let devicelist_encoded = String::from_utf8_lossy(&usbpcap_enumlist.stdout).to_string();
        let re = Regex::new(r"value=([^}]*)").unwrap();
        let device_names: Vec<String> = re.captures_iter(&devicelist_encoded)
            .map(|cap| cap[1].trim_start_matches(r"\\.\").to_string())
            .collect();

        return device_names;
    }
}