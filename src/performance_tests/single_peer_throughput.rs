use crate::network::messages;
use crate::network::peer::{self, Peer};
use crate::shell::get_example_transaction;
use log::info;
use std::thread::sleep;
use std::time::Duration;

static mut counter: u32 = 0;

async fn test_single_peer_tx_throughput_sender(
    sending_peer: &Peer,
    receiving_id: u32,
    duration: u64,
) {
    let sleep_time = Duration::from_micros(duration);
    let sending_peerid = sending_peer.peerid;
    let receiving_ip = sending_peer.ip_map.get(&receiving_id).unwrap();
    let receiving_ports = sending_peer.port_map.get(&receiving_ip.to_owned()).unwrap();

    let mut sender_counter: u32 = 0;
    loop {
        let frame =
            messages::get_transaction_msg(sending_peerid, receiving_id, get_example_transaction());
        peer::send_transaction(frame, receiving_ip.to_owned(), receiving_ports.to_owned()).await;

        sender_counter += 1;
        info!("Sent {} transactions", sender_counter);

        sleep(sleep_time);
    }
}

pub fn test_single_peer_tx_throughput_receiver() {
    unsafe {
        counter += 1;
        info!("Received {} transactions", counter);
    }
}
