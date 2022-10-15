mod block;
use block::Block;
use block::BlockHeader;
mod hash;
mod merkle;
use merkle::Merkle;
mod signer_and_verifier;
mod transaction;
use transaction::Transaction;

use std::collections::HashMap;
use std::io;
use std::process;
use std::vec::Vec;

const BLOCK_SIZE: u128 = 1;

fn main() {
    println!("Welcome to the simple transaction chain!\n");
    let (mut blockchain, mut transaction_list, mut utxo) = initialize();

    loop {
        display_utxo(&utxo);
        let (senders, receivers, units, contribution) = add_transaction();
        if !update_transaction(
            &senders,
            &receivers,
            &units,
            &contribution,
            &mut transaction_list,
            &mut utxo,
        ) {
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

fn display_utxo(utxo: &HashMap<String, u128>) {
    println!("Current UTXO:");
    for (key, value) in utxo {
        println!("{} : {}", key, value);
    }
    println!();
}

fn add_transaction() -> (Vec<String>, Vec<String>, Vec<u128>, Vec<u128>) {
    println!("New transaction:\n");

    let mut senders: Vec<String> = Vec::new();
    let mut receivers: Vec<String> = Vec::new();
    let mut units: Vec<u128> = Vec::new();
    let mut contribution: Vec<u128> = Vec::new();
    let mut user_input = String::new();

    loop {
        //Receive and Proccess user Input-Output-Unit Pairs
        println!("Please Enter the Sender-Contribution Pair as Follows, 'a 10':");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        
        let split = user_input.split_whitespace(); //Tokenize by whitespace
        let s2 = split.clone();
        if s2.count() != 2{
            continue;
        }

        let mut i = 0;
        for s in split{ //Note next_chunk() not yet functional so for loop is needed
            if i % 2 == 0{
                senders.push(s.trim().to_string());
            }
            else{
                let unit: u128 = match s.trim().parse() {
                    Ok(num) => num,
                    Err(_) => process::exit(1),
                };
                contribution.push(unit);
            }
            i += 1;
        }

        println!("Would you like to add another sender-contribution pair? [y/n]:");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        match user_input.trim() {
            "y" => {}
            _ => break,
        }
    }

    loop{
        println!("Please Enter a Receiver Unit Pair as Follows, 'a 10':");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        let split = user_input.split_whitespace(); //Tokenize by whitespace

        //Check 2 tokens were entered
        let s2 = split.clone();
        if s2.count() != 2{
            continue;
        }

        let mut i = 0;
        for s in split{ //Note next_chunk() not yet functional so for loop is needed
            if i % 2 == 0{
                receivers.push(s.trim().to_string());
            }
            else{
                let unit: u128 = match s.trim().parse() {
                    Ok(num) => num,
                    Err(_) => process::exit(1),
                };
                units.push(unit);
            }
            i += 1;
        }
        println!("Would you like to add another receiver-unit pair? [y/n]:");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        match user_input.trim() {
            "y" => {}
            _ => break,
        }
    }

    return (senders, receivers, units, contribution);
}

fn update_transaction(
    senders: &Vec<String>,
    receivers: &Vec<String>,
    units: &Vec<u128>,
    contribution: &Vec<u128>,
    transaction_list: &mut Vec<Transaction>,
    utxo: &mut HashMap<String, u128>,
) -> bool {

    //Check if senders are contributing enough
    let c_sum: u128 = contribution.iter().sum();
    let u_sum: u128 = units.iter().sum();

    if u_sum > c_sum {
        println!("Invalid Transaction!");
        return false;
    }

    for (i, s) in senders.iter().enumerate(){
        if !(utxo.contains_key(s)) || contribution[i] > *utxo.get(s).unwrap(){
            println!("Invalid transaction!\n");
            return false;
        }
    }

    let transaction: Transaction = Transaction {
        senders: senders.clone(),
        receivers: receivers.clone(),
        units: units.clone(),
    };
    transaction_list.push(transaction);

    //Generate transaction hash, sign transaction with private key, verify signed transaction with public key
    println!();
    let transaction_hash = hash::hash_as_string(transaction_list.last().unwrap());
    let (secret_key, public_key) = signer_and_verifier::create_keypair();
    let signed_transaction = signer_and_verifier::sign(&transaction_hash, &secret_key);
    println!("The signed transaction is {}:", signed_transaction);
    println!("The public key for this transaction is {}:", public_key);
    println!(
        "Does the signed transaction correspond to public key?: {}\n",
        signer_and_verifier::verify(&transaction_hash, &signed_transaction, &public_key)
    );

    for key in senders {
        utxo.remove(key);
    }

    for (i, receiver) in receivers.iter().enumerate() {
        if utxo.contains_key(receiver) {
            *utxo.get_mut(receiver).unwrap() += (*units)[i];
        } else {
            utxo.insert(receiver.to_string(), (*units)[i]);
        }
    }

    let fee: u128 = c_sum - u_sum;
    println!("Transaction fee: {}\n", &fee);
    return true;
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
