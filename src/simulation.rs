use crate::components::block::{Block, BlockHeader};
use crate::components::merkle::Merkle;
use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
use crate::components::utxo::UTXO;
use crate::utils::save_and_load::Config;
use crate::utils::sign_and_verify;
use crate::utils::sign_and_verify::{PrivateKey, PublicKey, Verifier};
use crate::utils::{hash, save_and_load, validator};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

static BLOCK_DURATION: u32 = 10;
static BLOCK_MEAN: f32 = 1.0;
pub static BLOCK_SIZE: u32 = 8;
static INVALID_BLOCK_FREQUENCY: u32 = 3;
static INVALID_TRANSACTION_FREQUENCY: u32 = 50;
static MAX_NUM_OUTPUTS: usize = 3;
static TRANSACTION_DURATION: u32 = 5;
static TRANSACTION_MEAN: f32 = 1.0;

pub fn start(rx_sim: Receiver<String>) {
    let mut blockchain: Vec<Block> = Vec::new();
    let mut utxo: UTXO = UTXO(HashMap::new());
    let mut keymap: KeyMap = KeyMap(HashMap::new());
    let sim_config: Config = Config {
        block_duration: BLOCK_DURATION,
        block_mean: BLOCK_MEAN,
        block_size: BLOCK_SIZE,
        invalid_tx_mean_ratio: INVALID_TRANSACTION_FREQUENCY,
        max_tx_outputs: MAX_NUM_OUTPUTS,
        tx_mean: TRANSACTION_MEAN,
        tx_duration: TRANSACTION_DURATION,
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

    keymap.insert(outpoint0.clone(), (private_key0, public_key0));
    keymap.insert(outpoint1.clone(), (private_key1, public_key1));

    utxo.insert(outpoint0, tx_out0);
    utxo.insert(outpoint1, tx_out1);

    let initial_tx_outs = utxo.values().cloned().collect();

    // Create the merkle tree and the genesis block
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

    // Notation: senderfile_receiverfile_object(s)sent_tx/rx
    let (transaction_block_transaction_keymap_tx, transaction_block_transaction_keymap_rx) =
        mpsc::channel();
    let (block_sim_block_tx, block_sim_block_rx) = mpsc::channel();
    let (block_sim_utxo_tx, block_sim_utxo_rx) = mpsc::channel();
    let (block_sim_keymap_tx, block_sim_keymap_rx) = mpsc::channel();
    let (block_validator_block_tx, block_validator_block_rx) = mpsc::channel();

    thread::spawn(|| {
        Transaction::transaction_generator(
            transaction_block_transaction_keymap_tx,
            MAX_NUM_OUTPUTS,
            TRANSACTION_MEAN,
            TRANSACTION_DURATION,
            INVALID_TRANSACTION_FREQUENCY,
            utxo,
            keymap,
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
            transaction_block_transaction_keymap_rx,
            utxo_copy,
            blockchain_copy,
            BLOCK_MEAN,
            BLOCK_DURATION,
            INVALID_BLOCK_FREQUENCY,
        );
    });

    thread::spawn(|| {
        validator::chain_validator(block_validator_block_rx, utxo_copy2, blockchain_copy2)
    });

    utxo = UTXO(HashMap::new());
    keymap = KeyMap(HashMap::new());
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
            keymap = key_map1;
        }

        let command = rx_sim.try_recv();
        if let Ok(command1) = command {
            if command1 == "save" {
                save_and_load::serialize_json(
                    &initial_tx_outs,
                    &blockchain,
                    &utxo,
                    &keymap,
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
