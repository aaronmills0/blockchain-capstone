#[cfg(test)]
mod tests {
    use crate::components::transaction::{
        Outpoint, PublicKeyScript, SignatureScript, Transaction, TxIn, TxOut,
    };
    use crate::components::utxo::UTXO;
    use crate::utils::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::utils::{hash, sign_and_verify};
    use std::collections::HashMap;
    use std::time::Instant;

    fn one_input_diff_output_transaction_valid(number_of_outputs: usize) -> (Transaction, UTXO) {
        // We first insert an unspent output in the utxo to which we will refer later on.
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
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
        utxo.insert(outpoint0.clone(), tx_out0.clone());

        // We create a signature script for the input of our new transaction
        let (old_private_key, old_public_key) = key_map[&outpoint0].clone();

        let message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        let sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key, &old_public_key),
            full_public_key: old_public_key,
        };

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

        // We create a new keypair corresponding to our new transaction which allows us to create its tx_out
        let (_, public_key1) = sign_and_verify::create_keypair();
        let mut tx_outs: Vec<TxOut> = Vec::new();
        let value_to_be_spent: usize = 500;
        for _ in 1..number_of_outputs {
            tx_outs.push(TxOut {
                value: value_to_be_spent as u32 / number_of_outputs as u32,
                pk_script: PublicKeyScript {
                    public_key_hash: hash::hash_as_string(&public_key1),
                    verifier: Verifier {},
                },
            });
        }

        let transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            tx_outputs: tx_outs,
        };

        return (transaction1, utxo);
    }

    fn diff_input_one_output_transaction_valid(number_of_inputs: usize) -> (Transaction, UTXO) {
        // We first insert an unspent output in the utxo to which we will refer later on.
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
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
        utxo.insert(outpoint0.clone(), tx_out0.clone());

        // We create a signature script for the input of our new transaction
        let (old_private_key, old_public_key) = key_map[&outpoint0].clone();
        let mut tx_ins = Vec::new();
        let message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        let sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key, &old_public_key),
            full_public_key: old_public_key,
        };

        tx_ins.push(TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        });

        for c in 1..number_of_inputs {
            let (private_key, public_key) = sign_and_verify::create_keypair();
            let outpoint: Outpoint = Outpoint {
                txid: "0".repeat(64),
                index: c as u32,
            };

            let tx_out: TxOut = TxOut {
                value: 500,
                pk_script: PublicKeyScript {
                    public_key_hash: hash::hash_as_string(&public_key),
                    verifier: Verifier {},
                },
            };

            key_map.insert(outpoint.clone(), (private_key, public_key));
            utxo.insert(outpoint.clone(), tx_out.clone());

            // We create a signature script for the input of our new transaction
            let (old_private_key, old_public_key) = key_map[&outpoint].clone();
            let message = String::from(&outpoint.txid)
                + &outpoint.index.to_string()
                + &tx_out.pk_script.public_key_hash;

            let sig_script = SignatureScript {
                signature: sign_and_verify::sign(&message, &old_private_key, &old_public_key),
                full_public_key: old_public_key,
            };

            tx_ins.push(TxIn {
                outpoint,
                sig_script,
            });
        }

        // We create a new keypair corresponding to our new transaction which allows us to create its tx_out
        let (_, public_key1) = sign_and_verify::create_keypair();
        let tx_out1: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let transaction1: Transaction = Transaction {
            tx_inputs: tx_ins,
            tx_outputs: Vec::from([tx_out1]),
        };

        return (transaction1, utxo);
    }

    #[ignore]
    #[test]
    fn test_verif_one_input_diff_outputs() {
        let base: usize = 10;
        let mut multiplicative_index: usize;
        for n in 0..10 {
            let val = base.pow(n.try_into().unwrap());
            multiplicative_index = if val > 100000 { 100000 * (n - 4) } else { val };
            let (transaction, utxo) = one_input_diff_output_transaction_valid(multiplicative_index);

            let start = Instant::now();
            utxo.verify_transaction(&transaction);
            let duration = start.elapsed();

            println!(
                "Time elapsed for {:#} is: {:?}",
                multiplicative_index, duration
            );
        }
    }

    #[ignore]
    #[test]
    fn test_verif_diff_input_one_output() {
        let base: usize = 10;
        let mut multiplicative_index: usize;
        for n in 0..10 {
            let val = base.pow(n.try_into().unwrap());
            multiplicative_index = if val > 100000 { 100000 * (n - 4) } else { val };
            let (transaction, utxo) = diff_input_one_output_transaction_valid(multiplicative_index);

            let start = Instant::now();
            utxo.verify_transaction(&transaction);
            let duration = start.elapsed();

            assert_eq!(transaction.tx_inputs.len(), multiplicative_index);

            println!(
                "Time elapsed for {:#} is: {:?}",
                multiplicative_index, duration
            );
        }
    }
}
