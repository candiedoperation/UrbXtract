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

use std::{fs::File, io::{ErrorKind, Read}, time::Duration};

use clap::Parser;
use pcap::Device;
use reconstructor::ReconstructedTransmission;
use sniffer::UrbXractPacket;
use tokio::{sync::mpsc, time::sleep};

#[cfg(target_os="windows")]
use ostools::windows;

#[derive(Parser, Debug)]
struct CLIArgs {
    #[arg(short, long)]
    capture_interface: String
}

#[tokio::main]
async fn main() {
    /* Print License and Available Capture Interface */
    println!("{}", licenses::get_license_string_short());

    let capture_pipename = r"\\.\pipe\helloworldpipe";
    let mut win_pipe = windows::create_named_pipe(capture_pipename, 65536).unwrap();
    win_pipe.await_clientconnect();
    println!("extcap Module Connected to Pipe");

    loop {
        let mut buf = [0; 65536];
        match win_pipe.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => println!("{} bytes read", n),
            Err(e) => panic!("Error: {:?}", e),
        }
        
        // let bytes = win_pipe.read(&mut buf).unwrap();
        //println!("Read: {} bytes", bytes);
    }

    /* Enumerate the Capture Devices */
    // let device_list = Device::list().unwrap();
    // print!("{:?}", device_list);

    /* Parse CLI Args */
    // let cli_args = CLIArgs::parse();

    // /* Create Multi-producer Single-Consumer Channel and start capture */
    // let (sniffer_tx, sniffer_rx) = mpsc::channel::<UrbPacket>(2);
    // let capture_handle = sniffer::capture(cli_args.capture_interface, sniffer_tx);

    // /* Create Channel for Packet Reconstruction and Pass Sniffer Receiver */
    // let (reconstruct_tx, reconstruct_rx) = mpsc::channel::<ReconstructedTransmission>(2);
    // reconstructor::consume(reconstruct_tx, sniffer_rx);

    // /* Create User Interface and start the Render loop */
    // let terminal_interface = ratatui::init();
    // let mut app = textui::UserInterface::new(reconstruct_rx);
    // app.run(terminal_interface).await;

    // /* Reset the Terminal */
    // capture_handle.abort();
    // ratatui::restore();
}