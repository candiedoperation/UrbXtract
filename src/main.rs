use std::process;

use clap::Parser;
use reconstructor::ReconstructedTransmission;
use sniffer::UrbPacket;
use tokio::sync::mpsc;

mod textui;
mod sniffer;
mod reconstructor;

#[derive(Parser, Debug)]
struct CLIArgs {
    #[arg(short, long)]
    capture_interface: String
}

#[tokio::main]
async fn main() {
    /* Parse CLI Args */
    let cli_args = CLIArgs::parse();

    /* Create Multi-producer Single-Consumer Channel and start capture */
    let (sniffer_tx, sniffer_rx) = mpsc::channel::<UrbPacket>(2);
    let capture_handle = sniffer::capture(cli_args.capture_interface, sniffer_tx);

    /* Create Channel for Packet Reconstruction and Pass Sniffer Receiver */
    let (reconstruct_tx, reconstruct_rx) = mpsc::channel::<ReconstructedTransmission>(2);
    reconstructor::consume(reconstruct_tx, sniffer_rx);

    /* Create User Interface and start the Render loop */
    let terminal_interface = ratatui::init();
    let mut app = textui::UserInterface::new(reconstruct_rx);
    app.run(terminal_interface).await;

    /* Reset the Terminal */
    capture_handle.abort();
    ratatui::restore();
}