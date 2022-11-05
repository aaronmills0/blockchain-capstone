use chrono::Local;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_json::Value;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::sign_and_verify::PrivateKey;
use crate::sign_and_verify::PublicKey;
use crate::simulation::KeyMap;
use crate::transaction::Outpoint;
use crate::{block::Block, utxo::UTXO};

#[derive(Serialize, Deserialize)]
pub struct Config {
    block_mean: f32,
    block_multiplier: u32,
    block_size: u32,
    max_tx_outputs: usize,
    tx_mean: f32,
    tx_multiplier: u32,
    invalid_tx_mean: f32,
    invalid_tx_sigma: f32,
}

pub fn serialize_json(
    blockchain: Vec<Block>,
    utxo: UTXO,
    keymap: KeyMap,
    sim_config: Config,
    file_prefix: Option<String>,
) {
    let blockchain_json = serde_json::to_value(&blockchain);
    let utxo_json = serde_json::to_value(&utxo);
    let keymap_json = serde_json::to_value(&keymap);
    let config_json = serde_json::to_value(&sim_config);

    if blockchain_json.is_err() {
        error!("Failed to serialize blocks!");
    }

    if utxo_json.is_err() {
        error!("Failed to serialize utxo!");
    }

    if keymap_json.is_err() {
        error!("Failed to serialize utxo!");
    }

    if config_json.is_err() {
        error!("Failed to serialize simulation configuration data!");
    }

    let mut map = Map::new();
    map.insert(String::from("blockchain"), blockchain_json.unwrap());
    map.insert(String::from("utxo"), utxo_json.unwrap());
    map.insert(String::from("keymap"), keymap_json.unwrap());
    map.insert(String::from("config"), config_json.unwrap());

    let json: Value = serde_json::Value::Object(map);

    if env::consts::OS == "windows" {
        if fs::create_dir("config\\").is_err() {
            error!("Failed to create directory!");
        }
    } else {
        if fs::create_dir("config/").is_err() {
            error!("Failed to create directory!");
        }
    }

    let cwd = std::env::current_dir().unwrap();

    let mut dirpath = cwd.into_os_string().into_string().unwrap();
    dirpath.push_str("/config");

    let dir_path = Path::new(&dirpath);
    let date_time = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    // The new file_name is timestamped
    let mut prefix = String::new();
    if file_prefix.is_some() {
        prefix.push_str(&file_prefix.unwrap());
    }
    let file_name: &str = &format!("{}_{}.json", prefix, date_time);

    let file_path = dir_path.join(file_name);
    let file = File::create(file_path);
    if file.is_err() {
        error!("Failed to create new file.");
        panic!();
    }
    if file
        .unwrap()
        .write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
        .is_err()
    {
        error!("Failed to write to file.");
        panic!();
    }
}

pub fn deserialize_json(
    filepath: &str,
) -> (
    Vec<Block>,
    UTXO,
    Vec<(Outpoint, (PrivateKey, PublicKey))>,
    Config,
) {
    let data = fs::read_to_string(filepath);
    if data.is_err() {
        println!("Failed to load file. {:?}", data.err());
        //error!("Failed to load file. {:?}", data.err());
        panic!();
    }
    let json: Value = serde_json::from_str(&data.unwrap()).unwrap();

    let blockchain_json = json
        .get("blockchain")
        .unwrap()
        .as_array()
        .unwrap()
        .to_owned();
    for block in blockchain_json.clone() {
        println!("{}", block);
    }
    let utxo_json = json.get("utxo").unwrap().to_owned();
    let keymap_json = json.get("keymap").unwrap().as_array().unwrap().to_owned();
    let config_json = json.get("config").unwrap().to_owned();

    let mut blockchain: Vec<Block> = Vec::new();
    for block in blockchain_json {
        blockchain.push(serde_json::from_value(block).unwrap());
    }
    let utxo = serde_json::from_value(utxo_json.clone());
    let mut keymap: Vec<(Outpoint, (PrivateKey, PublicKey))> = Vec::new();
    for pair in keymap_json.clone() {
        keymap.push(serde_json::from_value(pair).unwrap());
    }
    let config = serde_json::from_value(config_json.clone());

    if utxo.is_err() {
        error!("Failed to deserialize utxo! {:?}", utxo.err());
        panic!();
    }

    if config.is_err() {
        error!(
            "Failed to deserialize simulation configuration! {:?}",
            config.err()
        );
        panic!();
    }

    return (blockchain, utxo.unwrap(), keymap, config.unwrap());
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::block::{Block, BlockHeader};
    use crate::hash;
    use crate::merkle::Merkle;
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::transaction::{
        Outpoint, PublicKeyScript, SignatureScript, Transaction, TxIn, TxOut,
    };
    use std::collections::HashMap;
    #[test]
    fn test_serialize_json() {
        // Creation and initialization of the UTXO

        let mut utxo: UTXO = UTXO(HashMap::new());

        let mut keymap: KeyMap = KeyMap(HashMap::new());

        let (private_key00, public_key00) = sign_and_verify::create_keypair();
        let outpoint00: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };
        let (private_key01, public_key01) = sign_and_verify::create_keypair();
        let outpoint01: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 1,
        };

        let tx_out00: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key00),
                verifier: Verifier {},
            },
        };

        let tx_out01: TxOut = TxOut {
            value: 850,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key01),
                verifier: Verifier {},
            },
        };

        utxo.insert(outpoint00.clone(), tx_out00);
        utxo.insert(outpoint01.clone(), tx_out01);

        let mut blockchain: Vec<Block> = Vec::new();
        let mut transactions1: Vec<Transaction> = Vec::new();
        let mut transactions2: Vec<Transaction> = Vec::new();

        // Create a single new transaction (2 inputs and 1 output)
        let mut tx_inputs1: Vec<TxIn> = Vec::new();
        let mut tx_outputs1: Vec<TxOut> = Vec::new();
        let sig_script10: SignatureScript;
        let private_key10: PrivateKey;
        let public_key10: PublicKey;
        let message10: String;
        let pk_script10: PublicKeyScript;
        let sig_script11: SignatureScript;
        let message11: String;

        message10 = String::from(&outpoint00.txid)
            + &outpoint00.index.to_string()
            + &utxo[&outpoint00].pk_script.public_key_hash;

        sig_script10 = SignatureScript {
            signature: sign_and_verify::sign(&message10, &private_key00),
            full_public_key: public_key00,
        };

        tx_inputs1.push(TxIn {
            outpoint: outpoint00,
            sig_script: sig_script10,
        });

        message11 = String::from(&outpoint01.txid)
            + &outpoint01.index.to_string()
            + &utxo[&outpoint01].pk_script.public_key_hash;

        sig_script11 = SignatureScript {
            signature: sign_and_verify::sign(&message11, &private_key01),
            full_public_key: public_key01,
        };

        tx_inputs1.push(TxIn {
            outpoint: outpoint01,
            sig_script: sig_script11,
        });

        (private_key10, public_key10) = sign_and_verify::create_keypair();

        pk_script10 = PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key10),
            verifier: Verifier {},
        };

        tx_outputs1.push(TxOut {
            value: 1350,
            pk_script: pk_script10,
        });

        let transaction1 = Transaction {
            tx_inputs: tx_inputs1,
            tx_outputs: tx_outputs1,
        };

        let outpoint10 = Outpoint {
            txid: hash::hash_as_string(&transaction1),
            index: 0 as u32,
        };

        utxo.verify_transaction(&transaction1);
        utxo.update(&transaction1);
        transactions1.push(transaction1);

        // Create genesis block and first block

        let genesis_merkle: Merkle = Merkle {
            tree: Vec::from(["0".repeat(64).to_string()]),
        };
        let genesis_block: Block = Block {
            header: BlockHeader {
                previous_hash: "0".repeat(64).to_string(),
                merkle_root: genesis_merkle.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: genesis_merkle,
            transactions: Vec::new(),
        };

        blockchain.push(genesis_block);

        let merkle1 = Merkle::create_merkle_tree(&transactions1);

        let block1 = Block {
            header: BlockHeader {
                previous_hash: hash::hash_as_string(&blockchain.last().unwrap().header),
                merkle_root: merkle1.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: merkle1,
            transactions: transactions1,
        };
        blockchain.push(block1);

        // Create a second transaction and second block.
        // Second transaction will have one input and two outputs.

        let mut tx_inputs2: Vec<TxIn> = Vec::new();
        let mut tx_outputs2: Vec<TxOut> = Vec::new();
        let sig_script20: SignatureScript;
        let private_key20: PrivateKey;
        let public_key20: PublicKey;
        let message20: String;
        let pk_script20: PublicKeyScript;
        let pk_script21: PublicKeyScript;
        let private_key21: PrivateKey;
        let public_key21: PublicKey;

        message20 = String::from(&outpoint10.txid)
            + &outpoint10.index.to_string()
            + &utxo[&outpoint10].pk_script.public_key_hash;

        sig_script20 = SignatureScript {
            signature: sign_and_verify::sign(&message20, &private_key10),
            full_public_key: public_key10,
        };

        tx_inputs2.push(TxIn {
            outpoint: outpoint10,
            sig_script: sig_script20,
        });

        (private_key20, public_key20) = sign_and_verify::create_keypair();

        pk_script20 = PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key20),
            verifier: Verifier {},
        };

        tx_outputs2.push(TxOut {
            value: 350,
            pk_script: pk_script20,
        });

        (private_key21, public_key21) = sign_and_verify::create_keypair();

        pk_script21 = PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key21),
            verifier: Verifier {},
        };

        tx_outputs2.push(TxOut {
            value: 1000,
            pk_script: pk_script21,
        });

        let transaction2 = Transaction {
            tx_inputs: tx_inputs2,
            tx_outputs: tx_outputs2,
        };

        let outpoint20 = Outpoint {
            txid: hash::hash_as_string(&transaction2),
            index: 0 as u32,
        };

        let outpoint21 = Outpoint {
            txid: hash::hash_as_string(&transaction2),
            index: 1 as u32,
        };

        utxo.verify_transaction(&transaction2);
        utxo.update(&transaction2);
        transactions2.push(transaction2);

        let merkle2 = Merkle::create_merkle_tree(&transactions2);

        let block2 = Block {
            header: BlockHeader {
                previous_hash: hash::hash_as_string(&blockchain.last().unwrap().header),
                merkle_root: merkle2.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: merkle2,
            transactions: transactions2,
        };
        blockchain.push(block2);

        keymap.insert(outpoint20, (private_key20, public_key20));
        keymap.insert(outpoint21, (private_key21, public_key21));

        let config: Config = Config {
            block_mean: 1.0,
            block_multiplier: 10,
            block_size: 1,
            max_tx_outputs: 4,
            tx_mean: 1.0,
            tx_multiplier: 10,
            invalid_tx_mean: 1.0,
            invalid_tx_sigma: 1.0,
        };

        serialize_json(blockchain, utxo, keymap, config, Some(String::from("test")));
    }

    #[test]
    fn test_deserialize_json() {
        let path = "config/test_2022-11-04-12-21-22.json";

        let (blockchain, utxo, keymap, config) = deserialize_json(path);
    }
}
