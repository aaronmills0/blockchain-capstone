use crate::block::Block;
use crate::hash;
use crate::utxo::UTXO;
use log::{info, warn};
use std::sync::mpsc::Receiver;
use std::vec::Vec;

pub fn chain_validator(receiver: Receiver<Block>, mut utxo: UTXO, mut chain: Vec<Block>) {
    loop {
        let incoming_block = receiver.recv().unwrap();

        if fork_exists(&incoming_block, &chain) {
            continue;
        }

        let (valid, utxo_option) = utxo.batch_verify_and_update(&incoming_block.transactions);
        if !valid {
            warn!("Validator received block containing invalid transactions. Ignoring block");
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
            "Block {} has been introduced, no fork detected",
            hash::hash_as_string(&block.header)
        );
        return false;
    } else {
        for b in chain.iter().rev() {
            if b.header.previous_hash.eq(prev_hash) {
                warn!(
                    "Fork has been detected! Fork root at block header {}",
                    prev_hash
                );
                return true;
            }
        }
        warn!(
            "Received new block containing previous hash ({}) to unknown block",
            prev_hash
        );
        return true;
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{Block, BlockHeader};
    use crate::hash;
    use crate::merkle::Merkle;
    use crate::validator::fork_exists;

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
}
