use crate::hash::hash_as_string;
use crate::sign_and_verify::{PublicKey, Signature};
use crate::transaction::{Outpoint, Transaction, TxIn, TxOut};
use log::warn;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/**
 * The UTXO is a map containing the Unspent Transaction (X) Outputs.
 *
 * The key to this map comes from the Outpoint (see transactions.rs),
 * and is formed from the concatenation (or some other function that yields a unique key)
 * between the transaction identifier (txid) and the output number.
 *
 * The value (TxOut) is the unspent output containing the unspent value and
 * the public key script, which is used to verify the arguments (pushed onto the stack)
 * in the transaction input.
 */
#[derive(Clone)]
pub struct UTXO(pub HashMap<Outpoint, TxOut>);

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
                    "Invalid transaction! UTXO does not contain unspent output. {:#?}",
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
                "Invalid transaction! The total available balance cannot support this transaction."
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
                .verify(&message, &signature, &public_key))
            {
                return false;
            }
        }

        return true;
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
