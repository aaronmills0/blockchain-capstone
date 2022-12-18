use crate::hash::hash_as_string;
use crate::sign_and_verify::{PublicKey, Signature, Verifier};
use crate::transaction::{Outpoint, Transaction, TxIn, TxOut};
use ed25519_dalek::{PublicKey as DalekPublicKey, Signature as DalekSignature};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/**
 * The Utxo is a map containing the Unspent Transaction (X) Outputs.
 *
 * The key to this map comes from the Outpoint (see transactions.rs),
 * and is formed from the concatenation (or some other function that yields a unique key)
 * between the transaction identifier (txid) and the output number.
 *
 * The value (TxOut) is the unspent output containing the unspent value and
 * the public key script, which is used to verify the arguments (pushed onto the stack)
 * in the transaction input.
 */
#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct UTXO(#[serde_as(as = "Vec<(_, _)>")] pub HashMap<Outpoint, TxOut>);

impl Deref for UTXO {
    type Target = HashMap<Outpoint, TxOut>;
    fn deref(&self) -> &HashMap<Outpoint, TxOut> {
        return &self.0;
    }
}

impl DerefMut for UTXO {
    fn deref_mut(&mut self) -> &mut HashMap<Outpoint, TxOut> {
        return &mut self.0;
    }
}

impl UTXO {
    /**
     * Requirements for transaction verification
     * 1. Transaction must be unspent (i.e. no double spending and must exist in the utxo).
     * Check that its 'previous' output exists in the utxo and remove it from the utxo copy
     * (we would like to be able to revert)
     * 2. The new transaction outputs value (sum) cannot exceed the previous transaction outputs (sum)
     * 3. We must ensure that the transaction verifies to true.
     */
    pub fn verify_transaction(&self, transaction: &Transaction) -> bool {
        let mut utxo: UTXO = self.clone();
        // Note: If values are u32, then their sum can potentially overflow when summed. We should consider increasing the balances to u64
        let mut incoming_balance: u32 = 0;
        let mut outgoing_balance: u32 = 0;
        let mut tx_out: TxOut;
        let mut in_out_pairs: Vec<(TxIn, TxOut)> = Vec::new();
        for tx_in in transaction.tx_inputs.iter() {
            // If the uxto doesn't contain the output associated with this input: invalid transaction
            if !utxo.contains_key(&tx_in.outpoint) {
                warn!(
                    "Discarding invalid transaction! UTXO does not contain unspent outpoint: {:#?}",
                    &tx_in.outpoint
                );
                return false;
            }
            // Get the transaction output, add its value to the incoming balance
            // Store the TxIn, TxOut pair in in_out_pairs for verification later
            // Remove the output from the uxto copy.
            tx_out = utxo.get(&tx_in.outpoint).unwrap().clone();
            incoming_balance += tx_out.value;
            utxo.remove(&tx_in.outpoint);
            in_out_pairs.push((tx_in.clone(), tx_out));
        }
        // At this point, double spending and existance of unspent transaction output has been verified (1.)

        // Obtain the total amount that is requested to be transferred
        for new_tx_out in transaction.tx_outputs.iter() {
            outgoing_balance += new_tx_out.value;
        }

        // If we do not have the balance to fulfill this transaction, return false.
        if outgoing_balance > incoming_balance {
            warn!(
                "Discarding invalid transaction! The total available balance cannot support this transaction."
            );
            return false;
        }

        // At this point, incoming_balance being lesser than or equal to outgoing_balance has been verified (2.)
        let mut signature: &Signature;
        let mut public_key: &PublicKey;
        // message concatenates txid, output index of the previous transaction, old public key script, new public key script, and the value for the next recipient
        // For now, a message contains the txid, output index of the previous transaction, old public key hash
        let mut message: String;
        for (tx_in, tx_out) in in_out_pairs.iter() {
            signature = &tx_in.sig_script.signature;
            public_key = &tx_in.sig_script.full_public_key;
            message = String::from(&tx_in.outpoint.txid)
                + &tx_in.outpoint.index.to_string()
                + &tx_out.pk_script.public_key_hash;
            if !(tx_out
                .pk_script
                .verifier
                .verify(&message, signature, public_key))
            {
                warn!(
                    "Discarding invalid transaction! The transaction script could not be verified"
                );
                return false;
            }
        }

        return true;
    }

    pub fn batch_verify_and_update(&self, transactions: &Vec<Transaction>) -> (bool, Option<UTXO>) {
        let mut utxo: UTXO = self.clone();
        let mut incoming_balance: u32;
        let mut outgoing_balance: u32;
        let mut tx_out: TxOut;
        let mut in_out_pairs: Vec<(TxIn, TxOut)> = Vec::new();
        let mut msg_vec: Vec<Vec<u8>> = Vec::new();
        let mut sig_vec: Vec<DalekSignature> = Vec::new();
        let mut pk_vec: Vec<DalekPublicKey> = Vec::new();
        for transaction in transactions {
            incoming_balance = 0;
            outgoing_balance = 0;
            for tx_in in transaction.tx_inputs.iter() {
                // If the uxto doesn't contain the output associated with this input: invalid transaction
                if !utxo.contains_key(&tx_in.outpoint) {
                    warn!(
                        "Discarding invalid transaction! UTXO does not contain unspent outpoint: {:#?}",
                        &tx_in.outpoint
                    );
                    return (false, None);
                }
                // Get the transaction output, add its value to the incoming balance
                // Store the TxIn, TxOut pair in in_out_pairs for verification later
                // Remove the output from the uxto copy.
                tx_out = utxo.get(&tx_in.outpoint).unwrap().clone();
                incoming_balance += tx_out.value;

                sig_vec.push(tx_in.sig_script.signature.0);
                pk_vec.push(tx_in.sig_script.full_public_key.0);
                msg_vec.push(Vec::from(
                    (String::from(tx_in.outpoint.txid.clone())
                        + &tx_in.outpoint.index.to_string()
                        + &tx_out.pk_script.public_key_hash)
                        .as_bytes(),
                ));

                utxo.remove(&tx_in.outpoint);
                in_out_pairs.push((tx_in.clone(), tx_out));
            }
            for new_tx_out in transaction.tx_outputs.iter() {
                outgoing_balance += new_tx_out.value;
            }
            if outgoing_balance > incoming_balance {
                warn!(
                    "Discarding invalid transaction! The total available balance cannot support this transaction."
                );
                return (false, None);
            }
            // Update the utxo copy even though signature has not been checked yet
            utxo.update(&transaction);
        }

        let msg_bytes: Vec<&[u8]> = msg_vec.iter().map(|x| &x[..]).collect();

        let sig_status = Verifier::verify_batch(&msg_bytes, &sig_vec, &pk_vec);

        if sig_status {
            return (true, Some(utxo));
        } else {
            return (false, None);
        }
    }

    pub fn update(&mut self, transaction: &Transaction) {
        for tx_in in transaction.tx_inputs.iter() {
            self.remove(&tx_in.outpoint);
        }

        let txid: String = hash_as_string(transaction);
        // Iterate through the transfer quantity - receiver pairs
        for (i, tx_out) in transaction.tx_outputs.iter().enumerate() {
            let outpoint: Outpoint = Outpoint {
                txid: txid.clone(),
                index: (i as u32),
            };
            self.insert(outpoint, tx_out.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HashMap, Transaction, UTXO};
    use crate::hash;
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::transaction::{Outpoint, PublicKeyScript, SignatureScript, TxIn, TxOut};

    fn create_valid_transactions() -> (Transaction, UTXO) {
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

        let (private_key0_1, public_key0_1) = sign_and_verify::create_keypair();
        let outpoint0_1: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 1,
        };

        let tx_out0_1: TxOut = TxOut {
            value: 100,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0_1),
                verifier: Verifier {},
            },
        };

        let (private_key0_2, public_key0_2) = sign_and_verify::create_keypair();
        let outpoint0_2: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 2,
        };

        let tx_out0_2: TxOut = TxOut {
            value: 200,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key0_2),
                verifier: Verifier {},
            },
        };

        key_map.insert(outpoint0.clone(), (private_key0, public_key0));
        key_map.insert(outpoint0_1.clone(), (private_key0_1, public_key0_1));
        key_map.insert(outpoint0_2.clone(), (private_key0_2, public_key0_2));

        utxo.insert(outpoint0.clone(), tx_out0.clone());
        utxo.insert(outpoint0_1.clone(), tx_out0_1.clone());
        utxo.insert(outpoint0_2.clone(), tx_out0_2.clone());

        let old_private_key0: PrivateKey;
        let old_public_key0: PublicKey;

        let old_private_key0_1: PrivateKey;
        let old_public_key0_1: PublicKey;

        let old_private_key0_2: PrivateKey;
        let old_public_key0_2: PublicKey;

        (old_private_key0, old_public_key0) = key_map[&outpoint0].clone();
        (old_private_key0_1, old_public_key0_1) = key_map[&outpoint0_1].clone();
        (old_private_key0_2, old_public_key0_2) = key_map[&outpoint0_2].clone();

        let message = String::from(&outpoint0.txid)
            + &outpoint0.index.to_string()
            + &tx_out0.pk_script.public_key_hash;

        let sig_script1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key0, &old_public_key0),
            full_public_key: old_public_key0,
        };

        let tx_in1: TxIn = TxIn {
            outpoint: outpoint0,
            sig_script: sig_script1,
        };

        let message = String::from(&outpoint0_1.txid)
            + &outpoint0_1.index.to_string()
            + &tx_out0_1.pk_script.public_key_hash;

        let sig_script1_1 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key0_1, &old_public_key0_1),
            full_public_key: old_public_key0_1,
        };

        let tx_in1_1: TxIn = TxIn {
            outpoint: outpoint0_1,
            sig_script: sig_script1_1,
        };

        let message = String::from(&outpoint0_2.txid)
            + &outpoint0_2.index.to_string()
            + &tx_out0_2.pk_script.public_key_hash;

        let sig_script1_2 = SignatureScript {
            signature: sign_and_verify::sign(&message, &old_private_key0_2, &old_public_key0_2),
            full_public_key: old_public_key0_2,
        };

        let tx_in1_2: TxIn = TxIn {
            outpoint: outpoint0_2,
            sig_script: sig_script1_2,
        };

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (_private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1, tx_in1_1, tx_in1_2]),
            tx_outputs: Vec::from([tx_out1]),
        };

        return (transaction1, utxo);
    }

    fn create_invalid_transactions_insufficient_balance() -> (Transaction, UTXO) {
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

        let old_private_key: PrivateKey;
        let old_public_key: PublicKey;

        (old_private_key, old_public_key) = key_map[&outpoint0].clone();

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

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (_private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 700,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            tx_outputs: Vec::from([tx_out1]),
        };

        return (transaction1, utxo);
    }

    fn create_invalid_transactions_no_output_corresponding_to_input() -> (Transaction, UTXO) {
        //We do not include the unspent transaction in the utxo. That way, we cannot access the previous unspent output
        let utxo: UTXO = UTXO(HashMap::new());
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

        let old_private_key: PrivateKey;
        let old_public_key: PublicKey;

        (old_private_key, old_public_key) = key_map[&outpoint0].clone();

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

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (_private_key1, public_key1) = sign_and_verify::create_keypair();

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

        return (transaction1, utxo);
    }

    fn create_invalid_transactions_nomatch_signature() -> (Transaction, UTXO) {
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

        let old_private_key: PrivateKey;
        let old_public_key: PublicKey;

        (old_private_key, old_public_key) = sign_and_verify::create_keypair();

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

        //We create a new keypair corresponding to our new transaction which allows us to create its tx_out

        let (_private_key1, public_key1) = sign_and_verify::create_keypair();

        let tx_out1: TxOut = TxOut {
            value: 700,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key1),
                verifier: Verifier {},
            },
        };

        let transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            tx_outputs: Vec::from([tx_out1]),
        };

        return (transaction1, utxo);
    }

    #[test]
    fn test_utxo_verify_valid_transaction() {
        let (transaction, utxo) = create_valid_transactions();

        assert!(utxo.verify_transaction(&transaction));
    }

    #[test]
    fn test_utxo_verify_invalid_transaction_insufficient_balance() {
        let (transaction, utxo) = create_invalid_transactions_insufficient_balance();

        assert!(!(utxo.verify_transaction(&transaction)));
    }

    #[test]
    fn test_utxo_verify_invalid_transaction_nomatch_signature() {
        let (transaction, utxo) = create_invalid_transactions_nomatch_signature();

        assert!(!(utxo.verify_transaction(&transaction)));
    }

    #[test]
    fn test_utxo_verify_invalid_transaction_input_nomatch_output() {
        let (transaction, utxo) = create_invalid_transactions_no_output_corresponding_to_input();

        assert!(!(utxo.verify_transaction(&transaction)));
    }

    #[test]
    fn test_utxo_update() {
        let (transaction, mut utxo) = create_valid_transactions();

        let old_outpoint = Outpoint {
            txid: hash::hash_as_string(&transaction),
            index: (0),
        };

        utxo.update(&transaction);

        assert_eq!(utxo.get(&old_outpoint).unwrap().value, 500);
        assert_eq!(utxo.len(), 1);
    }
}
