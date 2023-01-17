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
        let base: u32 = 10;
        let mut multiplicative_index: u32;
        for r in 0..5 {
            for k in 0..10 {
                let val = base.pow(k.try_into().unwrap());
                multiplicative_index = if val > 100000 { 100000 * (k - 4) } else { val };

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
                } else {
                    let (result, _) = utxo_copy.batch_verify_and_update(&transactions);
                    assert!(result);
                }
                let duration = start.elapsed();

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
        let mut flag: usize = 0;
        test_tx_throughput(flag);
        flag = 1;
        test_tx_throughput(flag);
    }
}
