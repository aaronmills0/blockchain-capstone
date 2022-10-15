use crate::merkle::Merkle;
use crate::transaction::Transaction;
use crate::utxo::UTXO;
use crate::{hash, simulation};

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
    pub nonce: u128,
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
        mean: f64,
        multiplier: u64,
    ) {
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
        let mut counter: u128;
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
            mining_time = time::Duration::from_secs(multiplier * normalized as u64);
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
                println!("Block {}", hash::hash_as_string(&block.header));
                continue;
            }
            println!(" <= Block {}", hash::hash_as_string(&block.header));
        }
    }
}

mod tests {
    use super::{Block, HashMap, Transaction, UTXO};

    #[test]
    fn test_verify_and_update_valid_transactions() {
        let tx0: Transaction = Transaction {
            senders: Vec::from([String::from("a")]),
            receivers: Vec::from([String::from("x"), String::from("y")]),
            units: Vec::from([20, 30]),
        };
        let tx1: Transaction = Transaction {
            senders: Vec::from([String::from("x"), String::from("y")]),
            receivers: Vec::from([String::from("b")]),
            units: Vec::from([50]),
        };
        let mut transactions: Vec<Transaction> = Vec::from([tx0, tx1]);
        let mut utxo: UTXO = UTXO {
            map: HashMap::new(),
        };
        utxo.map.insert(String::from("a"), 50);
        (transactions, utxo) = Block::verify_and_update(transactions, utxo);

        assert_eq!(2, transactions.len());
        assert_eq!(1, utxo.map.len());
        assert_eq!(50, *utxo.map.get(&String::from("b")).unwrap());
    }

    #[test]
    fn test_verify_and_update_invalid_transactions_insufficient_balance() {
        let tx0: Transaction = Transaction {
            senders: Vec::from([String::from("a")]),
            receivers: Vec::from([String::from("x"), String::from("y")]),
            units: Vec::from([20, 30]),
        };
        let tx1: Transaction = Transaction {
            senders: Vec::from([String::from("x"), String::from("y")]),
            receivers: Vec::from([String::from("b")]),
            units: Vec::from([50]),
        };
        let tx2: Transaction = Transaction {
            senders: Vec::from([String::from("b")]),
            receivers: Vec::from([String::from("p"), String::from("q"), String::from("r")]),
            units: Vec::from([20, 20, 20]),
        };
        let mut transactions: Vec<Transaction> = Vec::from([tx0, tx1, tx2]);
        let mut utxo: UTXO = UTXO {
            map: HashMap::new(),
        };
        utxo.map.insert(String::from("a"), 50);
        (transactions, utxo) = Block::verify_and_update(transactions, utxo);

        assert_eq!(2, transactions.len());
        assert_eq!(1, utxo.map.len());
        assert_eq!(50, *utxo.map.get(&String::from("b")).unwrap());
    }

    #[test]
    fn test_verify_and_update_invalid_transactions_unknown_sender() {
        let tx0: Transaction = Transaction {
            senders: Vec::from([String::from("a")]),
            receivers: Vec::from([String::from("x"), String::from("y")]),
            units: Vec::from([20, 30]),
        };
        let tx1: Transaction = Transaction {
            senders: Vec::from([String::from("x"), String::from("y")]),
            receivers: Vec::from([String::from("b"), String::from("c"), String::from("d")]),
            units: Vec::from([5, 15, 20]),
        };
        let tx2: Transaction = Transaction {
            senders: Vec::from([
                String::from("b"),
                String::from("c"),
                String::from("d"),
                String::from("e"),
            ]),
            receivers: Vec::from([String::from("p"), String::from("q"), String::from("r")]),
            units: Vec::from([20, 20, 10]),
        };
        let mut transactions: Vec<Transaction> = Vec::from([tx0, tx1, tx2]);
        let mut utxo: UTXO = UTXO {
            map: HashMap::new(),
        };
        utxo.map.insert(String::from("a"), 50);
        (transactions, utxo) = Block::verify_and_update(transactions, utxo);

        assert_eq!(2, transactions.len());
        assert_eq!(3, utxo.map.len());
        assert_eq!(5, *utxo.map.get(&String::from("b")).unwrap());
        assert_eq!(15, *utxo.map.get(&String::from("c")).unwrap());
        assert_eq!(20, *utxo.map.get(&String::from("d")).unwrap());
    }
}
