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

mod ostools;
mod textui;
mod sniffer;
mod licenses;
mod reconstructor;


use clap::Parser;
use ratatui::Terminal;
use reconstructor::ReconstructedTransmission;
use sniffer::{PacketCaptureImpl, UrbXractPacket};
use tokio::sync::mpsc;
use sniffer::PacketCapture;

#[derive(Parser, Debug)]
struct CLIArgs {
    #[arg(short, long)]
    capture_interface: String
}

#[tokio::main]
async fn main() {
    /* Print License and Available Capture Interface */
    println!("{}", licenses::get_license_string_short());

    /* Enumerate the Capture Devices */
    println!("Available Devices:\n{:?}\n", PacketCapture::get_devices_list());

    /* Parse CLI Args */
    let cli_args = CLIArgs::parse();

    /* Create Multi-producer Single-Consumer Channel and start capture */
    let (sniffer_tx, sniffer_rx) = mpsc::channel::<UrbXractPacket>(2);
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