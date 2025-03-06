mod protocol_serial;
mod protocol_scsi;

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

#[derive(Debug)]
pub struct ReconstructedTransmission {
    pub combined_payload: String,
    pub sources: Vec<UrbPacket>,
}

pub trait ReconstructionModule {
    fn new(module_tx: Sender<ReconstructedTransmission>) -> Self;
    async fn consume_packet(&mut self, urb_packet: UrbPacket);
}

/* Define Constants  */
const COMMAND_BLK_WRAP_SIGNATURE: u32 = 0x43425355;

async fn consume_core(consume_tx: Sender<ReconstructedTransmission>, mut sniffer_rx: Receiver<UrbPacket>) {
    /* Enumerate and Define Plugin Modules */
    let mut serial_reconstructor = protocol_serial::Reconstructor::new(consume_tx.clone());
    let mut scsi_reconstructor = protocol_scsi::Reconstructor::new(consume_tx.clone());
    
    /* Consume Packets as Sniffer captures them */
    while let Some(urb_packet) = sniffer_rx.recv().await {        
        if urb_packet.data.is_some() {
            /* Get URB Data */
            let urb_data = urb_packet.data.as_ref().unwrap();
            
            /* Check for CommandBlockWrapper */
            let cbw_packet = unsafe { ptr::read_unaligned(urb_data.as_ptr() as *const CommandBlockWrapper) };
            if cbw_packet.signature == COMMAND_BLK_WRAP_SIGNATURE {
                /* Usually a SCSI Command? */
                scsi_reconstructor.consume_packet(urb_packet).await;
            } else {
                /* Use the Serial Module */
                serial_reconstructor.consume_packet(urb_packet).await;
            }
        }
    }
}

pub fn consume(consume_tx: Sender<ReconstructedTransmission>, sniffer_rx: Receiver<UrbPacket>) {
    tokio::spawn(async move {
        /* Call the core-consumer */
        consume_core(consume_tx, sniffer_rx).await;
    });
}
