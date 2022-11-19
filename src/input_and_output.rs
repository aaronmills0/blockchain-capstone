#[cfg(test)]
mod tests {
    use crate::hash;
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::transaction::Transaction;
    use crate::transaction::{Outpoint, PublicKeyScript, SignatureScript, TxIn, TxOut};
    use crate::utxo::UTXO;
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    fn one_input_diff_output_transaction_valid(number_of_outputs: u32) -> (Transaction, UTXO) {
        //We first insert an unspent output in the utxo to which we will
        //refer later on.
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

        //We create a signature script for the input of our new transaction
        let mut sig_script1: SignatureScript;

        let mut old_private_key: PrivateKey;
        let mut old_public_key: PublicKey;

        (old_private_key, old_public_key) = key_map[&outpoint0].clone();

        let mut message: String;

        message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key),
            full_public_key: old_public_key,
        };

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (private_key1, public_key1) = sign_and_verify::create_keypair();

        let mut count = 0;

        let mut tx_outs: Vec<TxOut> = Vec::new();

        let value_to_be_spent: u32 = 500;

        loop {
            tx_outs.push(TxOut {
                value: value_to_be_spent / number_of_outputs,
                pk_script: PublicKeyScript {
                    public_key_hash: hash::hash_as_string(&public_key1),
                    verifier: Verifier {},
                },
            });
            count = count + 1;
            if count == number_of_outputs {
                break;
            }
        }

        let mut transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            txin_count: 1,
            tx_outputs: tx_outs,
            txout_count: 1,
        };

        return (transaction1, utxo);
    }

    fn diff_input_one_output_transaction_valid(number_of_inputs: usize) -> (Transaction, UTXO) {
        //We first insert an unspent output in the utxo to which we will
        //refer later on.
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

        //We create a signature script for the input of our new transaction
        let mut sig_script1: SignatureScript;

        let mut old_private_key: PrivateKey;
        let mut old_public_key: PublicKey;

        (old_private_key, old_public_key) = key_map[&outpoint0].clone();

        let mut tx_ins = Vec::new();
        let mut message: String;

        message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key),
            full_public_key: old_public_key,
        };

        tx_ins.push(TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        });

        let mut count = 1;

        loop {
            if count == number_of_inputs {
                break;
            }

            let (private_key, public_key) = sign_and_verify::create_keypair();
            let outpoint: Outpoint = Outpoint {
                txid: "0".repeat(64),
                index: count as u32,
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

            //We create a signature script for the input of our new transaction
            let mut sig_script: SignatureScript;

            let mut old_private_key: PrivateKey;
            let mut old_public_key: PublicKey;

            (old_private_key, old_public_key) = key_map[&outpoint].clone();
            let mut message: String;

            message = String::from(&outpoint.txid)
                + &outpoint.index.to_string()
                + &tx_out.pk_script.public_key_hash;

            sig_script = SignatureScript {
                signature: sign_and_verify::sign(&message, &old_private_key),
                full_public_key: old_public_key,
            };

            tx_ins.push(TxIn {
                outpoint: outpoint,
                sig_script: sig_script,
            });

            count = count + 1;
        }

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let mut transaction1: Transaction = Transaction {
            tx_inputs: tx_ins,
            txin_count: 1,
            tx_outputs: Vec::from([tx_out1]),
            txout_count: 1,
        };

        return (transaction1, utxo);
    }
    /*
    #[test]
    fn test_one_input_hundred_outputs() {
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut transaction: Transaction;

        let start = Instant::now();
        (transaction, utxo) = one_input_diff_output_transaction_valid(100);
        let duration = start.elapsed();

        println!("Time elapsed is: {:?}", duration);

        assert_eq!(transaction.tx_outputs.len(), 100);
    }

    #[test]
    fn test_one_input_thousand_outputs() {
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut transaction: Transaction;

        let start = Instant::now();
        (transaction, utxo) = one_input_diff_output_transaction_valid(1000);
        let duration = start.elapsed();

        println!("Time elapsed is: {:?}", duration);

        assert_eq!(transaction.tx_outputs.len(), 1000);
    }

    #[test]
    fn test_one_input_tens_of_thousand_outputs() {
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut transaction: Transaction;

        let start = Instant::now();
        (transaction, utxo) = one_input_diff_output_transaction_valid(10000);
        let duration = start.elapsed();

        println!("Time elapsed is: {:?}", duration);

        assert_eq!(transaction.tx_outputs.len(), 10000);
    }

    #[test]
    fn test_one_input_hundreds_of_thousand_outputs() {
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut transaction: Transaction;

        let start = Instant::now();
        (transaction, utxo) = one_input_diff_output_transaction_valid(100000);
        let duration = start.elapsed();

        println!("Time elapsed is: {:?}", duration);

        assert_eq!(transaction.tx_outputs.len(), 100000);
    } */
    #[test]
    fn test_verif_one_input_diff_outputs() {
        let base: u32 = 10;

        let mut multiplicative_index: u32 = 0;
        for n in 0..10 {
            let mut utxo: UTXO = UTXO(HashMap::new());
            let mut transaction: Transaction;

            if ((base.pow(n.try_into().unwrap())) > 100000) {
                multiplicative_index = 100000 * (n - 4);
                (transaction, utxo) = one_input_diff_output_transaction_valid(multiplicative_index);
            } else {
                multiplicative_index = base.pow(n.try_into().unwrap());
                (transaction, utxo) =
                    one_input_diff_output_transaction_valid(base.pow(n.try_into().unwrap()));
            }

            let start = Instant::now();
            utxo.verify_transaction(&transaction);
            let duration = start.elapsed();

            println!(
                "Time elapsed for {:#} is: {:?}",
                multiplicative_index, duration
            );
        }
    }

    #[test]
    fn test_verif_diff_input_one_output() {
        let base: usize = 10;

        let mut multiplicative_index: usize = 0;
        for n in 0..10 {
            let mut utxo: UTXO = UTXO(HashMap::new());
            let mut transaction: Transaction;
            multiplicative_index = base.pow(n.try_into().unwrap());

            if ((base.pow(n.try_into().unwrap())) > 100000) {
                multiplicative_index = 100000 * (n - 4);
                (transaction, utxo) = diff_input_one_output_transaction_valid(multiplicative_index);
            } else {
                multiplicative_index = base.pow(n.try_into().unwrap());
                (transaction, utxo) =
                    diff_input_one_output_transaction_valid(base.pow(n.try_into().unwrap()));
            }

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
