use crate::block::{Block, BlockHeader};
use crate::hash;
use crate::merkle::Merkle;
use crate::save_and_load;
use crate::save_and_load::Config;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;

use crate::sign_and_verify;
use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
use crate::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
use crate::utxo::UTXO;
use crate::validator;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::mpsc::Receiver;
use std::{collections::HashMap, sync::mpsc, thread};

static BLOCK_MEAN: f32 = 1.0;
static BLOCK_DURATION: u32 = 10;
pub static BLOCK_SIZE: u32 = 8;
static MAX_NUM_OUTPUTS: usize = 3;
static TRANSACTION_MEAN: f32 = 1.0;
static TRANSACTION_DURATION: u32 = 5;
static INVALID_TRANSACTION_FREQUENCY: u32 = 50;
static INVALID_BLOCK_FREQUENCY: u32 = 3;

pub fn start(rx_sim: Receiver<String>) {
    let mut blockchain: Vec<Block> = Vec::new();
    let mut utxo: UTXO = UTXO(HashMap::new());
    let mut key_map: KeyMap = KeyMap(HashMap::new());
    let sim_config: Config = Config {
        block_mean: BLOCK_MEAN,
        block_duration: BLOCK_DURATION,
        block_size: BLOCK_SIZE,
        max_tx_outputs: MAX_NUM_OUTPUTS,
        tx_mean: TRANSACTION_MEAN,
        tx_duration: TRANSACTION_DURATION,
        invalid_tx_mean: 1.0,
        invalid_tx_sigma: 1.0,
    };

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

    let pr_keys = (private_key0.clone(), private_key1.clone());
    let pu_keys = (public_key0.clone(), public_key1.clone());

    key_map.insert(outpoint0.clone(), (private_key0, public_key0));
    key_map.insert(outpoint1.clone(), (private_key1, public_key1));

    utxo.insert(outpoint0, tx_out0);
    utxo.insert(outpoint1, tx_out1);

    let initial_tx_outs = utxo.values().cloned().collect();

    // Create genesis block
    // Create the merkle tree for the genesis block
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
    // Create the blockchain and add the genesis block to the chain
    blockchain.push(genesis_block);
    let blockchain_copy = blockchain.clone();
    let blockchain_copy2 = blockchain.clone();

    let utxo_copy = utxo.clone();
    let utxo_copy2 = utxo.clone();

    // senderfile_receiverfile_object(s)sent_tx/rx
    let (simulation_transaction_string_tx, simulation_transaction_string_rx) = mpsc::channel();
    let (transaction_block_transaction_keymap_tx, transaction_block_transaction_keymap_rx) =
        mpsc::channel();
    let (block_sim_block_tx, block_sim_block_rx) = mpsc::channel();
    let (block_sim_utxo_tx, block_sim_utxo_rx) = mpsc::channel();
    let (block_sim_keymap_tx, block_sim_keymap_rx) = mpsc::channel();
    let (block_validator_block_tx, block_validator_block_rx) = mpsc::channel();

    thread::spawn(|| {
        Transaction::transaction_generator(
            transaction_block_transaction_keymap_tx,
            simulation_transaction_string_rx,
            MAX_NUM_OUTPUTS,
            TRANSACTION_MEAN,
            TRANSACTION_DURATION,
            INVALID_TRANSACTION_FREQUENCY,
            utxo,
            key_map,
        );
    });

    thread::spawn(|| {
        Block::block_generator(
            (
                block_sim_block_tx,
                block_sim_utxo_tx,
                block_sim_keymap_tx,
                block_validator_block_tx,
            ),
            (transaction_block_transaction_keymap_rx,),
            utxo_copy,
            blockchain_copy,
            BLOCK_MEAN,
            BLOCK_DURATION,
            INVALID_BLOCK_FREQUENCY,
        );
    });

    let _verifier_handle = thread::spawn(|| {
        validator::chain_validator(block_validator_block_rx, utxo_copy2, blockchain_copy2)
    });

    utxo = UTXO(HashMap::new());
    key_map = KeyMap(HashMap::new());
    loop {
        let new_block = block_sim_block_rx.try_recv();
        if let Ok(block1) = new_block {
            blockchain.push(block1);
        }

        let new_utxo = block_sim_utxo_rx.try_recv();
        if let Ok(utxo1) = new_utxo {
            utxo = utxo1;
        }

        let new_keymap = block_sim_keymap_rx.try_recv();
        if let Ok(key_map1) = new_keymap {
            key_map = key_map1;
        }

        let command = rx_sim.try_recv();
        if let Ok(command1) = command {
            if command1 == "save" {
                if let Err(e) =
                    simulation_transaction_string_tx.send(mpsc::TryRecvError::Disconnected)
                {
                    warn!(
                        "Sending error to transaction generator failed with message {}",
                        e
                    )
                };
                save_and_load::serialize_json(
                    &initial_tx_outs,
                    &blockchain,
                    &utxo,
                    &key_map,
                    &pr_keys,
                    &pu_keys,
                    &sim_config,
                    Some(String::from("state")),
                );
            }
        }
    }
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyMap(#[serde_as(as = "Vec<(_, _)>")] pub HashMap<Outpoint, (PrivateKey, PublicKey)>);

impl Deref for KeyMap {
    type Target = HashMap<Outpoint, (PrivateKey, PublicKey)>;
    fn deref(&self) -> &HashMap<Outpoint, (PrivateKey, PublicKey)> {
        return &self.0;
    }
}

impl DerefMut for KeyMap {
    fn deref_mut(&mut self) -> &mut HashMap<Outpoint, (PrivateKey, PublicKey)> {
        return &mut self.0;
    }
}
