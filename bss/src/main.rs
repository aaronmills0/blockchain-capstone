mod block;
use block::Block;
use block::BlockHeader;
mod hash;
mod merkle;
use merkle::Merkle;
mod signer_and_verifier;
mod simulation;
mod transaction;
use transaction::Transaction;
mod utxo;
mod shell;
use shell::interpreter;

use std::collections::HashMap;
use std::vec::Vec;

const BLOCK_SIZE: u128 = 1;

fn main() {
    println!("Welcome to the simple transaction chain!\n");
    let (mut blockchain, mut transaction_list, mut utxo) = initialize();

    println!("For list of supported commands, enter help");
    loop {

        if !interpreter(&mut utxo, &mut transaction_list, &mut blockchain){
            continue;
        }

        if transaction_list.len() == BLOCK_SIZE.try_into().unwrap() {
            create_block(&mut blockchain, &mut transaction_list);
        }
    }
}

fn initialize() -> (Vec<Block>, Vec<Transaction>, HashMap<String, u128>) {
    let mut blockchain: Vec<Block> = Vec::new();
    let transaction_list: Vec<Transaction> = Vec::new();
    let mut utxo: HashMap<String, u128> = HashMap::new();

    //Create/add genesis block
    let genesis_merkle: Merkle = Merkle {
        tree: Vec::from(["0".repeat(64)]),
    };
    let genesis_block: Block = Block {
        header: BlockHeader {
            previous_hash: "0".repeat(64),
            merkle_root: genesis_merkle.tree.first().unwrap().clone(),
            nonce: 0,
        },
        transactions: genesis_merkle,
    };
    blockchain.push(genesis_block);

    utxo.insert(String::from("a"), 50);
    utxo.insert(String::from("b"), 20);

    return (blockchain, transaction_list, utxo);
}

fn create_block(blockchain: &mut Vec<Block>, transaction_list: &mut Vec<Transaction>) {
    let merkle: Merkle = Merkle::create_merkle_tree(&transaction_list);
    let new_block: Block = Block {
        header: BlockHeader {
            previous_hash: hash::hash_as_string(blockchain.last().unwrap()),
            merkle_root: merkle.tree.first().unwrap().clone(),
            nonce: 0,
        },
        transactions: merkle,
    };
    blockchain.push(new_block);

    println!("A block was added to the chain!");
    for block in blockchain {
        if hash::hash_as_string(&block.header.merkle_root)
            == hash::hash_as_string(
                &"0000000000000000000000000000000000000000000000000000000000000000",
            )
        {
            print!("Block {}", hash::hash_as_string(block));
            continue;
        }
        print!(" <= Block {}", hash::hash_as_string(block));
    }
    transaction_list.clear();
    println!();
}
