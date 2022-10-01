
use std::collections::HashMap;
use std::vec::Vec;
use std::io;
mod block;
mod transaction;
mod merkle;
mod hash;
use transaction::Transaction;
use block::Block;
use block::BlockHeader;
use merkle::Merkle;

fn main() {
    println!("Welcome to the simple transaction chain!");
    println!();   
    //Initialize UTXO
    let mut utxo: HashMap<String, u128> = HashMap::new();
    utxo.insert(String::from("a"), 50);
    utxo.insert(String::from("b"), 20);

    //Create/add genesis block
    let genesis_merkle: Merkle = Merkle { 
        tree: Vec::from(["0000000000000000000000000000000000000000000000000000000000000000".to_string()])
    };
    let genesis_block: Block = Block {
        header: BlockHeader {
            previous_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            merkle_root: genesis_merkle.tree.first().unwrap().clone(),
            nonce: 0,
        },
        transactions: genesis_merkle,
    };
    let mut blockchain: Vec<Block> = Vec::new();
    blockchain.push(genesis_block);

    //Block creation data
    const BLOCK_SIZE: u128 = 1;
    let mut transaction_list: Vec<Transaction> = Vec::new();
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
            let merkle: Merkle = Merkle::create_merkle_tree(&transaction_list);
            let  new_block: Block = Block {
                header: BlockHeader {
                    previous_hash: hash::hash_as_string(blockchain.last().unwrap()),
                    merkle_root: merkle.tree.first().unwrap().clone(),
                    nonce: 0,
                },
                transactions: merkle,
            };
            blockchain.push(new_block);

            println!("A block was added to the chain!");
            for block in &blockchain {
                if hash::hash_as_string(block) == "0000000000000000000000000000000000000000000000000000000000000000" {
                    print!("Block {}", hash::hash_as_string(block));
                    continue;
                }
                print!(" <= Block {}", hash::hash_as_string(block));
            }
            transaction_list.clear();
            print!("\n");
        }
        
    }

}
