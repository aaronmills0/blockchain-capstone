use crate::network::messages;
use crate::network::peer;
use crate::shell::get_example_transaction;
use log::info;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

static mut counter: u32 = 0;

pub async fn test_single_peer_tx_throughput_sender(
    sending_id: u32,
    ip_map: HashMap<u32, String>,
    ports_map: HashMap<String, Vec<String>>,
    receiving_id: u32,
    duration: u64,
) {
    let sleep_time = Duration::from_micros(duration);
    let receiving_ip = ip_map.get(&receiving_id).unwrap();
    let receiving_ports = ports_map.get(&receiving_ip.to_owned()).unwrap();

    let mut sender_counter: u32 = 0;
    loop {
        let frame =
            messages::get_transaction_msg(sending_id, receiving_id, get_example_transaction());
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
