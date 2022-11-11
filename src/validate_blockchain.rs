use crate::block::Block;
use crate::hash;
use crate::save_and_load::deserialize_json;
use crate::sign_and_verify;
use crate::sign_and_verify::Verifier;
use crate::simulation::KeyMap;
use crate::transaction::{Outpoint, PublicKeyScript, TxOut};
use crate::utxo::UTXO;

use log::{info, warn};
use std::collections::HashMap;
use std::time::Instant;

pub fn validate_chain_performance_test(config_path: &str) {
    let start_time = Instant::now();
    let (_, existing_chain, existing_utxo, _, _) = deserialize_json(config_path);
    if validate_existing_chain(&existing_chain, &existing_utxo) {
        info!("Validate Blockchain Test Success!\n Chain was found to be valid in {:.2?}s for chain of size N = {} blocks.", start_time.elapsed(), existing_chain.len());
    } else {
        info!("Validate Blockchain Test Success!\n Chain was found to be invalid in {:.2?}s for chain of size N = {} blocks.", start_time.elapsed(), existing_chain.len());
    }
}
//STILL NEED TO WRITE UNIT TEST FOR UTXOEQUALS METHOD
fn validate_existing_chain(existing_chain: &[Block], existing_utxo: &UTXO) -> bool {
    //Genesis UTXO and Keymap of the chain to be validated
    let (mut utxo, _) = initial_state();
    for block in existing_chain.iter() {
        for tx in block.transactions.iter() {
            if !utxo.verify_transaction(tx) {
                warn!("Existing Chain Contains Eroneus Transactions; test data unusable. Exiting Test");
                panic!("Existing Chain Contains Eroneus Transactions; test data unusable");
            }
            utxo.update(tx);
        }
    }
    if !utxo.utxo_equals(existing_utxo) {
        return false;
    }
    return true;
}

fn initial_state() -> (UTXO, KeyMap) {
    let mut utxo: UTXO = UTXO(HashMap::new());
    let mut key_map: KeyMap = KeyMap(HashMap::new());

    let (private_key0, public_key0) = sign_and_verify::create_keypair();
    let outpoint0: Outpoint = Outpoint {
        txid: "0".repeat(64),
        index: 0,
    };
    let (private_key1, public_key1) = sign_and_verify::create_keypair();
    let outpoint1: Outpoint = Outpoint {
        txid: "0".repeat(64),
        index: 1,
    };

    let tx_out0: TxOut = TxOut {
        value: 500,
        pk_script: PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key0),
            verifier: Verifier {},
        },
    };

    let tx_out1: TxOut = TxOut {
        value: 850,
        pk_script: PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key1),
            verifier: Verifier {},
        },
    };

    key_map.insert(outpoint0.clone(), (private_key0, public_key0));
    key_map.insert(outpoint1.clone(), (private_key1, public_key1));

    utxo.insert(outpoint0, tx_out0);
    utxo.insert(outpoint1, tx_out1);

    return (utxo, key_map);
}
