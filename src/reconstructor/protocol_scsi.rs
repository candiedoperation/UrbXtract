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

    async fn consume_packet(&mut self, urb_packet: crate::sniffer::UrbPacket) {
        let transmission = ReconstructedTransmission {
            combined_payload: String::from("Indentified SCSI Packet. Parsing Not Implemented."),
            sources: vec![],
            bus_id: urb_packet.header.bus_id,
            dev_id: urb_packet.header.device,
            pkt_direction: (urb_packet.header.endpoint & 0x0F != 0),
        };

        /* Transmit Packet */
        self.module_tx.send(transmission).await.unwrap();
    }
}