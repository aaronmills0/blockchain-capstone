use crate::hash;
use crate::merkle::Merkle;
use crate::transaction::Transaction;
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Exp};
use serde::Serialize;
use std::sync::mpsc::Receiver;
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
    pub nonce: u128,
}

impl Block {
    /**
     * Take the receiver object as input
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
    pub fn block_generator(receiver: Receiver<Vec<Transaction>>, mean: f64, multiplier: u64) {
        if mean <= 0.0 {
            panic!("Invalid input. A non-positive mean is invalid for an exponential distribution");
        }
        let lambda: f64 = 1.0 / mean;
        let exp: Exp<f64> = Exp::new(lambda).unwrap();
        let mut rng: ThreadRng = rand::thread_rng();
        let mut sample: f64;
        let mut normalized: f64;
        let mut mining_time: time::Duration;
        // Create genesis block
        // Create the merkle tree for the genesis block
        let genesis_merkle: Merkle = Merkle {
            tree: Vec::from([
                "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            ]),
        };
        let genesis_block: Block = Block {
            header: BlockHeader {
                previous_hash: "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                merkle_root: genesis_merkle.tree.first().unwrap().clone(),
                nonce: 0,
            },
            transactions: genesis_merkle,
        };
        // Create the blockchain and add the genesis block to the chain
        let mut blockchain: Vec<Block> = Vec::new();
        blockchain.push(genesis_block);
        let mut block: Block;
        let mut merkle: Merkle;
        loop {
            sample = exp.sample(&mut rng);
            // For an exponential distribution (with lambda > 0), the values range from (0, lambda].
            // Since mean = 1/lambda, multiply the sample by the mean to normalize.
            normalized = sample * mean;
            // Get the 'mining' time as a duration
            mining_time = time::Duration::from_secs(multiplier * normalized as u64);
            // Sleep to mimic the 'mining' time
            thread::sleep(mining_time);
            // Create a new block
            merkle = Merkle::create_merkle_tree(&receiver.recv().unwrap());
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
        }
    }

    pub fn print_blockchain(blockchain: &Vec<Block>) {
        for block in blockchain {
            if hash::hash_as_string(&block.header.merkle_root)
                == hash::hash_as_string(
                    &"0000000000000000000000000000000000000000000000000000000000000000",
                )
            {
                print!("Block {}", hash::hash_as_string(&block.header));
                continue;
            }
            print!(" <= Block {}", hash::hash_as_string(&block.header));
            println!();
        }
    }
}
