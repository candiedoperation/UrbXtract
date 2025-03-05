use sniffer::UrbPacket;
use tokio::sync::mpsc;

mod sniffer;

#[tokio::main]
async fn main() {
    /* Create Multi-producer Single-Consumer Channel and start capture */
    let (sniffer_tx, mut sniffer_rx) = mpsc::channel::<UrbPacket>(2);
    sniffer::capture(String::from("usbmon0"), sniffer_tx);

    /* Consume Packets as Sniffer captures them */
    while let Some(urb_packet) = sniffer_rx.recv().await {
        println!("Got URB PAcket!");
    }
}