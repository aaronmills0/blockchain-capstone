use std::collections::HashMap;
use std::vec::Vec;
use std::io;
mod structs;
mod hash;
mod signer_and_verifier;
use structs::Transaction;
use structs::Block;

fn main() {
    println!("Welcome to the simple transaction chain!");
    println!();

    let mut transaction_list: Vec<Transaction> = Vec::new();
    
    //Initialize UTXO
    let mut utxo: HashMap<String, u128> = HashMap::new();
    utxo.insert(String::from("a"), 50);
    utxo.insert(String::from("b"), 20);

    //Create/add genesis block
    let genesis_block: Block = Block {
        block_id: 0,
        transactions: Vec::new(),
    };
    let mut blockchain: Vec<Block> = Vec::new();
    blockchain.push(genesis_block);

    //Block creation data
    const BLOCK_SIZE: u128 = 1;
    let mut block_id: u128 = 1;

    //Runs once for each transaction
    loop {
        let mut senders: Vec<String> = Vec::new();
        let mut receivers: Vec<String> = Vec::new();
        let mut units: Vec<u128> = Vec::new();
        let mut sender_input = String::new();
        let mut user_input = String::new();

        //Display current UTXO state
        println!("Current UTXO:");
        for (key, value) in &utxo {
            println!("{} : {}", key, value);
        }
        println!();

        //Display current transaction chain state
        println!("Current transaction chain:");
        for transaction in &transaction_list {
            for (i, val) in transaction.senders.iter().enumerate() {
                print!("[");
                print!("[ {} -> {}, {}]", val, transaction.receivers[i], transaction.units[i]);
                print!("]");
            }
        }
        println!();

        println!("New transaction");
        println!();
        
        'txio: loop {
            //Receive and Proccess user Input-Output-Unit Pairs
            println!("Sender address:");
            user_input.clear();
            io::stdin().read_line(&mut sender_input).expect("Failed to read line");
            senders.push(sender_input.trim().to_string());
            
            println!("Receiver address:");
            user_input.clear();
            io::stdin().read_line(&mut user_input).expect("Failed to read line");
            receivers.push(user_input.trim().to_string());

            println!("Transfer quantity:");
            user_input.clear();
            io::stdin().read_line(&mut user_input).expect("Failed to read line");
            let unit: u128 = match user_input.trim().parse() {
                Ok(num) => num,
                Err(_) => return,
            };
            units.push(unit);
            
            
            println!("Would you like to add another input -> output? y/n?");
            user_input.clear();
            io::stdin().read_line(&mut user_input).expect("Failed to read line");
            sender_input.clear();

            match user_input.trim() {
                "y" => continue 'txio,
                _ => break,
            }

        }

        //Initializes a hashmap used to store the inputs 
        //and the summed units over a transaction
        let mut input_sum: HashMap<String, u128> = HashMap::new();
        for (i, val) in senders.iter().enumerate() {
            if  input_sum.contains_key(val) {
                *input_sum.get_mut(val).unwrap() += units[i];
            }
            else {
                input_sum.insert(val.to_string(), units[i]);
            }
        }

        //Verify validity of transactions
        let mut trans_sum = 0;
        let mut utxo_sum = 0;
        for (sender, value) in &input_sum {
            if !(utxo.contains_key(sender)) || value > utxo.get(sender).unwrap() {
                println!("Invalid transaction");
                return;
            }
            else {
                trans_sum += value;
                utxo_sum += utxo.get(sender).unwrap();
            }
        }

        //Create and add a transaction instance
        let transaction: Transaction = Transaction {
            senders: senders.clone(),
            receivers: receivers.clone(),
            units: units.clone(),
        };
        transaction_list.push(transaction);

        //Generate transaction hash, sign transaction with private key, verify signed transaction with public key
        println!()
        let transaction_hash= hash::hash_as_string(transaction_list.last().unwrap());
        let (signed_transaction, public_key) = signer_and_verifier::sign(&transaction_hash);
        println!("The signed transaction is {}:",signed_transaction);
        println!("The public key for this transaction is {}:", public_key);
        println!("Does the signed transaction correpsond to public key?: {}", signer_and_verifier::verify(&transaction_hash, &signed_transaction, &public_key));
        

        // Update UTXO

        for (key, _) in &input_sum {
            utxo.remove(key);
        }

        let fee: u128 = utxo_sum - trans_sum;

        for (i, r) in receivers.iter().enumerate() {
            if !utxo.contains_key(r) {
                utxo.insert(r.to_string(), units[i]);
            }
            else {
                *utxo.get_mut(r).unwrap() += units[i];
            }
        }

        println!("Transaction fee: {}", &fee);
        println!();

        //Create a block when enough transactions have pilled up
        if transaction_list.len() == BLOCK_SIZE.try_into().unwrap() {
            let  new_block: Block = Block {
                block_id: block_id.clone(),
                transactions: transaction_list.clone(),
            };
            blockchain.push(new_block);
            block_id += 1;

            println!("A block was added to the chain!");
            for block in &blockchain {
                if block.block_id == 0 {
                    print!("Block {}", block.block_id);
                    continue;
                }
                print!(" <= Block {}", block.block_id);
            }
            transaction_list.clear();
            print!("\n");
        }
        
    }

}
