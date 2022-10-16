use crate::hash;
use crate::signer_and_verifier;
use crate::utxo::UTXO;

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Alphanumeric, Distribution, Exp};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::vec::Vec;
use std::{thread, time};

#[derive(Clone, Debug, Serialize)]
pub struct Transaction {
    pub senders: Vec<String>,
    pub receivers: Vec<String>,
    pub units: Vec<u128>,
    pub transaction_signature: String,
}

impl Transaction {
    /**
     * Creates transactions at random times that follow an exponential distribution given by a specified mean
     * The transactions will be sent and received by existing addresses in the utxo
     * The amount of senders and receivers is also random (uniform), but a maximum is specified
     * The amount received for each address is random (uniform) but is normalized so that the total amount received
     * is (approximately) equal to the total balance of the senders
     *
     * The transaction list created is constantly transmitted so that the block generator can receive it
     */
    pub fn transaction_generator(
        max_num_receivers: usize,
        mean_transaction_rate: f64,
        multiplier: u64,
        transmitter: Sender<Transaction>,
        receiver: Receiver<UTXO>,
        mut utxo: UTXO,
    ) {
        if max_num_receivers <= 0 {
            panic!("Invalid input. The max number of receivers must be larger than zero and no larger than {} but was {}", utxo.map.len(), max_num_receivers);
        }
        if mean_transaction_rate <= 0.0 {
            panic!("Invalid input. A non-positive mean for transaction rate is invalid for an exponential distribution but the mean was {}", mean_transaction_rate);
        }

        let lambda: f64 = 1.0 / mean_transaction_rate;
        let exp: Exp<f64> = Exp::new(lambda).unwrap();
        let mut rng: ThreadRng = rand::thread_rng();
        let mut sample: f64;
        let mut normalized: f64;
        let mut transaction_rate: time::Duration;
        let mut verified_utxo = utxo.clone();
        loop {
            sample = exp.sample(&mut rng);
            // For an exponential distribution (with lambda > 0), the values range from (0, lambda].
            // Since mean = 1 / lambda, multiply the sample by the mean to normalize.
            normalized = sample * mean_transaction_rate;
            // Get the time between transactions generated as a duration
            transaction_rate = time::Duration::from_secs(multiplier * normalized as u64);
            // Sleep to mimic the time between creation of transactions
            thread::sleep(transaction_rate);

            let transaction = Self::create_transaction(&utxo, &mut rng, max_num_receivers);
            utxo.update(&transaction);
            transmitter.send(transaction).unwrap();

            let new_utxo = receiver.try_recv();
            if new_utxo.is_ok() {
                verified_utxo = new_utxo.unwrap();
            }
        }
    }

    fn create_transaction(
        utxo: &UTXO,
        rng: &mut ThreadRng,
        max_num_receivers: usize,
    ) -> Transaction {
        let mut address_list: Vec<String> = Vec::new();
        for (address, _) in utxo.map.iter() {
            address_list.push(address.to_string());
        }

        let num_senders: usize = rng.gen_range(1..=utxo.map.len());
        let num_receivers: usize = rng.gen_range(1..=max_num_receivers);

        let senders: Vec<String> = address_list
            .choose_multiple(rng, num_senders)
            .cloned()
            .collect();

        let mut receivers_set: HashSet<String> = HashSet::new();
        let mut counter: usize = 0;
        while counter < num_receivers {
            let s: String = rng
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();
            if receivers_set.contains(&s) {
                continue;
            }
            counter += 1;
            receivers_set.insert(s);
        }

        let mut total_balance: u128 = 0;
        for sender in &senders {
            total_balance += utxo.map.get(sender).unwrap();
        }

        let mut units: Vec<u128> = Vec::new();
        let mut unit_sum: u128 = 0;
        let mut value_sum: u128 = 0;
        for _ in 0..num_receivers {
            let new_value = rng.gen_range(1..=100);
            value_sum += new_value;
            units.push(new_value);
        }
        for unit in units.iter_mut() {
            *unit *= total_balance / value_sum;
            unit_sum += *unit;
        }
        units[0] += total_balance - unit_sum;

        let mut transaction_senders = String::new();
        for s in &senders {
            transaction_senders.push_str(&s);
        }
        let transaction_hash: String = hash::hash_as_string(&transaction_senders);
        let (secret_key, public_key) = signer_and_verifier::create_keypair();
        let signature_of_sender = signer_and_verifier::sign(&transaction_hash, &secret_key);
        let transaction_signature = signature_of_sender.to_string();
        let verified =
            signer_and_verifier::verify(&transaction_hash, &signature_of_sender, &public_key);
        println!(
            "\nTransaction created with {} senders and {} receivers.\n\tOwner public key: {}. \n\tTransaction signature: {}. \n\tSignature verified: {}",
            num_senders, num_receivers, public_key, transaction_signature, verified
        );
        return Transaction {
            senders: senders,
            receivers: Vec::from_iter(receivers_set),
            units: units,
            transaction_signature: transaction_signature,
        };
    }
}

mod tests {
    use crate::{transaction::Transaction, utxo::UTXO};

    use rand::rngs::ThreadRng;
    use std::collections::HashMap;

    #[test]
    fn create_transaction_valid() {
        let mut utxo: UTXO = UTXO {
            map: HashMap::new(),
        };
        utxo.map.insert(String::from("a"), 50);
        utxo.map.insert(String::from("b"), 20);
        utxo.map.insert(String::from("c"), 10);
        utxo.map.insert(String::from("d"), 30);
        utxo.map.insert(String::from("e"), 80);
        utxo.map.insert(String::from("f"), 40);

        let mut rng: ThreadRng = rand::thread_rng();
        let max_num_receivers: usize = 4;

        let transaction = Transaction::create_transaction(&utxo, &mut rng, max_num_receivers);

        assert!(transaction.senders.len() > 0 && transaction.senders.len() <= utxo.map.len());
        assert!(
            transaction.receivers.len() > 0 && transaction.receivers.len() <= max_num_receivers
        );
        assert_eq!(transaction.receivers.len(), transaction.units.len());
    }
}
