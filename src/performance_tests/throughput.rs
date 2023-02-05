#[cfg(test)]
mod tests {
    use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
    use crate::components::utxo::UTXO;
    use crate::simulation::KeyMap;
    use crate::utils::sign_and_verify::Verifier;
    use crate::utils::{hash, sign_and_verify};
    use rand_1::rngs::ThreadRng;
    use std::collections::HashMap;
    use std::time::Instant;

    fn test_tx_throughput(flag: usize) {
        let base: u32 = 2;
        let mut multiplicative_index: u32;
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

                    for b in 0..max_base + 1 {
                        let exponential = new_base.pow(b.try_into().unwrap());
                        let mut batch_size = 0;

                        if (exponential <= 65536
                            || exponential == new_base.pow(max_base.try_into().unwrap()))
                        {
                            batch_size = exponential;
                        } else {
                            continue;
                        };

                        start = Instant::now();
                        let (result, _) = utxo_copy
                            .parallel_batch_verify_and_update(&transactions, batch_size as usize);
                        assert!(result);
                        let duration = start.elapsed();

                        println!(
                            "Time elapsed for {:#} transactions in Run {:#} for batch size {:#} is: {:?}. The number of threads is {:?}.",
                            val, r, batch_size, duration, val/batch_size
                        );
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
