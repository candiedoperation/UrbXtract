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


use clap::{Command, CommandFactory, Parser};
use reconstructor::ReconstructedTransmission;
use sniffer::{PacketCaptureImpl, UrbXractPacket};
use tokio::sync::mpsc;
use sniffer::PacketCapture;

#[derive(Parser, Debug)]
#[command(name = "urbxtract")]
struct CLIArgs {
    #[arg(short, long, help="Specify Capture Interface (Required)")]
    iface: Option<String>,
    
    #[arg(long, help="Show License Information")]
    license_info: bool
}

#[tokio::main]
async fn main() {
    /* Parse CLI Args */
    let cli_args = CLIArgs::parse();
    
    if cli_args.license_info {
        println!("\n{}", licenses::get_license_string_full());
        return;
    }
    
    /* Print License and Available Capture Interface */
    println!("\n{}\n", licenses::get_license_string_short());

    if cli_args.iface.is_none() {
        /* Enumerate the Capture Devices */
        println!(
            "Available Capture Interfaces:\n{}\n", 
            PacketCapture::get_devices_list()
                .iter()
                .fold(String::from(""), |acc, dev| acc + &format!("✲  {}\n", dev))
        );

        /* Print Usage and return */
        let mut cmd = CLIArgs::command();
        eprintln!("{}\n", cmd.render_long_help());
        return;
    }

    /* Create Multi-producer Single-Consumer Channel and start capture */
    let (sniffer_tx, sniffer_rx) = mpsc::channel::<UrbXractPacket>(2);
    let capture_handle = sniffer::capture(cli_args.iface.unwrap(), sniffer_tx);

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