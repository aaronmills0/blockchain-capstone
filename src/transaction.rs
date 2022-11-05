use crate::hash;
use crate::sign_and_verify;
use crate::sign_and_verify::{PrivateKey, PublicKey, Signature, Verifier};
use crate::simulation::KeyMap;
use crate::utxo::UTXO;

use log::{info, warn};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Distribution, Exp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{Receiver, Sender};
use std::vec::Vec;
use std::{thread, time};

#[derive(Clone, Serialize, Deserialize)]
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
        max_num_outputs: usize,
        mean_transaction_rate: f32,
        multiplier: u32,
        transmitter: Sender<Transaction>,
        receiver: Receiver<UTXO>,
        mut utxo: UTXO,
        mut key_map: KeyMap,
    ) {
        if max_num_outputs <= 0 {
            warn!("Invalid input. The max number of receivers must be larger than zero and no larger than {} but was {}", utxo.len(), max_num_outputs);
        }
        if mean_transaction_rate <= 0.0 {
            warn!("Invalid input. A non-positive mean for transaction rate is invalid for an exponential distribution but the mean was {}", mean_transaction_rate);
        }

        let lambda: f32 = 1.0 / mean_transaction_rate;
        let exp: Exp<f32> = Exp::new(lambda).unwrap();
        let mut rng: ThreadRng = rand::thread_rng();
        let mut sample: f32;
        let mut normalized: f32;
        let mut transaction_rate: time::Duration;
        let mut verified_utxo = utxo.clone();
        info!("Original UTXO:");
        for (key, value) in &utxo.0 {
            info!("{:#?}: {:#?}", key, value);
        }
        let mut transaction_counter = 0;
        loop {
            sample = exp.sample(&mut rng);
            // For an exponential distribution (with lambda > 0), the values range from (0, lambda].
            // Since mean = 1 / lambda, multiply the sample by the mean to normalize.
            normalized = sample * mean_transaction_rate;
            // Get the time between transactions generated as a duration
            transaction_rate = time::Duration::from_secs((multiplier * normalized as u32) as u64);
            // Sleep to mimic the time between creation of transactions
            thread::sleep(transaction_rate);

            let transaction =
                Self::create_rng_transaction(&utxo, &mut key_map, &mut rng, max_num_outputs);
            transaction_counter += 1;
            info!("{} Transactions Created", transaction_counter);
            utxo.update(&transaction);
            info!("Updated UTXO");
            for (key, value) in &utxo.0 {
                info!("{:#?}: {:#?}", key, value);
            }
            transmitter.send(transaction).unwrap();

            let new_utxo = receiver.try_recv();
            if new_utxo.is_ok() {
                verified_utxo = new_utxo.unwrap();
            }
        }
    }

    fn create_rng_transaction(
        utxo: &UTXO,
        key_map: &mut HashMap<Outpoint, (PrivateKey, PublicKey)>,
        rng: &mut ThreadRng,
        max_num_outputs: usize,
    ) -> Transaction {
        let mut unspent_txos: Vec<Outpoint> = Vec::new();
        for (utxo_key, _) in utxo.iter() {
            unspent_txos.push(utxo_key.clone());
        }

        let num_inputs: usize = rng.gen_range(1..=utxo.len());
        let num_outputs: usize = rng.gen_range(1..=max_num_outputs);

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

        let mut tx_inputs: Vec<TxIn> = Vec::new();
        let mut tx_outputs: Vec<TxOut> = Vec::new();
        let mut outpoint: Outpoint;
        let mut sig_script: SignatureScript;
        let mut new_private_key: PrivateKey;
        let mut new_public_key: PublicKey;
        let mut old_private_key: PrivateKey;
        let mut old_public_key: PublicKey;
        let mut message: String;
        let mut pk_script: PublicKeyScript;
        for i in 0..num_inputs {
            outpoint = utxo_keys[i].clone();

            (old_private_key, old_public_key) = key_map[&outpoint].clone();

            message = String::from(&outpoint.txid)
                + &outpoint.index.to_string()
                + &utxo[&outpoint].pk_script.public_key_hash;

            sig_script = SignatureScript {
                signature: sign_and_verify::sign(&message, &old_private_key),
                full_public_key: old_public_key,
            };

            key_map.remove(&outpoint); // Remove the old key pair

            tx_inputs.push(TxIn {
                outpoint: outpoint,
                sig_script: sig_script,
            });
        }

        let mut key_vec: Vec<(PrivateKey, PublicKey)> = Vec::new();
        for j in 0..num_outputs {
            (new_private_key, new_public_key) = sign_and_verify::create_keypair();

            pk_script = PublicKeyScript {
                public_key_hash: hash::hash_as_string(&new_public_key),
                verifier: Verifier {},
            };

            key_vec.push((new_private_key, new_public_key));

            tx_outputs.push(TxOut {
                value: output_values[j],
                pk_script: pk_script,
            });
        }
        info!(
            "Transaction created with {} inputs and {} outputs.",
            num_inputs, num_outputs
        );
        let transaction = Transaction {
            tx_inputs: tx_inputs,
            tx_outputs: tx_outputs,
        };

        let txid: String = hash::hash_as_string(&transaction);

        // Update the key_map

        for k in 0..num_outputs {
            outpoint = Outpoint {
                txid: txid.clone(),
                index: k as u32,
            };
            key_map.insert(outpoint, key_vec[k].clone());
        }

        return transaction;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TxIn {
    pub outpoint: Outpoint,
    pub sig_script: SignatureScript,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SignatureScript {
    pub signature: Signature,
    pub full_public_key: PublicKey,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Debug)]
pub struct Outpoint {
    pub txid: String,
    pub index: u32,
}

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
