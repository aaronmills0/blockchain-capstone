use crate::components::block::Block;
use crate::components::transaction::{Outpoint, TxOut};
use crate::components::utxo::UTXO;
use crate::simulation::KeyMap;
use crate::utils::sign_and_verify::{PrivateKey, PublicKey};
use chrono::Local;
use log::{error, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{env, fs};

#[allow(clippy::too_many_arguments)]
pub fn serialize_json(
    blockchain: &Vec<Block>,
    file_prefix: Option<String>,
    initial_tx_outs: &Vec<TxOut>,
    keymap: &KeyMap,
    pr_keys: &(PrivateKey, PrivateKey),
    pu_keys: &(PublicKey, PublicKey),
    sim_config: &Config,
    utxo: &UTXO,
) {
    let blockchain_json = serde_json::to_value(blockchain);
    let config_json = serde_json::to_value(sim_config);
    let initial_tx_outs_json = serde_json::to_value(initial_tx_outs);
    let keymap_json = serde_json::to_value(keymap);
    let pr_keys_json = serde_json::to_value(pr_keys);
    let pu_keys_json = serde_json::to_value(pu_keys);
    let utxo_json = serde_json::to_value(utxo);

    if blockchain_json.is_err() {
        error!("Failed to serialize blocks!");
        panic!();
    }

    if config_json.is_err() {
        error!("Failed to serialize simulation configuration data!");
        panic!();
    }

    if initial_tx_outs_json.is_err() {
        error!("Failed to serialize initial tx outs!");
        panic!();
    }

    if keymap_json.is_err() {
        error!("Failed to serialize keymap!");
        panic!();
    }

    if pr_keys_json.is_err() {
        error!("Failed to serialize pr_keys!");
        panic!();
    }

    if pu_keys_json.is_err() {
        error!("Failed to serialize pu_keys!");
        panic!();
    }

    if utxo_json.is_err() {
        error!("Failed to serialize utxo!");
        panic!();
    }

    let mut map = Map::new();
    map.insert(String::from("blockchain"), blockchain_json.unwrap());
    map.insert(String::from("config"), config_json.unwrap());
    map.insert(
        String::from("initial tx outs"),
        initial_tx_outs_json.unwrap(),
    );
    map.insert(String::from("keymap"), keymap_json.unwrap());
    map.insert(String::from("pr_keys"), pr_keys_json.unwrap());
    map.insert(String::from("pu_keys"), pu_keys_json.unwrap());
    map.insert(String::from("utxo"), utxo_json.unwrap());

    let json: Value = serde_json::Value::Object(map);

    let slash = if env::consts::OS == "windows" {
        "\\"
    } else {
        "/"
    };
    if fs::create_dir_all("config".to_owned() + slash).is_err() {
        warn!("Failed to create directory! It may already exist, or permissions are needed.");
    }

    let cwd = std::env::current_dir().unwrap();
    let mut dirpath = cwd.into_os_string().into_string().unwrap();
    dirpath.push_str("/config");

    let dir_path = Path::new(&dirpath);
    let date_time = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    let mut prefix = String::new();
    if let Some(prefix1) = file_prefix {
        prefix.push_str(&prefix1);
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

#[allow(clippy::type_complexity)]
pub fn deserialize_json(
    filepath: &str,
) -> (
    Vec<Block>,
    Config,
    Vec<TxOut>,
    KeyMap,
    (PrivateKey, PrivateKey),
    (PublicKey, PublicKey),
    UTXO,
) {
    let data = fs::read_to_string(filepath);
    if data.is_err() {
        error!("Failed to load file. {:?}", data.err());
        panic!();
    }
    let json: Value = serde_json::from_str(&data.unwrap()).unwrap();

    let blockchain_json = json
        .get("blockchain")
        .unwrap()
        .as_array()
        .unwrap()
        .to_owned();
    let config_json = json.get("config").unwrap().to_owned();
    let initial_tx_outs_json = json
        .get("initial tx outs")
        .unwrap()
        .as_array()
        .unwrap()
        .to_owned();
    let keymap_json = json.get("keymap").unwrap().as_array().unwrap().to_owned();
    let pr_keys_json = json.get("pr_keys").unwrap().to_owned();
    let pu_keys_json = json.get("pu_keys").unwrap().to_owned();
    let utxo_json = json.get("utxo").unwrap().to_owned();

    let mut blockchain: Vec<Block> = Vec::new();
    for block in blockchain_json {
        blockchain.push(serde_json::from_value(block).unwrap());
    }
    let config = serde_json::from_value(config_json);
    let mut initial_tx_outs: Vec<TxOut> = Vec::new();
    for tx_out in initial_tx_outs_json {
        initial_tx_outs.push(serde_json::from_value(tx_out).unwrap());
    }
    let mut keymap: Vec<(Outpoint, (PrivateKey, PublicKey))> = Vec::new();
    for pair in keymap_json {
        keymap.push(serde_json::from_value(pair).unwrap());
    }
    let pr_keys = serde_json::from_value(pr_keys_json);
    let pu_keys = serde_json::from_value(pu_keys_json);
    let utxo = serde_json::from_value(utxo_json);

    if config.is_err() {
        error!(
            "Failed to deserialize simulation configuration! {:?}",
            config.err()
        );
        panic!();
    }

    if utxo.is_err() {
        error!("Failed to deserialize utxo! {:?}", utxo.err());
        panic!();
    }

    let mut key_map = KeyMap(HashMap::new());
    for (outpoint, key_pair) in keymap {
        key_map.insert(outpoint, key_pair);
    }

    return (
        blockchain,
        config.unwrap(),
        initial_tx_outs,
        key_map,
        pr_keys.unwrap(),
        pu_keys.unwrap(),
        utxo.unwrap(),
    );
}

pub fn save_object<T: Serialize>(obj: &T, object_name: String, dirname: String) {
    let mut map = Map::new();
    let json_result = serde_json::to_value(obj);

    if json_result.is_err() {
        error!("Failed to serialize peer");
        panic!();
    }

    let mut json = json_result.unwrap();

    map.insert(String::from(object_name.clone()), json);

    json = serde_json::Value::Object(map);

    let slash = if env::consts::OS == "windows" {
        "\\"
    } else {
        "/"
    };
    if fs::create_dir_all(dirname.to_owned() + slash).is_err() {
        warn!("Failed to create directory! It may already exist, or permissions are needed.");
    }

    let cwd = std::env::current_dir().unwrap();
    let mut dirpath = cwd.into_os_string().into_string().unwrap();
    dirpath.push_str(&(slash.to_owned() + &dirname));

    let dir_path = Path::new(&dirpath);

    let file_name: &str = &format!("{}{}", object_name, String::from(".json"));

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

pub fn load_object<T: DeserializeOwned>(object_name: String, dirname: String) -> T {
    let slash = if env::consts::OS == "windows" {
        "\\"
    } else {
        "/"
    };
    let data = fs::read_to_string(dirname.to_owned() + slash + &object_name + ".json");
    if data.is_err() {
        error!("Failed to load file. {:?}", data.err());
        panic!();
    }
    let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
    let object = serde_json::from_value(json.get(object_name).unwrap().to_owned());
    return object.unwrap();
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub block_duration: u32,
    pub block_mean: f32,
    pub block_size: u32,
    pub invalid_tx_mean_ratio: u32,
    pub max_tx_outputs: usize,
    pub tx_duration: u32,
    pub tx_mean: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::block::{Block, BlockHeader};
    use crate::components::merkle::Merkle;
    use crate::components::transaction::{
        Outpoint, PublicKeyScript, SignatureScript, Transaction, TxIn, TxOut,
    };
    use crate::utils::sign_and_verify::Verifier;
    use crate::utils::{hash, sign_and_verify};
    use std::collections::HashMap;

    #[test]
    fn test_serialize_json() {
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

        let pr_keys = (private_key00.clone(), private_key01.clone());
        let pu_keys = (public_key00.clone(), public_key01.clone());

        utxo.insert(outpoint00.clone(), tx_out00);
        utxo.insert(outpoint01.clone(), tx_out01);

        let initial_tx_outs = utxo.values().cloned().collect();

        let mut blockchain: Vec<Block> = Vec::new();
        let mut transactions1: Vec<Transaction> = Vec::new();
        let mut transactions2: Vec<Transaction> = Vec::new();

        // Create a single new transaction (2 inputs and 1 output)
        let mut tx_inputs1: Vec<TxIn> = Vec::new();
        let mut tx_outputs1: Vec<TxOut> = Vec::new();

        let message10 = String::from(&outpoint00.txid)
            + &outpoint00.index.to_string()
            + &utxo[&outpoint00].pk_script.public_key_hash;

        let sig_script10 = SignatureScript {
            signature: sign_and_verify::sign(&message10, &private_key00, &public_key00),
            full_public_key: public_key00,
        };

        tx_inputs1.push(TxIn {
            outpoint: outpoint00,
            sig_script: sig_script10,
        });

        let message11 = String::from(&outpoint01.txid)
            + &outpoint01.index.to_string()
            + &utxo[&outpoint01].pk_script.public_key_hash;

        let sig_script11 = SignatureScript {
            signature: sign_and_verify::sign(&message11, &private_key01, &public_key01),
            full_public_key: public_key01,
        };

        tx_inputs1.push(TxIn {
            outpoint: outpoint01,
            sig_script: sig_script11,
        });

        let (private_key10, public_key10) = sign_and_verify::create_keypair();

        let pk_script10 = PublicKeyScript {
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
            index: 0_u32,
        };

        utxo.verify_transaction(&transaction1);
        utxo.update(&transaction1);
        transactions1.push(transaction1);

        // Create genesis block and first block
        let genesis_merkle: Merkle = Merkle {
            tree: Vec::from(["0".repeat(64)]),
        };
        let genesis_block: Block = Block {
            header: BlockHeader {
                previous_hash: "0".repeat(64),
                merkle_root: genesis_merkle.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: genesis_merkle,
            transactions: Vec::new(),
        };

        blockchain.push(genesis_block);

        let (merkle1, _) = Merkle::create_merkle_tree(&transactions1, false, 0);
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

        // Create a second transaction and second block. Second transaction will have one input and two outputs.
        let mut tx_inputs2: Vec<TxIn> = Vec::new();
        let mut tx_outputs2: Vec<TxOut> = Vec::new();

        let message20 = String::from(&outpoint10.txid)
            + &outpoint10.index.to_string()
            + &utxo[&outpoint10].pk_script.public_key_hash;

        let sig_script20 = SignatureScript {
            signature: sign_and_verify::sign(&message20, &private_key10, &public_key10),
            full_public_key: public_key10,
        };

        tx_inputs2.push(TxIn {
            outpoint: outpoint10,
            sig_script: sig_script20,
        });

        let (private_key20, public_key20) = sign_and_verify::create_keypair();
        let pk_script20 = PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key20),
            verifier: Verifier {},
        };

        tx_outputs2.push(TxOut {
            value: 350,
            pk_script: pk_script20,
        });

        let (private_key21, public_key21) = sign_and_verify::create_keypair();
        let pk_script21 = PublicKeyScript {
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
            index: 0_u32,
        };

        let outpoint21 = Outpoint {
            txid: hash::hash_as_string(&transaction2),
            index: 1_u32,
        };

        utxo.verify_transaction(&transaction2);
        utxo.update(&transaction2);
        transactions2.push(transaction2);

        let (merkle2, _) = Merkle::create_merkle_tree(&transactions2, false, 0);
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
            block_duration: 10,
            block_mean: 1.0,
            block_size: 1,
            invalid_tx_mean_ratio: 50,
            max_tx_outputs: 4,
            tx_mean: 1.0,
            tx_duration: 10,
        };

        serialize_json(
            &blockchain,
            Some(String::from("test")),
            &initial_tx_outs,
            &keymap,
            &pr_keys,
            &pu_keys,
            &config,
            &utxo,
        );
    }

    #[ignore]
    #[test]
    fn test_deserialize_json() {
        let path = "./config/test_2022-11-15-11-50-06.json";

        let (blockchain, config, initial_tx_outs, keymap, pr_keys, pu_keys, utxo) =
            deserialize_json(path);

        serialize_json(
            &blockchain,
            Some(String::from("test")),
            &initial_tx_outs,
            &keymap,
            &pr_keys,
            &pu_keys,
            &config,
            &utxo,
        );
    }
}
