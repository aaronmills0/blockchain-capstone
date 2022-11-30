use crate::block::Block;
use crate::hash;
use crate::merkle::Merkle;
use crate::save_and_load::deserialize_json;
use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
use crate::simulation::KeyMap;
use crate::transaction::{Outpoint, PublicKeyScript, TxOut};
use crate::utxo::UTXO;

use log::{info, warn};
use std::collections::HashMap;
use std::time::Instant;

pub fn validate_chain_performance_test(config_path: &str) {
    let (_, existing_chain, existing_utxo, _, pr_keys, pu_keys, _) = deserialize_json(config_path);
    let (initial_utxo, _) = initial_state(pr_keys, pu_keys);
    display_utxo(&existing_utxo);
    println!("\n-------------------------------\n");
    let start_time = Instant::now();
    if validate_existing_chain(&existing_chain, &existing_utxo, initial_utxo) {
        println!("Chain Validation Test: Validate Blockchain Test Success!\n Chain was found to be valid in {:.2?} for chain of size N = {} blocks.", start_time.elapsed(), existing_chain.len());
    } else {
        info!("Chain Validation Test: Validate Blockchain Test Success!\n Chain was found to be invalid in {:.2?} for chain of size N = {} blocks.", start_time.elapsed(), existing_chain.len());
    }
}

fn validate_existing_chain(
    existing_chain: &[Block],
    existing_utxo: &UTXO,
    mut initial_utxo: UTXO,
) -> bool {
    //Genesis UTXO and Keymap of the chain to be validated
    for block in existing_chain.iter() {
        //println!("prev hash: {}", block.header.previous_hash);
        if block.header.previous_hash == "0".repeat(64)
            || Merkle::create_merkle_tree(&block.transactions)
                .tree
                .first()
                .unwrap()
                != &block.header.merkle_root
        {
            warn!(
                "Chain Validation Test: received a block with invalid merkle root. Ignoring block"
            );
            continue;
        }

        //(_, initial_utxo) = Block::verify_and_update(block.transactions.clone(), initial_utxo);

        // for tx in block.transactions.iter() {
        //     if !initial_utxo.verify_transaction(tx) {
        //         warn!("Chain Validation Test: Existing Chain Contains Eroneus Transactions; test data unusable. Exiting Test");
        //         panic!();
        //     }
        //     initial_utxo.update(tx);

        // }
    }
    if !initial_utxo.utxo_equals(existing_utxo) {
        display_utxo(&initial_utxo);
        println!("\n-----------------------------------------\n");
        display_utxo(existing_utxo);
        return false;
    }

    return true;
}

fn display_utxo(utxo: &UTXO) {
    for (key, value) in utxo.iter() {
        println!("Outpoint: {}\n TxOut: {}", key.txid, value.value);
    }
}
fn initial_state(
    pr_keys: (PrivateKey, PrivateKey),
    pu_keys: (PublicKey, PublicKey),
) -> (UTXO, KeyMap) {
    let mut utxo: UTXO = UTXO(HashMap::new());
    let mut key_map: KeyMap = KeyMap(HashMap::new());

    let (private_key0, private_key1) = pr_keys;
    let (public_key0, public_key1) = pu_keys;

    let outpoint0: Outpoint = Outpoint {
        txid: "0".repeat(64),
        index: 0,
    };

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
