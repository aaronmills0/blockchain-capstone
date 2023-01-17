use crate::components::merkle::Merkle;
use crate::components::transaction::Transaction;
use crate::components::utxo::UTXO;
use crate::simulation;
use crate::simulation::KeyMap;
use crate::utils::hash;
use log::{info, warn};
use rand_1::rngs::ThreadRng;
use rand_1::Rng;
use rand_distr::{Distribution, Exp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub merkle: Merkle,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub previous_hash: String,
    pub merkle_root: String,
    pub nonce: u32,
}

impl Block {
    /**
     * Take the receiver object and initial utxo as input
     * Sample from an exponential distribution with a provided mean (in seconds).
     * Whatever we extract from the sample, we multiply by the multiplier to get the number of second until a block is created by the generator.
     *
     * Example: Sending a message to the block generator thread
     * let (tx, rx) = mpsc::channel();
     * let handle = thread::spawn(|| {
     *      Block::block_generator(rx, 1.0, 10);
     * });
     * tx.send(transactions);  
     */
    pub fn block_generator(
        // block_sim_block_tx, block_sim_utxo_tx, block_sim_keymap_tx, block_validator_block_tx
        block_tx: (Sender<Block>, Sender<UTXO>, Sender<KeyMap>, Sender<Block>),
        block_rx: Receiver<(Transaction, KeyMap)>, // transaction_block_transaction_keymap_rx
        mut utxo: UTXO,
        mut blockchain: Vec<Block>,
        mean: f32,
        duration: u32,
        invalid_block_frequency: u32,
    ) {
        if mean <= 0.0 {
            panic!("Invalid input. A non-positive mean is invalid for an exponential distribution");
        }

        let (block_sim_block_tx, block_sim_utxo_tx, block_sim_keymap_tx, block_validator_block_tx) =
            block_tx;
        let transaction_block_transaction_keymap_rx = block_rx;

        let lambda: f32 = 1.0 / mean;
        let exp: Exp<f32> = Exp::new(lambda).unwrap();

        let mut block: Block;
        let mut counter: u32;
        let mut keymap: KeyMap;
        let mut keymap_map: HashMap<String, KeyMap>;
        let mut merkle: Merkle;
        let mut mining_time: time::Duration;
        let mut normalized: f32;
        let mut rng: ThreadRng = rand_1::thread_rng();
        let mut sample: f32;
        let mut transactions: Vec<Transaction>;
        let mut tx: Transaction;
        loop {
            transactions = Vec::new();
            keymap_map = HashMap::new();
            keymap = KeyMap(HashMap::new());
            counter = 0;
            while counter < simulation::BLOCK_SIZE {
                (tx, keymap) = transaction_block_transaction_keymap_rx.recv().unwrap();
                keymap_map.insert(hash::hash_as_string(&tx), keymap.clone());
                transactions.push(tx);
                counter += 1;
            }

            sample = exp.sample(&mut rng);
            // For an exponential distribution (with lambda > 0), the values range from (0, lambda].
            // Since mean = 1/lambda, multiply the sample by the mean to normalize.
            normalized = sample * mean;
            // Get the 'mining' time as a duration
            mining_time = time::Duration::from_secs((duration * normalized as u32) as u64);
            thread::sleep(mining_time); // Sleep to mimic the 'mining' time

            (transactions, utxo) = Block::verify_and_update(transactions, utxo);
            if transactions.is_empty() {
                continue;
            }

            let mut found = false;
            for transaction in transactions.iter().rev() {
                let hash = hash::hash_as_string(transaction);
                if keymap_map.contains_key(&hash) {
                    keymap = keymap_map.remove(&hash).unwrap();
                    found = true;
                    break;
                }
            }
            if !found {
                panic!("KeyMap not found!");
            }

            if rng.gen_range(1..=invalid_block_frequency) == 1 {
                let invalid_type = rng.gen_range(1..=3);
                let merkle_copy: Merkle = Merkle::create_merkle_tree(&transactions);
                let mut merkle_root_copy = merkle_copy.tree.first().unwrap().clone();
                let mut previous_hash_copy =
                    hash::hash_as_string(&blockchain.last().unwrap().header);
                let mut transactions_copy = transactions.clone();

                if invalid_type == 1 {
                    warn!(
                        "Sending invalid block! Expect a block containing an invalid transaction."
                    );
                    let random_index = rng.gen_range(0..transactions_copy.len());
                    transactions_copy[random_index].tx_inputs[0].outpoint.txid =
                        hash::hash_as_string(
                            &transactions_copy[random_index].tx_inputs[0].outpoint.txid,
                        );
                } else if invalid_type == 2 {
                    warn!("Sending invalid block! Expect a block with an incorrect previous hash.");
                    previous_hash_copy = hash::hash_as_string(&previous_hash_copy);
                } else {
                    warn!("Sending invalid block! Expect a block with an incorrect merkle root.");
                    merkle_root_copy = hash::hash_as_string(&merkle_root_copy);
                }

                let invalid_block = Block {
                    header: BlockHeader {
                        previous_hash: previous_hash_copy,
                        merkle_root: merkle_root_copy,
                        nonce: 0,
                    },
                    merkle: merkle_copy,
                    transactions: transactions_copy,
                };
                block_validator_block_tx.send(invalid_block).unwrap();
            }

            info!("Creating block with {} transactions", transactions.len());
            merkle = Merkle::create_merkle_tree(&transactions);
            block = Block {
                header: BlockHeader {
                    previous_hash: hash::hash_as_string(&blockchain.last().unwrap().header),
                    merkle_root: merkle.tree.first().unwrap().clone(),
                    nonce: 0,
                },
                merkle,
                transactions,
            };

            let block_copy = block.clone();

            // Randomly Injects Fork
            if rng.gen_range(1..=10) == 1 {
                warn!("Sending a duplicate block! Expecting a fork to be detected.");
                let block_copy2 = block_copy.clone();
                block_validator_block_tx.send(block_copy2).unwrap();
            }

            block_sim_block_tx.send(block.clone()).unwrap();
            block_sim_utxo_tx.send(utxo.clone()).unwrap();
            block_sim_keymap_tx.send(keymap.clone()).unwrap();
            block_validator_block_tx.send(block_copy).unwrap();
            blockchain.push(block);

            Block::print_blockchain(&blockchain);
        }
    }

    /**
     * Given a vector of transactions, and the current utxo, verify the transactions and update the utxo
     * If a transaction is invalid, it is excluded from the returned transaction list, and the utxo update ignores its content
     */
    pub fn verify_and_update(
        transactions: Vec<Transaction>,
        utxo: UTXO,
    ) -> (Vec<Transaction>, UTXO) {
        let mut utxo1 = utxo;
        let mut transactions_valid: Vec<Transaction> = Vec::new();
        for transaction in transactions {
            if !utxo1.verify_transaction(&transaction) {
                continue;
            }
            utxo1.update(&transaction);
            transactions_valid.push(transaction);
        }
        return (transactions_valid, utxo1);
    }

    pub fn print_blockchain(blockchain: &Vec<Block>) {
        for block in blockchain {
            if hash::hash_as_string(&block.header.merkle_root)
                == hash::hash_as_string(&("0".repeat(64)))
            {
                info!("Block {}", hash::hash_as_string(&block.header));
                continue;
            }
            info!(" <= Block {}", hash::hash_as_string(&block.header));
        }
    }
}
