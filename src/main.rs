use std::process;

use reconstructor::ReconstructedTransmission;
use sniffer::UrbPacket;
use tokio::sync::mpsc;

mod textui;
mod sniffer;
mod reconstructor;

#[tokio::main]
async fn main() {
    /* Create Multi-producer Single-Consumer Channel and start capture */
    let (sniffer_tx, sniffer_rx) = mpsc::channel::<UrbPacket>(2);
    let capture_handle = sniffer::capture(String::from("usbmon3"), sniffer_tx);

    /* Create Channel for Packet Reconstruction and Pass Sniffer Receiver */
    let (reconstruct_tx, reconstruct_rx) = mpsc::channel::<ReconstructedTransmission>(2);
    reconstructor::consume(reconstruct_tx, sniffer_rx);

    /* Create User Interface and start the Render loop */
    let terminal_interface = ratatui::init();
    let mut app = textui::UserInterface::new(reconstruct_rx);
    app.run(terminal_interface).await;

    /* Reset the Terminal */
    capture_handle.abort();
    process::exit(0);
}