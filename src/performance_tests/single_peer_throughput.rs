use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
use crate::components::utxo::UTXO;
use crate::network::messages;
use crate::network::peer::get_connection;
use crate::simulation::KeyMap;
use crate::utils::hash;
use crate::utils::sign_and_verify::{PrivateKey, PublicKey, Verifier};
use ed25519_dalek::Keypair;
use log::info;
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
    let keypair = Keypair::from_bytes(&[
        9, 75, 189, 163, 133, 148, 28, 198, 139, 3, 56, 182, 118, 26, 250, 201, 129, 109, 104, 32,
        92, 248, 176, 200, 83, 98, 207, 118, 47, 231, 60, 75, 4, 65, 208, 174, 11, 82, 239, 211,
        201, 251, 90, 173, 173, 165, 36, 120, 162, 85, 139, 187, 164, 152, 53, 13, 62, 219, 144,
        86, 74, 205, 134, 25,
    ])
    .unwrap();
    let private_key = PrivateKey(keypair.secret);
    let public_key = PublicKey(keypair.public);
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

    info!("Creating transactions");

    for i in 0..65536 {
        let transaction =
            Transaction::create_transaction(&utxo, &mut key_map, &mut rng, max_num_outputs, false);
        utxo.update(&transaction);
        transactions.push(transaction);
        if i % 1024 == 0 {
            info!("{}", i);
        }
    }

    info!("Sending transactions");

    for t in transactions.iter() {
        let frame = messages::get_transaction_msg(sender_id, receiver_id, &t.clone());
        connection.write_frame(&frame).await.ok();
    }
}
