use crate::transaction::Transaction;
use std::collections::HashMap;

#[derive(Clone)]
pub struct UTXO {
    pub map: HashMap<String, u128>,
}

impl UTXO {
    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        let mut balance: u128 = 0;
        let mut transfer_quantity: u128 = 0;
        for sender in transaction.senders.iter() {
            // If the uxto doesn't contain the sender: invalid transaction
            if !self.map.contains_key(sender) {
                println!("Invalid transaction! The utxo does not contain the address {sender}.");
                return false;
            }
            // Otherwise, increment the total balance by the sender's balance
            balance += self.map.get(sender).unwrap();
        }

        // Obtain the total amount that is requested to be transferred
        for quantity in transaction.units.iter() {
            transfer_quantity += *quantity;
        }

        // If we do not have the balance to fulfill this transaction, return false.
        if transfer_quantity > balance {
            println!(
                "Invalid transaction! The total available balance cannot support this transaction."
            );
            return false;
        }

        return true;
    }

    pub fn update(&mut self, transaction: &Transaction) {
        for sender in transaction.senders.iter() {
            self.map.remove(sender);
        }

        // Iterate through the transfer quantity - receiver pairs
        for (quantity, receiver) in transaction.units.iter().zip(transaction.receivers.iter()) {
            // If the receiver address is not present in the utxo, create a new entry with the corresponding quantity
            if !self.map.contains_key(receiver) {
                self.map.insert(receiver.to_string(), *quantity);
            } else {
                // Otherwise, increment the receiver's balance by the quantity
                *self.map.get_mut(receiver).unwrap() += *quantity;
            }
        }
    }
}
