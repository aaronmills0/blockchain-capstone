use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
use crate::components::utxo::UTXO;
use crate::network::messages;
use crate::network::peer::get_connection;
use crate::simulation::KeyMap;
use crate::utils::sign_and_verify::Verifier;
use crate::utils::{hash, sign_and_verify};
use rand_1::rngs::ThreadRng;
use std::collections::HashMap;

pub async fn test_single_peer_tx_throughput_sender(
    sender_id: u32,
    ip_map: HashMap<u32, String>,
    ports_map: HashMap<String, Vec<String>>,
    receiver_id: u32,
) {
    let receiver_ip = ip_map.get(&receiver_id).unwrap();
    let receiver_ports = ports_map.get(&receiver_ip.to_owned()).unwrap();

    let ports: Vec<&str> = receiver_ports.iter().map(AsRef::as_ref).collect();
    let connection_opt = get_connection(receiver_ip, ports.as_slice()).await;
    if connection_opt.is_none() {
        panic!("Cannot connect to the receiver peer to send a transaction (test)");
    }
    let mut connection = connection_opt.unwrap();

    let mut utxo: UTXO = UTXO(HashMap::new());
    let mut key_map: KeyMap = KeyMap(HashMap::new());
    let mut transactions: Vec<Transaction> = Vec::new();
    let (private_key, public_key) = sign_and_verify::create_keypair();
    let outpoint: Outpoint = Outpoint {
        txid: "0".repeat(64),
        index: 0,
    };

    let tx_out: TxOut = TxOut {
        value: 500,
        pk_script: PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key),
            verifier: Verifier {},
        },
    };

    key_map.insert(outpoint.clone(), (private_key, public_key));
    utxo.insert(outpoint, tx_out);

    let mut rng: ThreadRng = rand_1::thread_rng();
    let max_num_outputs = 1;

    for _ in 0..131072 {
        let transaction =
            Transaction::create_transaction(&utxo, &mut key_map, &mut rng, max_num_outputs, false);
        utxo.update(&transaction);
        transactions.push(transaction);
    }

    for t in transactions.iter() {
        let frame = messages::get_transaction_msg(sender_id, receiver_id, t.clone());
        connection.write_frame(&frame).await.ok();
    }
}
