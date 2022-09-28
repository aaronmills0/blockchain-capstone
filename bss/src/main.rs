
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

    utxo.insert(String::from("a"), 50);
    utxo.insert(String::from("b"), 20);

    loop {
        let mut sender: String = String::new();
        let mut receiver: String = String::new();
        let mut units_string: String = String::new();

        println!("Current UTXO:");
        for (key, value) in &utxo {
            println!("{} : {}", key, value);
        }
        println!();

        println!("Current transaction chain:");
        for transaction in &transaction_list {
            print!("[ {} -> {}, {}] ", transaction.sender, transaction.receiver, transaction.units);
        }
        println!();

        println!("New transaction");
        println!();

        println!("Sender address:");
        io::stdin().read_line(&mut sender).expect("Failed to read line");
        sender = sender.trim().to_string();
        println!("Receiver address:");
        io::stdin().read_line(&mut receiver).expect("Failed to read line");
        receiver = receiver.trim().to_string();
        println!("Transfer quantity:");
        io::stdin().read_line(&mut units_string).expect("Failed to read line");
        println!();


        let units: u128 = match units_string.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        let optional_balance = utxo.get(&sender);

        if optional_balance.is_none() || *optional_balance.unwrap() < units {
            println!("Invalid transaction");
            println!();
            continue
        }

        let balance: u128 = *optional_balance.unwrap();

        let transaction: Transaction = Transaction {
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            units: units,
        };

        transaction_list.push(transaction);

        let fee: u128 = balance - units;

        utxo.remove(&sender);

        if !utxo.contains_key(&receiver) {
            utxo.insert(receiver, units);
        } else {
            *utxo.get_mut(&receiver).unwrap() += units;
        }

        println!("Transaction fee: {}", &fee);
        println!();
    }

}
