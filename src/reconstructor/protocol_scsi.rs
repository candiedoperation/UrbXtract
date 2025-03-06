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
        todo!("SCSI Parsing Not Implemented")
    }
}