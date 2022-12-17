#[cfg(test)]
mod tests {
    use crate::hash;
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::simulation::KeyMap;
    use crate::transaction::{Outpoint, PublicKeyScript, SignatureScript, TxIn, TxOut};
    use crate::{transaction::Transaction, utxo::UTXO};
    use log::warn;
    use rand_1::rngs::ThreadRng;
    use std::collections::HashMap;
    use std::time::Instant;

    static MAX_NUM_OUTPUTS: usize = 3;

    #[test]
    fn test_transaction_throughput() {
        let base: u32 = 10;
        let mut multiplicative_index: u32 = 0;
        for r in 0..5 {
            for k in 0..10 {
                if ((base.pow(k.try_into().unwrap())) > 100000) {
                    multiplicative_index = 100000 * (k - 4);
                } else {
                    multiplicative_index = base.pow(k.try_into().unwrap());
                }

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

                for n in 0..multiplicative_index {
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
                for tx in transactions.iter() {
                    if !utxo_copy.verify_transaction(tx) {
                        println!("Validator received block containing invalid transactions. Ignoring block");
                        continue;
                    }
                    utxo_copy.update(tx);
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

    #[test]
    fn test_transaction_throughput_batch_verify() {
        let base: u32 = 10;
        let mut multiplicative_index: u32 = 0;
        for r in 0..5 {
            for k in 0..10 {
                if ((base.pow(k.try_into().unwrap())) > 100000) {
                    multiplicative_index = 100000 * (k - 4);
                } else {
                    multiplicative_index = base.pow(k.try_into().unwrap());
                }

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

                for n in 0..multiplicative_index {
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
                let (result, updated_utxo) = utxo_copy.batch_verify_and_update(&transactions);
                assert!(result);
                let duration = start.elapsed();

                println!(
                    "Time elapsed for {:#} transactions in Run {:#} is: {:?}",
                    multiplicative_index, r, duration
                );

                println!();
            }
        }
    }
}
