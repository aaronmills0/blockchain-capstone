#[cfg(test)]
mod tests {
    use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
    use crate::components::utxo::UTXO;
    use crate::simulation::KeyMap;
    use crate::utils::sign_and_verify::Verifier;
    use crate::utils::{hash, sign_and_verify};
    use csv::Error;
    use rand_1::rngs::ThreadRng;
    use std::collections::HashMap;
    use std::time::Instant;

    fn test_tx_throughput(flag: usize) {
        let base: u32 = 2;
        let mut multiplicative_index: u32;

        let mut writer = csv::Writer::from_path("speedup_data.csv").unwrap();
        writer.write_record(&[
            "Number of transactions",
            "S (number of threads)",
            "P (share of parallelized program)",
            "Time in ms",
            "Expected Amdhal Speedup",
            "Real Speedup",
        ]);

        for r in 0..5 {
            for k in 0..20 {
                // val is the number of transactions to be used. For parallel verification, it is also the batch size.
                let val = base.pow(k.try_into().unwrap());
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
                for _ in 0..val {
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

                assert_eq!(transactions.len() as u32, val);

                let mut start = Instant::now();
                if flag == 0 {
                    for tx in transactions.iter() {
                        if !utxo_copy.verify_transaction(tx) {
                            println!("Validator received block containing invalid transactions. Ignoring block");
                            continue;
                        }
                        utxo_copy.update(tx);
                    }
                } else if flag == 1 {
                    let (result, _) = utxo_copy.batch_verify_and_update(&transactions);
                    assert!(result);
                } else {
                    // max_base is the base 2 logarithm of val which correpsonds to the number of batch sizes
                    // we will need to iterate over minus 1 (to account for 0). For example if the batch size is 16,
                    // then the number of iterations is log(16)=4 for 5 batch sizes (1,2,4,8,16)
                    let max_base = val.ilog2();
                    let new_base: u32 = 2;
                    // P is the parallellized share of the program
                    let mut P: f32 = 0.0;
                    let mut sequential_time: f32 = 0.0;

                    for b in (0..max_base + 1).rev() {
                        let exponential = new_base.pow(b.try_into().unwrap());
                        let mut batch_size = exponential;

                        start = Instant::now();
                        let (result, _, share_parallelized_program, full_duration) = utxo_copy
                            .parallel_batch_verify_and_update(&transactions, batch_size as usize);
                        assert!(result);
                        let duration = start.elapsed();

                        let number_of_threads = val / batch_size;

                        if number_of_threads == 1 {
                            P = share_parallelized_program;
                            sequential_time = full_duration as f32;
                        }

                        println!(
                            "Time elapsed for {:#} transactions in Run {:#} for batch size {:#} is: {:?}. The number of threads is {:?}.",
                            val, r, batch_size, duration, number_of_threads
                        );

                        let expected_speedup: f32 =
                            1.0 / ((1.0 - P) + (P / number_of_threads as f32));
                        let real_speedup: f32 = sequential_time / (full_duration as f32);

                        writer.write_record(&[
                            val.to_string(),
                            number_of_threads.to_string(),
                            P.to_string(),
                            full_duration.to_string(),
                            expected_speedup.to_string(),
                            real_speedup.to_string(),
                        ]);
                        writer.flush();
                        println!();
                    }
                    continue;
                }
                let duration = start.elapsed();

                println!(
                    "Time elapsed for {:#} transactions in Run {:#} is: {:?}",
                    val, r, duration
                );
                println!();
            }
        }
    }

    // Use flag 0 for sequential verification, 1 for batch verification, 2 for parallel verification
    #[ignore]
    #[test]
    fn test_transaction_throughput() {
        let mut flag: usize = 0;
        /*test_tx_throughput(flag);
        flag = 1;
        test_tx_throughput(flag);*/
        flag = 2;
        test_tx_throughput(flag);
    }
}
