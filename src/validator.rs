use crate::block::Block;
use crate::hash;
use crate::utxo::UTXO;
use log::{info, warn};
use std::sync::mpsc::Receiver;
use std::vec::Vec;

pub fn chain_validator(receiver: Receiver<Block>, mut utxo: UTXO, mut chain: Vec<Block>) {
    'main: loop {
        let incoming_block = receiver.recv().unwrap();

        if check_for_fork(&incoming_block, &chain){
            continue;
        }

        for tx in incoming_block.transactions.iter() {
            if !utxo.verify_transaction(tx) {
                warn!("Validator received block containing invalid transactions. Ignoring block");
                continue 'main;
            }
            utxo.update(tx);
        }

        chain.push(incoming_block);
    }
}

pub fn check_for_fork(block: &Block, chain: &Vec<Block>) -> bool {
    //Check prev block hash against newest validated block
    let prev_hash = &block.header.previous_hash;
    let head_hash = hash::hash_as_string(&chain.last().unwrap().header);
    if head_hash.eq(prev_hash) {
        println!("here");
        info!("Block {} has been introduced, no fork detected", prev_hash);
        return false;
    } else {
        for b in chain.iter().rev(){
            if b.header.previous_hash.eq(prev_hash) {
                warn!(
                    "Fork has been detected! Fork root at block header {}",
                    prev_hash
                );
                return true;
            }
        }
        warn!("Received new block containing previous hash ({}) to unknown block", prev_hash);
        return true;
    }
}

mod tests {
    use crate::block::{Block, BlockHeader};
    use crate::hash;
    use crate::merkle::Merkle;
    use crate::validator::check_for_fork;

    #[test]
    fn force_fork() {
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

        let merkle1: Merkle = Merkle {
            tree: Vec::from(["0".repeat(64).to_string()]),
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
            tree: Vec::from(["0".repeat(64).to_string()]),
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
        let mut blockchain: Vec<Block> = Vec::new();
        blockchain.push(genesis_block);
        let mut blockchain_copy = blockchain.clone();
        let block1_copy = block1.clone();

        assert!(check_for_fork(&block1, &blockchain) == false);
        blockchain_copy.push(block1_copy);

        assert!(check_for_fork(&block2, &blockchain_copy) == true);
    }
}
