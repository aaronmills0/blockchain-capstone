use crate::merkle::Merkle;
use crate::transaction::Transaction;
use crate::utxo::UTXO;
use crate::{hash, simulation};

use log::{info, warn};
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Exp};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

#[derive(Serialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Merkle,
}

#[derive(Serialize)]
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
        receiver: Receiver<Transaction>,
        transmitter: Sender<UTXO>,
        mut utxo: UTXO,
        mean: f32,
        multiplier: u32,
    ) {
        if mean <= 0.0 {
            panic!("Invalid input. A non-positive mean is invalid for an exponential distribution");
        }
        let lambda: f32 = 1.0 / mean;
        let exp: Exp<f32> = Exp::new(lambda).unwrap();
        let mut rng: ThreadRng = rand::thread_rng();
        let mut sample: f32;
        let mut normalized: f32;
        let mut mining_time: time::Duration;
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
            transactions: genesis_merkle,
        };
        // Create the blockchain and add the genesis block to the chain
        let mut blockchain: Vec<Block> = Vec::new();
        blockchain.push(genesis_block);
        let mut block: Block;
        let mut counter: u32;
        let mut merkle: Merkle;
        let mut transactions: Vec<Transaction> = Vec::new();
        loop {
            counter = 0;
            while counter < simulation::BLOCK_SIZE {
                transactions.push(receiver.recv().unwrap());
                counter += 1;
            }

            sample = exp.sample(&mut rng);
            // For an exponential distribution (with lambda > 0), the values range from (0, lambda].
            // Since mean = 1/lambda, multiply the sample by the mean to normalize.
            normalized = sample * mean;
            // Get the 'mining' time as a duration
            mining_time = time::Duration::from_secs((multiplier * normalized as u32) as u64);
            // Sleep to mimic the 'mining' time
            thread::sleep(mining_time);
            // Create a new block
            (transactions, utxo) = Block::verify_and_update(transactions, utxo);
            if transactions.len() == 0 {
                continue;
            }
            merkle = Merkle::create_merkle_tree(&transactions);
            block = Block {
                header: BlockHeader {
                    previous_hash: hash::hash_as_string(&blockchain.last().unwrap().header),
                    merkle_root: merkle.tree.first().unwrap().clone(),
                    nonce: 0,
                },
                transactions: merkle,
            };
            blockchain.push(block);
            Block::print_blockchain(&blockchain);
            transmitter.send(utxo.clone()).unwrap();

            transactions.clear();
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
        let mut utxo_copy = utxo.clone();
        let mut transactions_valid: Vec<Transaction> = Vec::new();
        for transaction in transactions {
            if !utxo_copy.verify_transaction(&transaction) {
                continue;
            }
            utxo_copy.update(&transaction);

            transactions_valid.push(transaction);
        }
        return (transactions_valid, utxo_copy);
    }

    pub fn print_blockchain(blockchain: &Vec<Block>) {
        for block in blockchain {
            if hash::hash_as_string(&block.header.merkle_root)
                == hash::hash_as_string(&("0".repeat(64)))
            {
                info!("\nBlock {}", hash::hash_as_string(&block.header));
                continue;
            }
            info!(" <= Block {}", hash::hash_as_string(&block.header));
        }
    }
}
