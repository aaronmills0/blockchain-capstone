use crate::block::{Block, BlockHeader};
use crate::hash;
use crate::merkle::Merkle;
use crate::sign_and_verify;
use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
use crate::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
use crate::utxo::UTXO;
use crate::validator;    

use std::{collections::HashMap, sync::mpsc, thread};

static BLOCK_MEAN: f32 = 1.0;
static BLOCK_MULTIPLIER: u32 = 10;
pub static BLOCK_SIZE: u32 = 8;
static MAX_NUM_OUTPUTS: usize = 3;
static TRANSACTION_MEAN: f32 = 1.0;
static TRANSACTION_MULTIPLIER: u32 = 5;

#[allow(dead_code)] // To prevent warning for unused functions
pub fn start() {
    let mut utxo: UTXO = UTXO(HashMap::new());

    let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
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

    // Create genesis block
    // Create the merkle tree for the genesis block
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
    // Create the blockchain and add the genesis block to the chain
    let mut blockchain: Vec<Block> = Vec::new();
    blockchain.push(genesis_block);
    let mut blockchain_copy = blockchain.clone();

    let (tx, rx) = mpsc::channel();
    let (ty, ry) = mpsc::channel();
    let (tv, rv) = mpsc::channel();
    let mut utxo_copy = utxo.clone();
    let mut utxo_copy2 = utxo.clone();
    let transaction_handle = thread::spawn(|| {
        Transaction::transaction_generator(
            MAX_NUM_OUTPUTS,
            TRANSACTION_MEAN,
            TRANSACTION_MULTIPLIER,
            tx,
            ry,
            utxo,
            key_map,
        );
    });

    let block_handle = thread::spawn(|| {
        Block::block_generator(rx, ty, tv, utxo_copy, blockchain, BLOCK_MEAN, BLOCK_MULTIPLIER);
    });

    let verifier_handle = thread::spawn(|| {
        validator::chain_validator(rv, utxo_copy2, blockchain_copy)
    });
}
