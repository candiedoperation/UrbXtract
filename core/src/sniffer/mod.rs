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

use tokio::sync::mpsc::Sender;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use linux::PacketCapture;
    } else if #[cfg(target_os = "windows")] {
        pub mod windows;
        pub use windows::PacketCapture;
    }
}

#[derive(Debug)]
pub struct UrbXractHeader {
    pub bus_id: u16,
    pub device_id: u16,
    pub endpoint_info: u8
}

#[derive(Debug)]
pub struct UrbXractPacket {
    pub header: UrbXractHeader,
    pub data: Option<Vec<u8>>
}

pub(crate) trait PacketCaptureImpl {
    async fn capture_core(device_name: String, tx: Sender<UrbXractPacket>);
    fn get_devices_list() -> Vec<String>;
}

pub fn capture(device_name: String, tx: Sender<UrbXractPacket>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        PacketCapture::capture_core(device_name, tx).await;
    })
}