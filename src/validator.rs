use crate::block::Block;
use crate::hash;
use crate::merkle::Merkle;
use crate::simulation::BLOCK_SIZE;
use crate::utxo::UTXO;
use log::{info, warn};
use std::sync::mpsc::Receiver;
use std::vec::Vec;

pub fn chain_validator(receiver: Receiver<Block>, mut utxo: UTXO, mut chain: Vec<Block>) {
    let batch_size = (BLOCK_SIZE / 8) as usize;

    loop {
        let incoming_block = receiver.recv().unwrap();

        if fork_exists(&incoming_block, &chain) {
            continue;
        }

        let merkle_tree = Merkle::create_merkle_tree(&incoming_block.transactions);
        if !merkle_tree
            .tree
            .first()
            .unwrap()
            .eq(&incoming_block.header.merkle_root)
        {
            warn!("Validator received block with invalid transactions or invalid merkle root. Ignoring block.");
            continue;
        }

        let (valid, utxo_option) =
            utxo.parallel_batch_verify_and_update(&incoming_block.transactions, batch_size);
        if !valid {
            warn!("Validator received block containing invalid transactions. Ignoring block.");
            continue;
        }

        utxo = utxo_option.unwrap();
        chain.push(incoming_block);
    }
}

pub fn fork_exists(block: &Block, chain: &[Block]) -> bool {
    //Check prev block hash against newest validated block
    let prev_hash = &block.header.previous_hash;
    let head_hash = hash::hash_as_string(&chain.last().unwrap().header);
    if head_hash.eq(prev_hash) {
        info!(
            "Validator: Block {} has been introduced, no fork detected",
            hash::hash_as_string(&block.header)
        );
        return false;
    } else {
        for b in chain.iter().rev() {
            if b.header.previous_hash.eq(prev_hash) {
                warn!(
                    "Validator: Fork has been detected! Fork root at block header {}",
                    prev_hash
                );
                return true;
            }
        }
        warn!(
            "Validator: Received new block containing previous hash ({}) to unknown block",
            prev_hash
        );
        return true;
    }
}

#[cfg(test)]
mod tests {

    use rand_1::rngs::ThreadRng;
    use std::collections::HashMap;
    use std::time::Instant;

    use crate::block::{Block, BlockHeader};
    use crate::merkle::Merkle;
    use crate::sign_and_verify::Verifier;
    use crate::simulation::KeyMap;
    use crate::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
    use crate::utxo::UTXO;
    use crate::validator::fork_exists;
    use crate::{hash, sign_and_verify};

    #[test]
    fn force_fork() {
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

        let merkle1: Merkle = Merkle {
            tree: Vec::from(["0".repeat(64)]),
        };
        let block1: Block = Block {
            header: BlockHeader {
                previous_hash: hash::hash_as_string(&genesis_block.header),
                merkle_root: merkle1.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: merkle1,
            transactions: Vec::new(),
        };

        let merkle2: Merkle = Merkle {
            tree: Vec::from(["0".repeat(64)]),
        };
        let block2: Block = Block {
            header: BlockHeader {
                previous_hash: hash::hash_as_string(&genesis_block.header),
                merkle_root: merkle2.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: merkle2,
            transactions: Vec::new(),
        };
        let blockchain: Vec<Block> = vec![genesis_block];
        let mut blockchain_copy = blockchain.clone();
        let block1_copy = block1.clone();

        assert!(!fork_exists(&block1, &blockchain));
        blockchain_copy.push(block1_copy);

        assert!(fork_exists(&block2, &blockchain_copy));
    }

    #[test]
    fn test_block_validation_time() {
        let base: u32 = 10;
        let mut multiplicative_index: u32;
        for k in 0..7 {
            multiplicative_index = base.pow(k.try_into().unwrap());

            let mut utxo: UTXO = UTXO(HashMap::new());
            let mut key_map: KeyMap = KeyMap(HashMap::new());
            let mut transactions: Vec<Transaction> = Vec::new();
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
            utxo.insert(outpoint0, tx_out0);
            let utxo_copy = utxo.clone();

            let mut rng: ThreadRng = rand_1::thread_rng();
            let max_num_outputs = 1;
            for _ in 0..multiplicative_index {
                let transaction = Transaction::create_transaction(
                    &utxo,
                    &mut key_map,
                    &mut rng,
                    max_num_outputs,
                    false,
                );
                utxo.update(&transaction);

                transactions.push(transaction);
            }

            assert_eq!(transactions.len() as u32, multiplicative_index);

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

            let merkle = Merkle::create_merkle_tree(&transactions);

            let block: Block = Block {
                header: BlockHeader {
                    previous_hash: hash::hash_as_string(&genesis_block.header),
                    merkle_root: merkle.tree.first().unwrap().clone(),
                    nonce: 0,
                },
                merkle,
                transactions,
            };

            let chain: Vec<Block> = vec![genesis_block];

            println!("{} transactions: \n", multiplicative_index);
            for r in 0..5 {
                let block_copy = block.clone();
                let utxo_copy2 = utxo_copy.clone();
                let chain_copy = chain.clone();
                print!("{}: ", r);
                assert!(validate_one_block(block_copy, utxo_copy2, chain_copy));
                println!();
            }
        }
    }

    // This helper function validates one block and times the three checks involved as well as the total time
    fn validate_one_block(incoming_block: Block, mut utxo: UTXO, chain: Vec<Block>) -> bool {
        let full_time = Instant::now();

        let fork_time = Instant::now();
        if fork_exists(&incoming_block, &chain) {
            println!("Received a block that doesn't branch off the head of the chain");
            return false;
        }
        let fork_time_elapsed = fork_time.elapsed().as_micros();
        print!("{}, ", fork_time_elapsed);

        let merkle_time = Instant::now();
        let merkle_tree = Merkle::create_merkle_tree(&incoming_block.transactions);
        if merkle_tree.tree.first().unwrap() != &incoming_block.header.merkle_root {
            println!("Received a block with an invalid merkle root");
            return false;
        }
        let merkle_time_elapsed = merkle_time.elapsed().as_micros();
        print!("{}, ", merkle_time_elapsed);

        let tx_time = Instant::now();
        for tx in incoming_block.transactions.iter() {
            if !utxo.verify_transaction(tx) {
                println!("Received a block containing invalid transactions");
                return false;
            }
            utxo.update(tx);
        }
        let tx_time_elapsed = tx_time.elapsed().as_micros();
        print!("{}, ", tx_time_elapsed);

        let full_time_elapsed = full_time.elapsed().as_micros();
        print!("{}", full_time_elapsed);

        return true;
    }
}
