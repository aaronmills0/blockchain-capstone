use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
use crate::components::utxo::UTXO;
use crate::network::peer::Command;
use crate::network::{decoder, messages};
use crate::simulation::KeyMap;
use crate::utils::hash;
use crate::utils::sign_and_verify::{PrivateKey, PublicKey, Verifier};
use ed25519_dalek::Keypair;
use log::{info, warn};
use mini_redis::Frame;
use rand_1::rngs::ThreadRng;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub async fn test_send_tx_peer_manager(
    sender_id: u32,
    ref_tx_manager: &Sender<Command>,
    num_transactions: u32,
) {
    let mut utxo: UTXO = UTXO(HashMap::new());
    let mut key_map: KeyMap = KeyMap(HashMap::new());
    let mut transactions: Vec<Transaction> = Vec::new();
    let multiplicative_index: u32 = num_transactions;
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

    for _ in 0..multiplicative_index {
        let transaction =
            Transaction::create_transaction(&utxo, &mut key_map, &mut rng, max_num_outputs, false);
        utxo.update(&transaction);
        transactions.push(transaction);
    }

    info!("Sending transactions to peer manager to be verified");

    let mut frames: Vec<Frame> = Vec::new();

    for t in transactions.iter() {
        let frame = messages::get_transaction_msg(sender_id, sender_id, &t.clone());
        frames.push(frame);
    }

    for frame in frames.iter() {
        let (resp_tx, _) = oneshot::channel();
        let tx_clone = ref_tx_manager.clone();

        let json = decoder::decode_json_msg(frame.clone());
        if json.is_none() {
            warn!("Missing json");
            panic!()
        }
        let cmd = Command::Set {
            key: String::from("transaction"),
            resp: resp_tx,
            payload: Some(vec![json.unwrap()]),
        };
        tx_clone.send(cmd).await.ok();
    }
}
