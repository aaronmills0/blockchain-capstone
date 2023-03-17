#[cfg(test)]
mod tests {
    use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
    use crate::components::utxo::UTXO;
    use crate::simulation::KeyMap;
    use crate::utils::save_and_load::{load_object, save_object};
    use crate::utils::sign_and_verify::Verifier;
    use crate::utils::{hash, sign_and_verify};
    use csv::Error;
    use rand_1::rngs::ThreadRng;
    use std::cmp::max;
    use std::collections::HashMap;
    use std::time::Instant;

    fn test_tx_throughput(flag: usize) {
        let base: u32 = 2;
        let mut multiplicative_index: u32;
        for r in 0..5 {
            let path = format!("throughput_data_multiple_transactions{}.csv", r);
            let mut writer = csv::Writer::from_path(path).unwrap();
            writer.write_record(&["Number of transactions", "Time in ms", "Throughput"]);

            for k in 0..19 {
                let val = base.pow(k.try_into().unwrap());
                multiplicative_index = val;

                let mut utxo: UTXO = UTXO(HashMap::new());
                let mut key_map: KeyMap = KeyMap(HashMap::new());
                let mut transactions: Vec<Transaction> = Vec::new();
                let mut loaded_transactions: Vec<Transaction> = Vec::new();
                let (private_key0, public_key0) = sign_and_verify::create_keypair();
                let hash_public_key0 = hash::hash_as_string(&public_key0);

                let outpoint0: Outpoint = Outpoint {
                    txid: "0".repeat(64),
                    index: 0,
                };

                let tx_out0: TxOut = TxOut {
                    value: 500,
                    pk_script: PublicKeyScript {
                        public_key_hash: hash_public_key0.clone(),
                        verifier: Verifier {},
                    },
                };

                key_map.insert(outpoint0.clone(), (private_key0, public_key0));
                utxo.insert(outpoint0, tx_out0);

                let mut rng: ThreadRng = rand_1::thread_rng();
                let max_num_outputs = 1;
                let mut utxo_copy = utxo.clone();
                for _ in 0..multiplicative_index {
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

                save_object(
                    &transactions,
                    String::from("transactions"),
                    String::from("account"),
                );

                assert_eq!(transactions.len() as u32, multiplicative_index);

                let start = Instant::now();
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
                } else if flag == 2 {
                    loaded_transactions =
                        load_object(String::from("transactions"), String::from("account"));
                    let (result, _) = utxo_copy.batch_verify_and_update(&loaded_transactions);
                    assert!(result);
                } else {
                    loaded_transactions =
                        load_object(String::from("transactions"), String::from("account"));
                    let (result, _) = utxo_copy.parallel_batch_verify_and_update(
                        &loaded_transactions,
                        (max(multiplicative_index, num_cpus::get() as u32) as usize
                            / num_cpus::get()),
                    );
                    assert!(result);
                }
                let duration = start.elapsed();
                let mut throughput: f32 =
                    1000000.0 * (multiplicative_index as f32 / duration.as_micros() as f32);

                writer.write_record(&[
                    multiplicative_index.to_string(),
                    duration.as_millis().to_string(),
                    throughput.to_string(),
                ]);
                writer.flush();
                println!();

                println!(
                    "Time elapsed for {:#} transactions in Run {:#} is: {:?}",
                    multiplicative_index, r, duration
                );
                println!();
            }
        }
    }

    // Use flag 0 for sequential verification and any other flag for batch verification
    #[ignore]
    #[test]
    fn test_transaction_throughput() {
        let mut flag: usize = 3;
        test_tx_throughput(flag);
        /* test_tx_throughput(flag);
        flag = 1;
        test_tx_throughput(flag); */
    }
}
