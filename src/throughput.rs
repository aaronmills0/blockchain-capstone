#[cfg(test)]
mod tests {
    use crate::block::Block;
    use crate::hash;
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::transaction::Transaction;
    use crate::transaction::{Outpoint, PublicKeyScript, SignatureScript, TxIn, TxOut};
    use crate::utxo::UTXO;
    use std::collections::HashMap;
    use std::time::Instant;

    fn create_valid_transactions(
        number_of_transactions: u32,
    ) -> (std::vec::Vec<Transaction>, UTXO) {
        //We first insert an unspent output in the utxo to which we will
        //refer later on.
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();
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
        utxo.insert(outpoint0.clone(), tx_out0.clone());

        //We create a signature script for the inputs of our new transaction
        let mut sig_script1: SignatureScript;

        let mut old_private_key0: PrivateKey;
        let mut old_public_key0: PublicKey;

        (old_private_key0, old_public_key0) = key_map[&outpoint0].clone();

        let mut message: String;

        message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key0),
            full_public_key: old_public_key0,
        };

        key_map.remove(&outpoint0);

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

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
            tx_inputs: Vec::from([tx_in1]),
            txin_count: 1,
            tx_outputs: Vec::from([tx_out1]),
            txout_count: 1,
        };

        let txid = hash::hash_as_string(&transaction1);

        transactions.push(transaction1);

        if (number_of_transactions > 1) {
            for n in 1..number_of_transactions {
                let outpoint0: Outpoint = Outpoint {
                    txid: txid.clone(),
                    index: 0,
                };

                key_map.insert(
                    outpoint0.clone(),
                    (private_key1.clone(), public_key1.clone()),
                );

                //We create a signature script for the inputs of our new transaction
                let mut sig_script1: SignatureScript;

                let mut old_private_key0: PrivateKey;
                let mut old_public_key0: PublicKey;

                (old_private_key0, old_public_key0) = key_map[&outpoint0].clone();

                let mut message: String;

                //We reconstruct the txout1 as it does not implement the copy trait
                let tx_out1: TxOut = TxOut {
                    value: 500,
                    pk_script: PublicKeyScript {
                        public_key_hash: hash::hash_as_string(&public_key1),
                        verifier: Verifier {},
                    },
                };

                message = String::from(&outpoint0.txid)
                    + &outpoint0.index.to_string()
                    + &tx_out1.pk_script.public_key_hash;

                sig_script1 = SignatureScript {
                    signature: sign_and_verify::sign(&message, &old_private_key0),
                    full_public_key: old_public_key0,
                };

                key_map.remove(&outpoint0);

                let tx_in1: TxIn = TxIn {
                    outpoint: outpoint0,
                    sig_script: sig_script1,
                };

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
                    tx_inputs: Vec::from([tx_in1]),
                    txin_count: 1,
                    tx_outputs: Vec::from([tx_out1]),
                    txout_count: 1,
                };

                let txid = hash::hash_as_string(&transaction1);
                transactions.push(transaction1);
            }
        }

        return (transactions, utxo);
    }

    #[test]
    fn test_transaction_throughput() {
        for k in 0..5 {
            let base: u32 = 10;
            let mut multiplicative_index: u32 = 0;

            for n in 0..10 {
                let mut utxo: UTXO = UTXO(HashMap::new());
                let mut transactions: Vec<Transaction>;
                multiplicative_index = base.pow(n.try_into().unwrap());

                if ((base.pow(n.try_into().unwrap())) > 100000) {
                    multiplicative_index = 100000 * (n - 4);
                    (transactions, utxo) = create_valid_transactions(multiplicative_index);
                } else {
                    multiplicative_index = base.pow(n.try_into().unwrap());
                    (transactions, utxo) = create_valid_transactions(multiplicative_index);
                }

                assert_eq!(transactions.len() as u32, multiplicative_index);

                let start = Instant::now();
                Block::verify_and_update(transactions, utxo);
                let duration = start.elapsed();

                println!(
                    "Time elapsed for {:#} in Run {:#} is: {:?}",
                    multiplicative_index, k, duration
                );

                println!();
            }
        }
    }
}
