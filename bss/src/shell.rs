use crate::transaction::Transaction;
use crate::block::Block;
use crate::signer_and_verifier;
use crate::hash;

use std::collections::HashMap;
use std::io;
use std::process;
use std::vec::Vec;

pub fn interpreter(
    utxo: &mut HashMap<String, u128>,
    transaction_list: &mut Vec<Transaction>,
    blockchain: &mut Vec<Block>,
    ) -> bool{

    let mut command = String::new();
    

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");
    
    
    if command.trim() == "add transaction"{
        let (senders, receivers, units, transaction_signatures) = add_transaction();
        if !update_transaction(
            &senders,
            &receivers,
            &units,
            &transaction_signatures,
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

fn add_transaction() -> (Vec<String>, Vec<String>, Vec<u128>, String) {
    println!("New transaction:\n");

    let mut senders: Vec<String> = Vec::new();
    let mut receivers: Vec<String> = Vec::new();
    let mut units: Vec<u128> = Vec::new();
    let mut user_input = String::new();

    //Receive and Proccess user Input-Output-Unit Pairs
    println!("Please Enter the Senders as Follows, 'a b c ...':");
    user_input.clear();
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");
    
    let split = user_input.split_whitespace(); //Tokenize by whitespace
    for s in split{
        if !senders.contains(&s.trim().to_string()){
            senders.push(s.trim().to_string());
        }
    }

    let mut transaction_senders = String::new().to_owned();
    for s in &senders {
        transaction_senders.push_str(&s);
    }
    println!(
        "The concatenation of all senders for this owner is {}",
        transaction_senders
    );
    let transaction_hash: String = hash::hash_as_string(&transaction_senders);
    let (secret_key, public_key) = signer_and_verifier::create_keypair();
    let signature_of_sender = signer_and_verifier::sign(&transaction_hash, &secret_key);
    let transaction_signatures = signature_of_sender.to_string();
    println!(
        "Signature of transaction is {}",
        signature_of_sender.to_string()
    );

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

        for (i, s) in split.enumerate(){ //Note next_chunk() not yet functional so for loop is needed
            if i % 2 == 0{
                if !receivers.contains(&s.trim().to_string()){
                    receivers.push(s.trim().to_string());
                }
            }
            else{
                let unit: u128 = match s.trim().parse() {
                    Ok(num) => num,
                    Err(_) => process::exit(1),
                };
                units.push(unit);
            }
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

    return (senders, receivers, units, transaction_signatures);
}

fn update_transaction(
    senders: &Vec<String>,
    receivers: &Vec<String>,
    units: &Vec<u128>,
    transaction_signature: &String,
    transaction_list: &mut Vec<Transaction>,
    utxo: &mut HashMap<String, u128>,
) -> bool {

    let u_sum: u128 = units.iter().sum();
    let mut s_sum: u128 = 0;
    for s in senders{
        if !(utxo.contains_key(s)){
            println!("Invalid transaction!\n");
            return false;
        }
        else{
            s_sum += utxo.get(s).unwrap();
        }
    }
    if u_sum > s_sum{
        println!("Invalid transaction!\n");
        return false;
    }

    let transaction: Transaction = Transaction {
        senders: senders.clone(),
        receivers: receivers.clone(),
        units: units.clone(),
        transaction_signature: transaction_signature.to_string(),
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

    let fee: u128 = s_sum - u_sum;
    println!("Transaction fee: {}\n", &fee);
    return true;
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