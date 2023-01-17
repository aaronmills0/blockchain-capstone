use crate::components::utxo::UTXO;
use crate::simulation::KeyMap;
use crate::utils::hash;
use crate::utils::sign_and_verify;
use crate::utils::sign_and_verify::{PrivateKey, PublicKey, Signature, Verifier};
use log::{info, warn};
use rand_1::rngs::ThreadRng;
use rand_1::seq::SliceRandom;
use rand_1::Rng;
use rand_distr::{Distribution, Exp};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;
use std::vec::Vec;
use std::{thread, time};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub tx_inputs: Vec<TxIn>,
    pub tx_outputs: Vec<TxOut>,
}

/**
 * Steps in creating a new transaction.
 * The receiving wallet must have a private-public key pair generated before the transaction can take place
 * The public key is then cryptographically hashed and provided to the spending wallet.
 * The spending wallet creates a new transactions with input(s) and output(s), and broadcasts it.
 * For the receiver to spend this transaction (with a new transaction),
 * they must create a transaction with input(s) that refers to output(s) by its transaction identifier (txid) and output number.
 * He then creates the SignatureScript that satisfies the PubKeyScript made by the original spender.
 * The signature script contains the following: Full Public Key, Signature that combines certain transaction data with the private key of the original receiver.
 * The transaction data that is signed to form the signature includes the txid and output index of the previous transaction, the previous outputs public key script, the new public key script, and the value for the next recipient
 */

impl Transaction {
    /**
     * Creates transactions at random times that follow an exponential distribution given by a specified mean
     * The transactions will be sent and received by existing addresses in the utxo
     * The amount of senders and receivers is also random (uniform), but a maximum is specified
     * The amount received for each address is random (uniform) but is normalized so that the total amount received
     * is (approximately) equal to the total balance of the senders
     *
     * The transaction list created is constantly transmitted so that the block generator can receive it
     */
    pub fn transaction_generator(
        transmitter: Sender<(Transaction, KeyMap)>,
        max_num_outputs: usize,
        transaction_mean: f32,
        transaction_duration: u32,
        mean_invalid_ratio: u32,
        mut utxo: UTXO,
        mut key_map: KeyMap,
    ) {
        if transaction_mean <= 0.0 {
            warn!("Invalid input. A non-positive mean for transaction rate is invalid for an exponential distribution but the mean was {}", transaction_mean);
        }

        let lambda: f32 = 1.0 / transaction_mean;
        let exp: Exp<f32> = Exp::new(lambda).unwrap();

        let mut invalid: bool;
        let mut normalized: f32;
        let mut rng: ThreadRng = rand_1::thread_rng();
        let mut sample: f32;
        let mut transaction_counter = 0;
        let mut transaction_rate: time::Duration;
        loop {
            // Invalid transactions happen with a probability of 1 / mean_invalid_ratio
            // There is no chance of invalid transactions if mean_invalid ratio is set to 0
            if mean_invalid_ratio == 0 {
                invalid = false;
            } else {
                invalid = (rng.gen_range(1..=mean_invalid_ratio)) == 1;
            }

            sample = exp.sample(&mut rng);
            // For an exponential distribution (with lambda > 0), the values range from (0, lambda].
            // Since mean = 1 / lambda, multiply the sample by the mean to normalize.
            normalized = sample * transaction_mean;
            // Get the time between transactions generated as a duration
            transaction_rate =
                time::Duration::from_secs((transaction_duration * normalized as u32) as u64);
            thread::sleep(transaction_rate); // Sleep to mimic the time between creation of transactions

            let transaction =
                Self::create_transaction(&utxo, &mut key_map, &mut rng, max_num_outputs, invalid);
            transaction_counter += 1;
            info!("{} Transactions Created", transaction_counter);

            if !invalid {
                utxo.update(&transaction);
            }
            transmitter.send((transaction, key_map.clone())).unwrap();
        }
    }

    pub fn create_transaction(
        utxo: &UTXO,
        key_map: &mut KeyMap,
        rng: &mut ThreadRng,
        max_num_outputs: usize,
        invalid: bool,
    ) -> Transaction {
        // Set a random invalid flag to true if this is intended to be an invalid transaction
        let mut invalid_input: bool = false;
        let mut invalid_sum: bool = false;
        let mut invalid_verification: bool = false;
        if invalid {
            match rng.gen_range(1..=3) {
                1 => {
                    invalid_input = true;
                    warn!("Expecting an invalid transaction since an input is not in the utxo");
                }
                2 => {
                    invalid_sum = true;
                    warn!("Expecting an invalid transaction since the total output balance is larger than the total input balance");
                }
                _ => {
                    invalid_verification = true;
                    warn!("Expecting an invalid transaction since the verification script fails");
                }
            }
        }

        let mut unspent_txos: Vec<Outpoint> = Vec::new();
        for (utxo_key, _) in utxo.iter() {
            unspent_txos.push(utxo_key.clone());
        }

        let num_inputs: usize = rng.gen_range(1..=utxo.len());
        let mut num_outputs: usize = rng.gen_range(1..=max_num_outputs);

        let utxo_keys: Vec<Outpoint> = unspent_txos
            .choose_multiple(rng, num_inputs)
            .cloned()
            .collect();

        let mut available_balance: u32 = 0;
        for key in &utxo_keys {
            available_balance += utxo.get(key).unwrap().value;
        }

        let mut output_values: Vec<u32> = Vec::new();
        let mut output_values_sum: u32 = 0;
        let mut total_generated_value: u32 = 0;
        for _ in 0..num_outputs {
            let generated_value = rng.gen_range(1..=100);
            total_generated_value += generated_value;
            output_values.push(generated_value);
        }
        let fraction = available_balance / total_generated_value;
        for value in output_values.iter_mut() {
            *value *= fraction;
            output_values_sum += *value;
        }
        output_values[0] += available_balance - output_values_sum;

        // If the invalid sum flag is set, ensure the output balance is greater than the input
        if invalid_sum {
            output_values[0] += 1
        }

        let invalid_index: usize = rng.gen_range(0..num_inputs);
        let mut message: String;
        let mut new_private_key: PrivateKey;
        let mut new_public_key: PublicKey;
        let mut old_private_key: PrivateKey;
        let mut old_public_key: PublicKey;
        let mut outpoint: Outpoint;
        let mut pk_script: PublicKeyScript;
        let mut sig_script: SignatureScript;
        let mut tx_inputs: Vec<TxIn> = Vec::new();
        let mut tx_outputs: Vec<TxOut> = Vec::new();
        for (i, utxo_key) in utxo_keys.iter().enumerate().take(num_inputs) {
            outpoint = utxo_key.clone();
            (old_private_key, old_public_key) = key_map[&outpoint].clone();
            message = String::from(&outpoint.txid)
                + &outpoint.index.to_string()
                + &utxo[&outpoint].pk_script.public_key_hash;

            // Set the public key as the old public key unless the invalid verification flag is set
            // and this is the chosen index for ensuring an invalid signature script
            let mut public_key = old_public_key;
            if invalid_verification && i == invalid_index {
                let (_, bad_public_key) = sign_and_verify::create_keypair();
                public_key = bad_public_key;
            }

            sig_script = SignatureScript {
                signature: sign_and_verify::sign(&message, &old_private_key, &public_key),
                full_public_key: public_key,
            };

            if !invalid {
                key_map.remove(&outpoint); // Remove the old key pair
            }

            // If the invalid input flag is set and this is the chosen invalid input,
            // Rehash the txid so that the output is faulty
            if invalid_input && i == invalid_index {
                outpoint.txid = hash::hash_as_string(&outpoint.txid);
            }

            tx_inputs.push(TxIn {
                outpoint,
                sig_script,
            });
        }

        let mut key_vec: Vec<(PrivateKey, PublicKey)> = Vec::new();
        for output_value in output_values.iter() {
            if *output_value == 0 {
                num_outputs -= 1;
                continue;
            }

            (new_private_key, new_public_key) = sign_and_verify::create_keypair();
            pk_script = PublicKeyScript {
                public_key_hash: hash::hash_as_string(&new_public_key),
                verifier: Verifier {},
            };

            key_vec.push((new_private_key, new_public_key));
            tx_outputs.push(TxOut {
                value: *output_value,
                pk_script,
            });
        }

        info!(
            "Transaction created with {} inputs and {} outputs.",
            num_inputs, num_outputs
        );

        let transaction = Transaction {
            tx_inputs,
            tx_outputs,
        };

        // Update the key_map but only if the transaction is valid
        if !invalid {
            let txid: String = hash::hash_as_string(&transaction);
            for (k, key) in key_vec.iter().enumerate() {
                outpoint = Outpoint {
                    txid: txid.clone(),
                    index: k as u32,
                };
                key_map.insert(outpoint, key.clone());
            }
        }

        return transaction;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TxIn {
    pub outpoint: Outpoint,
    pub sig_script: SignatureScript,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SignatureScript {
    pub signature: Signature,
    pub full_public_key: PublicKey,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub struct Outpoint {
    pub txid: String,
    pub index: u32,
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Outpoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.txid.hash(state);
        self.index.hash(state);
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TxOut {
    pub value: u32,
    pub pk_script: PublicKeyScript,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PublicKeyScript {
    pub public_key_hash: String,
    pub verifier: Verifier,
}

#[cfg(test)]
mod tests {
    use super::hash;
    use crate::components::transaction::{
        Outpoint, PublicKeyScript, SignatureScript, Transaction, TxIn, TxOut,
    };
    use crate::components::utxo::UTXO;
    use crate::utils::sign_and_verify;
    use crate::utils::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use std::collections::HashMap;

    static MAX_NUM_OUTPUTS: usize = 3;

    #[test]
    fn test_create_transaction_valid() {
        // We first insert an unspent output in the utxo to which we will
        // refer later on.
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
        let tx_out1: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            tx_outputs: Vec::from([tx_out1]),
        };

        assert_eq!(transaction1.tx_inputs.len(), 1);
        assert_eq!(transaction1.tx_outputs.len(), 1);
        assert_eq!(transaction1.tx_outputs.get(0).unwrap().value, 500);
        assert!(transaction1.tx_outputs.len() <= utxo.len());
        assert!(transaction1.tx_outputs.len() <= MAX_NUM_OUTPUTS);
    }
}
