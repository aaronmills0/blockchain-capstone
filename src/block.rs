use crate::merkle::Merkle;
use crate::transaction::Transaction;
use crate::transaction::TxIn;
use crate::transaction::TxOut;
use crate::utxo::UTXO;
use crate::{hash, simulation};

use log::info;
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Exp};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

#[derive(Serialize)]
pub struct Block {
    pub header: BlockHeader,
    pub merkle: Merkle,
    pub transactions: Vec<Transaction>,
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
            merkle: genesis_merkle,
            transactions: Vec::new(),
        };
        // Create the blockchain and add the genesis block to the chain
        let mut blockchain: Vec<Block> = Vec::new();
        blockchain.push(genesis_block);
        let mut block: Block;
        let mut counter: u32;
        let mut merkle: Merkle;
        let mut transactions: Vec<Transaction>;
        loop {
            transactions = Vec::new();
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
                merkle,
                transactions,
            };
            blockchain.push(block);
            Block::print_blockchain(&blockchain);
            transmitter.send(utxo.clone()).unwrap();
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

mod tests {
    use super::{hash, Block, HashMap, Transaction, UTXO};
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::transaction::{Outpoint, PublicKeyScript, SignatureScript, TxIn, TxOut};
    use rand::rngs::ThreadRng;
    static MAX_NUM_OUTPUTS: usize = 3;

    fn create_valid_transactions() -> (std::vec::Vec<Transaction>, UTXO) {
        //We first insert an unspent output in the utxo to which we will
        //refer later on.
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
        let (private_key0, public_key0) = sign_and_verify::create_keypair();
        let outpoint0: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };

        let tx_out0: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0),
                verifier: Verifier {},
            },
        };

        let (private_key0_1, public_key0_1) = sign_and_verify::create_keypair();
        let outpoint0_1: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 1,
        };

        let tx_out0_1: TxOut = TxOut {
            value: 100,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0_1),
                verifier: Verifier {},
            },
        };

        let (private_key0_2, public_key0_2) = sign_and_verify::create_keypair();
        let outpoint0_2: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 2,
        };

        let tx_out0_2: TxOut = TxOut {
            value: 200,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0_2),
                verifier: Verifier {},
            },
        };

        key_map.insert(outpoint0.clone(), (private_key0, public_key0));
        key_map.insert(outpoint0_1.clone(), (private_key0_1, public_key0_1));
        key_map.insert(outpoint0_2.clone(), (private_key0_2, public_key0_2));

        utxo.insert(outpoint0.clone(), tx_out0.clone());
        utxo.insert(outpoint0_1.clone(), tx_out0_1.clone());
        utxo.insert(outpoint0_2.clone(), tx_out0_2.clone());

        //We create a signature script for the inputs of our new transaction
        let mut sig_script1: SignatureScript;
        let mut sig_script1_1: SignatureScript;
        let mut sig_script1_2: SignatureScript;

        let mut old_private_key0: PrivateKey;
        let mut old_public_key0: PublicKey;

        let mut old_private_key0_1: PrivateKey;
        let mut old_public_key0_1: PublicKey;

        let mut old_private_key0_2: PrivateKey;
        let mut old_public_key0_2: PublicKey;

        (old_private_key0, old_public_key0) = key_map[&outpoint0].clone();
        (old_private_key0_1, old_public_key0_1) = key_map[&outpoint0_1].clone();
        (old_private_key0_2, old_public_key0_2) = key_map[&outpoint0_2].clone();

        let mut message: String;

        message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key0),
            full_public_key: old_public_key0,
        };

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

        message = String::from(&outpoint0_1.txid)
            + &outpoint0_1.index.to_string()
            + &tx_out0_1.pk_script.public_key_hash;

        sig_script1_1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key0_1),
            full_public_key: old_public_key0_1,
        };

        let tx_in1_1: TxIn = TxIn {
            outpoint: outpoint0_1,
            sig_script: sig_script1_1,
        };

        message = String::from(&outpoint0_2.txid)
            + &outpoint0_2.index.to_string()
            + &tx_out0_2.pk_script.public_key_hash;

        sig_script1_2 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key0_2),
            full_public_key: old_public_key0_2,
        };

        let tx_in1_2: TxIn = TxIn {
            outpoint: outpoint0_2,
            sig_script: sig_script1_2,
        };

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let mut transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1, tx_in1_1, tx_in1_2]),
            txin_count: 1,
            tx_outputs: Vec::from([tx_out1]),
            txout_count: 1,
        };

        let mut transactions: Vec<Transaction> = Vec::from([transaction1]);

        return (transactions, utxo);
    }

    fn create_invalid_transactions_insufficient_balance() -> (std::vec::Vec<Transaction>, UTXO) {
        //We first insert an unspent output in the utxo to which we will
        //refer later on.
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
        let (private_key0, public_key0) = sign_and_verify::create_keypair();
        let outpoint0: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };

        let tx_out0: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0),
                verifier: Verifier {},
            },
        };

        key_map.insert(outpoint0.clone(), (private_key0, public_key0));
        utxo.insert(outpoint0.clone(), tx_out0.clone());

        //We create a signature script for the input of our new transaction
        let mut sig_script1: SignatureScript;

        let mut old_private_key: PrivateKey;
        let mut old_public_key: PublicKey;

        (old_private_key, old_public_key) = key_map[&outpoint0].clone();

        let mut message: String;

        message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key),
            full_public_key: old_public_key,
        };

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 700,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let mut transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            txin_count: 1,
            tx_outputs: Vec::from([tx_out1]),
            txout_count: 1,
        };

        let mut transactions: Vec<Transaction> = Vec::from([transaction1]);

        return (transactions, utxo);
    }

    fn create_invalid_transactions_no_output_corresponding_to_input(
    ) -> (std::vec::Vec<Transaction>, UTXO) {
        //We do not include the unspent transaction in the utxo. That way, we cannot access the previous unspent output
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
        let (private_key0, public_key0) = sign_and_verify::create_keypair();
        let outpoint0: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };

        let tx_out0: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0),
                verifier: Verifier {},
            },
        };

        key_map.insert(outpoint0.clone(), (private_key0, public_key0));

        //We create a signature script for the input of our new transaction
        let mut sig_script1: SignatureScript;

        let mut old_private_key: PrivateKey;
        let mut old_public_key: PublicKey;

        (old_private_key, old_public_key) = key_map[&outpoint0].clone();

        let mut message: String;

        message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key),
            full_public_key: old_public_key,
        };

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let mut transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            txin_count: 1,
            tx_outputs: Vec::from([tx_out1]),
            txout_count: 1,
        };

        let mut transactions: Vec<Transaction> = Vec::from([transaction1]);

        return (transactions, utxo);
    }

    fn create_invalid_transactions_nomatch_signature() -> (std::vec::Vec<Transaction>, UTXO) {
        //We first insert an unspent output in the utxo to which we will
        //refer later on.
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
        let (private_key0, public_key0) = sign_and_verify::create_keypair();
        let outpoint0: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };

        let tx_out0: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0),
                verifier: Verifier {},
            },
        };

        key_map.insert(outpoint0.clone(), (private_key0, public_key0));
        utxo.insert(outpoint0.clone(), tx_out0.clone());

        //We create a signature script for the input of our new transaction
        let mut sig_script1: SignatureScript;

        let mut old_private_key: PrivateKey;
        let mut old_public_key: PublicKey;

        (old_private_key, old_public_key) = sign_and_verify::create_keypair();

        let mut message: String;

        message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key),
            full_public_key: old_public_key,
        };

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 700,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let mut transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            txin_count: 1,
            tx_outputs: Vec::from([tx_out1]),
            txout_count: 1,
        };

        let mut transactions: Vec<Transaction> = Vec::from([transaction1]);

        return (transactions, utxo);
    }

    #[test]
    fn test_verify_and_update_valid_transactions() {
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut transactions: Vec<Transaction>;

        (transactions, utxo) = create_valid_transactions();

        let old_outpoint = Outpoint {
            txid: hash::hash_as_string(transactions.get(0).unwrap()),
            index: (0),
        };

        (transactions, utxo) = Block::verify_and_update(transactions, utxo);

        assert_eq!(1, utxo.len());

        assert_eq!(1, transactions.len());

        assert_eq!(500, utxo.get(&old_outpoint).unwrap().value);
    }

    //The unspent output can spend 500. The transaction whose input in
    //links to this unspent output wants to spend 700 which is impossible.
    #[test]
    fn test_verify_and_update_invalid_transactions_insufficient_balance() {
        let mut utxo_original: UTXO = UTXO(HashMap::new());
        let mut utxo_new: UTXO = UTXO(HashMap::new());
        let mut transactions: Vec<Transaction>;

        (transactions, utxo_original) = create_invalid_transactions_insufficient_balance();

        let old_outpoint = transactions
            .get(0)
            .unwrap()
            .tx_inputs
            .get(0)
            .unwrap()
            .outpoint
            .clone();

        (transactions, utxo_new) = Block::verify_and_update(transactions, utxo_original.clone());

        assert_eq!(utxo_new.len(), utxo_original.len());
        assert_eq!(
            utxo_new.get(&old_outpoint).unwrap().value,
            utxo_original.get(&old_outpoint).unwrap().value
        );
        assert_eq!(0, transactions.len());
    }

    #[test]
    fn test_verify_and_update_invalid_input_nomatch_output() {
        let mut utxo_original: UTXO = UTXO(HashMap::new());
        let mut utxo_new: UTXO = UTXO(HashMap::new());
        let mut transactions: Vec<Transaction>;

        (transactions, utxo_original) =
            create_invalid_transactions_no_output_corresponding_to_input();

        let old_outpoint = transactions
            .get(0)
            .unwrap()
            .tx_inputs
            .get(0)
            .unwrap()
            .outpoint
            .clone();

        (transactions, utxo_new) = Block::verify_and_update(transactions, utxo_original.clone());

        assert_eq!(utxo_new.len(), utxo_original.len());
        assert_eq!(utxo_new.len(), 0);
        assert_eq!(0, transactions.len());
    }

    #[test]
    fn test_verify_and_update_invalid_nomatch_signature() {
        let mut utxo_original: UTXO = UTXO(HashMap::new());
        let mut utxo_new: UTXO = UTXO(HashMap::new());
        let mut transactions: Vec<Transaction>;

        (transactions, utxo_original) =
            create_invalid_transactions_no_output_corresponding_to_input();

        let old_outpoint = transactions
            .get(0)
            .unwrap()
            .tx_inputs
            .get(0)
            .unwrap()
            .outpoint
            .clone();

        (transactions, utxo_new) = Block::verify_and_update(transactions, utxo_original.clone());

        assert_eq!(utxo_new.len(), utxo_original.len());
        assert_eq!(utxo_new.len(), 0);
        assert_eq!(0, transactions.len());
    }
}
