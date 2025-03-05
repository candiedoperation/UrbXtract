use std::ptr;

use pcap::{Capture, Device};
use tokio::sync::mpsc::Sender;


#[repr(C, packed)]
pub(crate) struct RawUrbHeader {
    /* This struct is packed, Convert to UrbHeader while using */
    pub(crate) id: [u8; 8],
    pub(crate) type_: u8,
    pub(crate) transfer_type: u8,
    pub(crate) endpoint: u8,
    pub(crate) device: u8,
    pub(crate) bus_id: [u8; 2],
    pub(crate) setup_flag: u8,
    pub(crate) data_flag: u8,
    pub(crate) timestamp_sec: [u8; 8],
    pub(crate) timestamp_usec: [u8; 4],
    pub(crate) status: [u8; 4],
    pub(crate) urb_length: [u8; 4],
    pub(crate) data_length: [u8; 4],
    pub(crate) setup_iso_union: [u8; 8],
    pub(crate) interval: [u8; 4],
    pub(crate) start_frame: [u8; 4],
    pub(crate) xfer_flags: [u8; 4],
    pub(crate) iso_ndesc: [u8; 4],
}

#[derive(Debug)]
pub struct UrbHeader {
    pub id: u64,
    pub type_: u8,
    pub transfer_type: u8,
    pub endpoint: u8,
    pub device: u8,
    pub bus_id: u16,
    pub setup_flag: char,
    pub data_flag: char,
    pub timestamp_sec: i64,
    pub timestamp_usec: i32,
    pub status: i32,
    pub urb_length: u32,
    pub data_length: u32,
    pub setup_iso: [u8; 8],
    pub interval: i32,
    pub start_frame: i32,
    pub xfer_flags: u32,
    pub iso_ndesc: u32
}

impl From<RawUrbHeader> for UrbHeader {
    fn from(raw_urbheader: RawUrbHeader) -> Self {
        Self {
            id: u64::from_ne_bytes(raw_urbheader.id),
            type_: raw_urbheader.type_,
            transfer_type: raw_urbheader.transfer_type,
            endpoint: raw_urbheader.endpoint,
            device: raw_urbheader.device,
            bus_id: u16::from_ne_bytes(raw_urbheader.bus_id),
            setup_flag: (raw_urbheader.setup_flag as char),
            data_flag: (raw_urbheader.data_flag as char),
            timestamp_sec: i64::from_ne_bytes(raw_urbheader.timestamp_sec),
            timestamp_usec: i32::from_ne_bytes(raw_urbheader.timestamp_usec),
            status: i32::from_ne_bytes(raw_urbheader.status),
            urb_length: u32::from_ne_bytes(raw_urbheader.urb_length),
            data_length: u32::from_ne_bytes(raw_urbheader.data_length),
            setup_iso: raw_urbheader.setup_iso_union,
            interval: i32::from_ne_bytes(raw_urbheader.interval),
            start_frame: i32::from_ne_bytes(raw_urbheader.start_frame),
            xfer_flags: u32::from_ne_bytes(raw_urbheader.xfer_flags),
            iso_ndesc: u32::from_ne_bytes(raw_urbheader.iso_ndesc),
        }
    }
}

pub struct UrbPacket {
    header: UrbHeader,
    data: Option<Vec<u8>>
}

/* Define Constants, Instance Variables */
const URB_PACKET_HDRLEN: usize = size_of::<RawUrbHeader>();


fn read_urb_header(data: &[u8]) -> UrbHeader {
    unsafe {
        let raw_header = ptr::read_unaligned(data.as_ptr() as *const RawUrbHeader);
        return raw_header.into();
    }
}

async fn capture_core(device_name: String, tx: Sender<UrbPacket>) {
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
        let urb_packet_header = read_urb_header(&pcap_packet.data[0..URB_PACKET_HDRLEN]);
        let urb_data_length = urb_packet_header.data_length as usize;
        
        /* Construct Payload Structure for Async Transmission */
        let urb_payload = UrbPacket {
            header: urb_packet_header,
            data: if urb_data_length > 0 { 
                /* Get Appropriate Data Region */
                let urb_packet_data = &pcap_packet.data[URB_PACKET_HDRLEN..(URB_PACKET_HDRLEN + urb_data_length)];
                Some(urb_packet_data.to_vec()) 
            } else {
                /* There's no Data */ 
                None 
            },
        };

        /* Transmit Packet using Tokio MPSC Channel */
        tx.send(urb_payload).await.unwrap();
    }
}

pub fn capture(device_name: String, tx: Sender<UrbPacket>) {
    tokio::spawn(async move {
        capture_core(device_name, tx).await;
    });
}