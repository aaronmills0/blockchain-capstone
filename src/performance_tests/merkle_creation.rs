#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        time::{Duration, Instant},
    };

    use itertools::izip;
    use rand_1::rngs::ThreadRng;

    use crate::{
        components::{
            merkle::{self, Merkle},
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
        for r in 0..5 {
            let path = format!("speedup_data{}.csv", r);
            let mut writer = csv::Writer::from_path(path).unwrap();
            writer.write_record(&[
                "Number of transactions",
                "Number of threads",
                "Share of parallelized program in sequential run",
                "Time in ms",
                "Expected Amdhal Speedup",
                "Real Speedup",
            ]);

            let mut P: f32 = 0.0;
            let base: u32 = 2;
            let total_number_transactions = 10; // CHANGE THIS. THIS IS FOR 1024 TX (2 power of 10)
            for exp in 2..=total_number_transactions {
                let mut is_parallel: bool = false;
                let mut num_threads: usize = 0;
                let mut num_transactions = base.pow(exp.try_into().unwrap());
                let transactions = get_transactions(num_transactions);
                let start = Instant::now();
                (_, P) = Merkle::create_merkle_tree(&transactions, is_parallel, num_threads);
                let sequential_duration = start.elapsed();

                writer.write_record(&[
                    num_transactions.to_string(),
                    "1".to_string(),
                    P.to_string(),
                    sequential_duration.as_millis().to_string(),
                    "1".to_string(),
                    "1".to_string(),
                ]);
                writer.flush();

                for count_threads in 1..=exp {
                    num_threads = base.pow(count_threads.try_into().unwrap()) as usize;
                    is_parallel = true;
                    let start = Instant::now();
                    Merkle::create_merkle_tree(&transactions, is_parallel, num_threads);
                    let duration = start.elapsed();
                    let current_experimental_speedup =
                        sequential_duration.as_micros() as f32 / duration.as_micros() as f32;
                    let current_Amdahl_speedup = 1.0 / ((1.0 - P) + (P / num_threads as f32));

                    writer.write_record(&[
                        num_transactions.to_string(),
                        num_threads.to_string(),
                        P.to_string(),
                        duration.as_millis().to_string(),
                        current_Amdahl_speedup.to_string(),
                        current_experimental_speedup.to_string(),
                    ]);
                    writer.flush();
                }
            }
        }
    }
}
