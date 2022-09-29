
use std::collections::HashMap;
use std::vec::Vec;
use std::io;
mod transaction;
use transaction::Transaction;
fn main() {
    println!("Welcome to the simple transaction chain!");
    println!();

    let mut utxo: HashMap<String, u128> = HashMap::new();

    let mut transaction_list: Vec<Transaction> = Vec::new();

    const BLOCK_SIZE: u128 = 1;

    utxo.insert(String::from("a"), 50);
    utxo.insert(String::from("b"), 20);

    'main: loop {
        let mut senders: Vec<String> = Vec::new();
        let mut receivers: Vec<String> = Vec::new();
        let mut units: Vec<u128> = Vec::new();
        let mut sender_input = String::new();
        let mut user_input = String::new();

        println!("Current UTXO:");
        for (key, value) in &utxo {
            println!("{} : {}", key, value);
        }
        println!();

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

        let mut input_sum: HashMap<String, u128> = HashMap::new();
        for (i, val) in senders.iter().enumerate() {
            if  input_sum.contains_key(val) {
                *input_sum.get_mut(val).unwrap() += units[i];
            }
            else {
                input_sum.insert(val.to_string(), units[i]);
            }
        }

        for (sender, value) in &input_sum {
            if !(utxo.contains_key(sender)) || value > utxo.get(sender).unwrap() {
                println!("Invalid transaction");
                println!();
                continue 'main;
            }
        }

        let transaction: Transaction = Transaction {
            senders: senders.clone(),
            receivers: receivers.clone(),
            units: units.clone(),
        };

        transaction_list.push(transaction);

        // Update UTXO
        //let fee: u128 = balance - units;
        for (input, val) in &input_sum {
            utxo.remove(input);
        }

        for (i, r) in receivers.iter().enumerate() {
            if !utxo.contains_key(r) {
                utxo.insert(r.to_string(), units[i]);
            }
            else {
                *utxo.get_mut(r).unwrap() += units[i];
            }
        }

        // println!("Transaction fee: {}", &fee);
        // println!();*/
    }

}
