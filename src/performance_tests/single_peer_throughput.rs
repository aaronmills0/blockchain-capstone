use crate::network::messages;
use crate::network::peer::get_connection;
use crate::shell::get_example_transaction;
use log::info;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

pub async fn test_single_peer_tx_throughput_sender(
    sender_id: u32,
    ip_map: HashMap<u32, String>,
    ports_map: HashMap<String, Vec<String>>,
    receiver_id: u32,
    duration: u64,
) {
    let sleep_time = Duration::from_micros(duration);
    let receiver_ip = ip_map.get(&receiver_id).unwrap();
    let receiver_ports = ports_map.get(&receiver_ip.to_owned()).unwrap();

    let ports: Vec<&str> = receiver_ports.iter().map(AsRef::as_ref).collect();
    let connection_opt = get_connection(receiver_ip, ports.as_slice()).await;
    if connection_opt.is_none() {
        panic!("Cannot connect to the receiver peer to send a transaction (test)");
    }
    let mut connection = connection_opt.unwrap();

    let mut sender_counter: u32 = 0;
    loop {
        let frame =
            messages::get_transaction_msg(sender_id, receiver_id, get_example_transaction());
        connection.write_frame(&frame).await.ok();

        sender_counter += 1;
        info!("Sent {} transactions", sender_counter);

        sleep(sleep_time);
    }
}
