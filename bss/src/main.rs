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

    println!("For list of supported commands, enter help");
    loop {

        //Handles user commands
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

fn display_utxo(utxo: &HashMap<String, u128>) {
    println!("Current UTXO:");
    for (key, value) in utxo {
        println!("{} : {}", key, value);
    }
    println!();
}
fn display_transactions(transaction_list: &mut Vec<Transaction>){
    println!("Current transaction chain:");
    for transaction in transaction_list {
        for (i, val) in transaction.senders.iter().enumerate() {
            print!("[");
            print!("[ {} -> {}, {}]", val, transaction.receivers[i], transaction.units[i]);
            print!("]");
        }
    }
    println!();
}

fn display_blocks(blockchain: &mut Vec<Block>){
    println!("Current Blockchain!");
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
    println!();
}

fn display_commands(){
    println!("-> status: Displays the current state of the UTXO, Pending Transactions,
    and the Blocks");
    println!("-> add transaction: Allows the user to add a specific transaction manually")
}

fn interpreter(
    utxo: &mut HashMap<String, u128>,
    transaction_list: &mut Vec<Transaction>,
    blockchain: &mut Vec<Block>,
    ) -> bool{

    let mut command = String::new();
    

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");
    
    
    if command.trim() == "add transaction"{
        let (senders, receivers, units) = add_transaction();
        if !update_transaction(
            &senders,
            &receivers,
            &units,
            transaction_list,
            utxo,
        ) {
            return false;
        }
        return true;
    }
    else if command.trim() == "status" {
        display_utxo(utxo);
        display_transactions(transaction_list);
        display_blocks(blockchain);
        return true;

    }
    else if command.trim() == "help" {
        display_commands();
        return true;
    }
    else if command.trim() == "start sim"{
        return true;
    }
    else {
        return false;
    }
}

fn add_transaction() -> (Vec<String>, Vec<String>, Vec<u128>) {
    println!("New transaction:\n");

    let mut senders: Vec<String> = Vec::new();
    let mut receivers: Vec<String> = Vec::new();
    let mut units: Vec<u128> = Vec::new();
    let mut user_input = String::new();
    loop {
        //Receive and Proccess user Input-Output-Unit Pairs
        println!("Sender address:");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        senders.push(user_input.trim().to_string());

        println!("Receiver address:");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        receivers.push(user_input.trim().to_string());

        println!("Transfer quantity:");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        let unit: u128 = match user_input.trim().parse() {
            Ok(num) => num,
            Err(_) => process::exit(1),
        };
        units.push(unit);
        println!();

        println!("Would you like to add another input -> output? [y/n]:");
        user_input.clear();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");

        match user_input.trim() {
            "y" => {}
            _ => break,
        }
    }

    return (senders, receivers, units);
}

fn update_transaction(
    senders: &Vec<String>,
    receivers: &Vec<String>,
    units: &Vec<u128>,
    transaction_list: &mut Vec<Transaction>,
    utxo: &mut HashMap<String, u128>,
) -> bool {
    let mut input_sum: HashMap<String, u128> = HashMap::new();
    for (i, val) in senders.iter().enumerate() {
        if input_sum.contains_key(val) {
            *input_sum.get_mut(val).unwrap() += (*units)[i];
        } else {
            input_sum.insert(val.to_string(), (*units)[i]);
        }
    }

    let mut trans_sum = 0;
    let mut utxo_sum = 0;
    for (sender, value) in &input_sum {
        if !(utxo.contains_key(sender)) || value > utxo.get(sender).unwrap() {
            println!("Invalid transaction!\n");
            return false;
        } else {
            trans_sum += value;
            utxo_sum += utxo.get(sender).unwrap();
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

    for (key, _) in &input_sum {
        utxo.remove(key);
    }

    for (i, receiver) in receivers.iter().enumerate() {
        if utxo.contains_key(receiver) {
            *utxo.get_mut(receiver).unwrap() += (*units)[i];
        } else {
            utxo.insert(receiver.to_string(), (*units)[i]);
        }
    }

    let fee: u128 = utxo_sum - trans_sum;
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
