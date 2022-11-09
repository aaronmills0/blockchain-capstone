#[cfg(test)]
mod tests {
    use super::{HashMap, Transaction, UTXO};
    use crate::hash;
    use crate::sign_and_verify;
    use crate::sign_and_verify::{PrivateKey, PublicKey, Verifier};
    use crate::transaction::{Outpoint, PublicKeyScript, SignatureScript, TxIn, TxOut};

    fn one_input_diff_output_transaction_valid(number_of_outputs: u64) {
        //We first insert an unspent output in the utxo to which we will
        //refer later on.
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut key_map: HashMap<Outpoint, (PrivateKey, PublicKey)> = HashMap::new();

        let value_to_be_spent = 10 * number_of_outputs;

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

        let count = 1;

        let tx_outs: Vec<TxIn> = Vec::new();

        loop {
            tx_outs.push(TxOut {
                value: value_to_be_spent / 10,
                pk_script: PublicKeyScript {
                    public_key_hash: hash::hash_as_string(&public_key1),
                    verifier: Verifier {},
                },
            });
        }

        let mut transaction1: Transaction = Transaction {
            tx_inputs: Vec::from([tx_in1]),
            txin_count: 1,
            tx_outputs: tx_outs,
            txout_count: 1,
        };

        return (transaction1, utxo);
    }

    #[test]
    fn one_input_several_outputs() {
        let mut utxo: UTXO = UTXO(HashMap::new());
        let mut transaction: Transaction;

        (transaction, utxo) = one_input_diff_output_transaction_valid(2);

        assert_eq!(transaction.tx_outputs.len(), 2);
    }
}
