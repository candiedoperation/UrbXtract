use reconstructor::{CommandBlockWrapper, ReconstructedTransmission};
use sniffer::UrbPacket;
use tokio::sync::mpsc;

mod sniffer;
mod reconstructor;

#[tokio::main]
async fn main() {
    /* Create Multi-producer Single-Consumer Channel and start capture */
    let (sniffer_tx, sniffer_rx) = mpsc::channel::<UrbPacket>(2);
    sniffer::capture(String::from("usbmon0"), sniffer_tx);

    /* Create Channel for Packet Reconstruction and Pass Sniffer Receiver */
    let (reconstruct_tx, mut reconstruct_rx) = mpsc::channel::<ReconstructedTransmission>(2);
    reconstructor::consume(reconstruct_tx, sniffer_rx);

    /* Consume Packets as Sniffer captures them */
    while let Some(transmission) = reconstruct_rx.recv().await {
        // if (urb_packet_header.bus_id == 3 && urb_packet_header.device == 4 && urb_packet_header.data_length > 0) {
        //     print!("{}", String::from_utf8_lossy(urb_packet_data));
        // }
    }
}