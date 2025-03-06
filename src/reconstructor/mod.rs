use std::ptr;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::sniffer::UrbPacket;

#[repr(C, packed)]
#[derive(Debug)]
pub struct CommandBlockWrapper {
    /*
        Strucure Info:
        https://wiki.osdev.org/USB_Mass_Storage_Class_Devices
        https://www.usb.org/sites/default/files/usbmassbulk_10.pdf
    */
    signature: u32,
    tag: u32,
    length: u32,
    direction: u8,
    logical_unitnumber: u8,
    command_length: u8,
    command_data: [u8; 16],
}

#[allow(dead_code)]
#[repr(C, packed)]
pub struct CommandStatusWrapper {
    signature: u32,
    tag: u32,
    residue: u32,
    status: bool,
}

pub struct ReconstructedTransmission {}

/* Define Constants */
const COMMAND_BLK_WRAP_SIGNATURE: u32 = 0x43425355;

async fn consume_core(consume_tx: Sender<ReconstructedTransmission>, mut sniffer_rx: Receiver<UrbPacket>) {
    /* Consume Packets as Sniffer captures them */
    while let Some(urb_packet) = sniffer_rx.recv().await {
        let urb_header = urb_packet.header;
        if urb_packet.data.is_some() {
            /* Get URB Data */
            let urb_data = urb_packet.data.unwrap();
            
            /* Check for CommandBlockWrapper */
            let cbw_packet = unsafe { ptr::read_unaligned(urb_data.as_ptr() as *const CommandBlockWrapper) };
            if cbw_packet.signature == COMMAND_BLK_WRAP_SIGNATURE {
                println!("Got CBW Packet:\n{:?}\n", cbw_packet);
            }
        }

        //println!("URB Packet:\n{:?}, Data: {:?}\n", urb_packet.header, urb_packet.data);
    }
}

pub fn consume(consume_tx: Sender<ReconstructedTransmission>, sniffer_rx: Receiver<UrbPacket>) {
    tokio::spawn(async move {
        /* Call the core-consumer */
        consume_core(consume_tx, sniffer_rx).await;
    });
}
