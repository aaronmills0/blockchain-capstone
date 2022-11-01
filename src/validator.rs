use crate::hash;
use crate::sign_and_verify;
use crate::sign_and_verify::{PrivateKey, PublicKey, Signature, Verifier};
use crate::utxo::UTXO;

use log::{info, warn};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Distribution, Exp};
use serde::Serialize;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{Receiver, Sender};
use std::vec::Vec;
use std::{thread, time};

pub fn chain_validator(
    receiver: Receiver<Block>,
    mut utxo: UTXO,
    mut chain: Vec<Block>
){
    loop{
        incoming_block = receiver.recv().unwrap();
        for tx in incoming_block.transactions.iter(){
            if !utxo.verify_transaction(tx){
                warn!("Validator received block containing invalid transactions");
            }
        }
        if !check_for_fork(incoming_block, chain){
            chain.push(incoming_block);
            for tx in incoming_block.transactions.iter(){
                utxo.udpate(tx);
            }
        }
    }
}

pub fn check_for_fork(block: Block, chain: Vec<Block>) -> bool{
    //Check prev block hash against newest validated block
    let prev_hash = block.header.previous_hash;
    let cur = chain.len() - 1;
    if hash::hash_as_string(chain[cur].unwrap().header).eq(prev_hash){
        info!("Block {} has been introduced, no fork detected", hash::hash_as_string(block.unwrap().header));
        return false;
    }
    else {
        loop {
            if cur < 0 {
                warn!("Invalid block detected! Prev_hash ({}) points to non-existing block", prev_hash);
                return true;
            }
            else if chain[cur].header.prev_hash.eq(prev_hash) {
                warn!("Fork has been detected! Fork root at block header {}", prev_hash);
                return true;
            }
            cur -= 1;
        }
    }
}

mod tests{
    use crate::{validator::check_for_fork};

    #[test]
    fn force_fork(){
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
                previous_hash: "0".repeat(64).to_string(),
                merkle_root: genesis_merkle.tree.first().unwrap().clone(),
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
                previous_hash: "0".repeat(64).to_string(),
                merkle_root: genesis_merkle.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: merkle2,
            transactions: Vec::new(),
        };
        let mut blockchain: Vec<Block> = Vec::new();
        blockchain.push(genesis_block);

        assert!(check_for_fork(block1, blockchain) == false);
        blockchain.push(block1);
        assert!(check_for_fork(block2) == true);
    }
}
