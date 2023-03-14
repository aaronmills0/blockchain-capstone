#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        time::{Duration, Instant},
    };

    use rand_1::rngs::ThreadRng;

    use crate::{
        components::{
            merkle::Merkle,
            transaction::{Outpoint, PublicKeyScript, Transaction, TxOut},
            utxo::UTXO,
        },
        simulation::KeyMap,
        utils::{
            hash,
            sign_and_verify::{self, Verifier},
        },
    };

    fn get_transactions(num: u32) -> Vec<Transaction> {
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: KeyMap = KeyMap(HashMap::new());
        let mut transactions: Vec<Transaction> = Vec::new();
        let (private_key0, public_key0) = sign_and_verify::create_keypair();
        let outpoint0: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };

        let tx_out0: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0),
                verifier: Verifier {},
            },
        };

        key_map.insert(outpoint0.clone(), (private_key0, public_key0));
        utxo.insert(outpoint0, tx_out0);

        let mut rng: ThreadRng = rand_1::thread_rng();
        let max_num_outputs = 1;
        let mut utxo_copy = utxo.clone();
        for _ in 0..num {
            let transaction = Transaction::create_transaction(
                &utxo,
                &mut key_map,
                &mut rng,
                max_num_outputs,
                false,
            );
            utxo.update(&transaction);
            transactions.push(transaction);
        }
        return transactions;
    }

    #[ignore]
    #[test]
    fn test_merkle_performance() {
        println!("Num CPUs: {}", num_cpus::get());
        let mut num_transactions = 1;
        let mut performance_results: Vec<Duration> = Vec::new();
        let mut num_transactions_vec: Vec<u32> = Vec::new();
        for _i in 0..=16 {
            let transactions = get_transactions(num_transactions);
            let start = Instant::now();
            Merkle::create_merkle_tree(&transactions);
            let duration = start.elapsed();
            performance_results.push(duration);
            num_transactions_vec.push(num_transactions);
            num_transactions = num_transactions * 2;
        }

        println!("{:?}", num_transactions_vec);
        println!("{:?}", performance_results);
    }
}
