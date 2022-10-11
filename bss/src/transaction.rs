use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Distribution, Exp};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::vec::Vec;
use std::{thread, time};

#[derive(Clone, Debug, Serialize)]
pub struct Transaction {
    pub senders: Vec<String>,
    pub receivers: Vec<String>,
    pub units: Vec<u128>,
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
    pub fn generate_transactions(
        max_num_senders: usize,
        max_num_receivers: usize,
        mean_transaction_rate: f64,
        multiplier: u64,
        transmitter: Sender<Vec<Transaction>>,
        utxo: HashMap<String, u128>,
    ) {
        if max_num_receivers <= 0 || max_num_receivers > utxo.len() {
            panic!("Invalid input. The max number of receivers must be larger than zero and no larger than {} but was {}", utxo.len(), max_num_receivers);
        }
        if max_num_senders <= 0 {
            panic!("Invalid input. The max number of senders must be larger than zero and no larger than {} but was {}", utxo.len(), max_num_senders);
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

        let transactions: &mut Vec<Transaction> = &mut Vec::new();
        loop {
            sample = exp.sample(&mut rng);
            // For an exponential distribution (with lambda > 0), the values range from (0, lambda].
            // Since mean = 1 / lambda, multiply the sample by the mean to normalize.
            normalized = sample * mean_transaction_rate;
            // Get the time between transactions generated as a duration
            transaction_rate = time::Duration::from_secs(multiplier * normalized as u64);
            // Sleep to mimic the time between creation of transactions
            thread::sleep(transaction_rate);

            transactions.push(Self::create_transaction(
                &utxo,
                &mut rng,
                max_num_senders,
                max_num_receivers,
            ));
            transmitter.send(transactions.to_vec()).unwrap();
        }
    }

    fn create_transaction(
        utxo: &HashMap<String, u128>,
        rng: &mut ThreadRng,
        max_num_senders: usize,
        max_num_receivers: usize,
    ) -> Transaction {
        let mut address_list: Vec<String> = Vec::new();
        for (address, _) in utxo {
            address_list.push(address.to_string());
        }

        let num_senders: usize = rng.gen_range(1..=max_num_senders);
        let num_receivers: usize = rng.gen_range(1..=max_num_receivers);

        let senders: Vec<String> = address_list
            .choose_multiple(rng, num_senders)
            .cloned()
            .collect();
        let receivers: Vec<String> = address_list
            .choose_multiple(rng, num_receivers)
            .cloned()
            .collect();

        let mut total_balance: u128 = 0;
        for sender in &senders {
            total_balance += utxo.get(sender).unwrap();
        }

        let mut units: Vec<u128> = Vec::new();
        let mut unit_sum: u128 = 0;
        for _ in 0..num_receivers {
            let new_unit = rng.gen_range(1..=total_balance);
            unit_sum += new_unit;
            units.push(new_unit);
        }
        for unit in units.iter_mut() {
            *unit *= total_balance / unit_sum;
        }

        return Transaction {
            senders: senders.clone(),
            receivers: receivers.clone(),
            units: units.clone(),
        };
    }
}

mod tests {
    use super::*;

    #[test]
    fn create_transaction_valid() {
        let mut utxo: HashMap<String, u128> = HashMap::new();
        utxo.insert(String::from("a"), 50);
        utxo.insert(String::from("b"), 20);
        utxo.insert(String::from("c"), 10);
        utxo.insert(String::from("d"), 30);
        utxo.insert(String::from("e"), 80);
        utxo.insert(String::from("f"), 40);

        let mut rng: ThreadRng = rand::thread_rng();
        let max_num_senders: usize = 3;
        let max_num_receivers: usize = 4;

        let transaction =
            Transaction::create_transaction(&utxo, &mut rng, max_num_senders, max_num_receivers);

        assert!(transaction.senders.len() > 0 && transaction.senders.len() <= max_num_senders);
        assert!(
            transaction.receivers.len() > 0 && transaction.receivers.len() <= max_num_receivers
        );
        assert_eq!(transaction.receivers.len(), transaction.units.len());
    }
}
