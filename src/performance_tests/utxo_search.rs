#[cfg(test)]
mod tests {
    use crate::components::transaction::{Outpoint, PublicKeyScript, TxOut};
    use crate::components::utxo::UTXO;
    use crate::utils::sign_and_verify::Verifier;
    use crate::utils::{hash, sign_and_verify};
    use rand_1::rngs::ThreadRng;
    use rand_1::Rng;
    use std::collections::HashMap;
    use std::time::Instant;

    #[ignore]
    #[test]
    fn test_utxo_search_time() {
        let mut rng: ThreadRng = rand_1::thread_rng(); // A random generator for a utxo index

        let num_elements_vec: Vec<usize> = vec![1, 10, 100, 1000, 10_000, 100_000, 1_000_000];
        for num_elements in num_elements_vec.iter() {
            print!("10 runs for {} elements:", num_elements);

            // Populate utxo
            let mut utxo: UTXO = UTXO(HashMap::new());
            let mut search_key: Outpoint = Outpoint {
                txid: "".to_string(),
                index: 0,
            };

            let i = rng.gen_range(0..*num_elements);
            for n in 0..*num_elements {
                let (_, public_key) = sign_and_verify::create_keypair();
                let outpoint: Outpoint = Outpoint {
                    txid: "0".repeat(64),
                    index: 0,
                };

                if i == n {
                    search_key = outpoint.clone();
                }

                let tx_out: TxOut = TxOut {
                    value: 500,
                    pk_script: PublicKeyScript {
                        public_key_hash: hash::hash_as_string(&public_key),
                        verifier: Verifier {},
                    },
                };

                utxo.insert(outpoint.clone(), tx_out);
            }

            for _ in 0..10 {
                let key = &search_key;
                let start = Instant::now();
                utxo.get(key);
                let duration = start.elapsed().as_nanos();
                print!(" {}ns", duration);
            }

            println!();
        }
    }
}
