#[cfg(test)]
mod tests {
    use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut, TxIn, SignatureScript};
    use crate::components::utxo::UTXO;
    use crate::network::messages;
    use crate::simulation::KeyMap;
    use crate::utils::sign_and_verify::Verifier;
    use crate::utils::{hash, sign_and_verify};
    use crate::network::peer::{self, Peer};
    use rand_1::rngs::ThreadRng;
    use std::collections::HashMap;
    use std::time::Instant;

    async fn test_single_peer_tx_throughput_sender(peer: &Peer) {
        let delta_t: u128 = 1000; //Number in micro seconds
        let mut entry: bool = true;
        let mut timer = Instant::now();
        loop{
            if timer.elapsed().as_micros() >= delta_t{
                let peerid = peer.peerid;
                let frame =
                    messages::get_transaction_msg(peerid, peerid, get_example_transaction());
                peer::send_transaction(frame, peer.address.to_owned(), peer.ports.to_owned()).await;
            }
        }
        
    }

    fn test_single_peer_tx_throughput_receiver(){

    }

    fn get_example_transaction() -> Transaction {
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
    
        let (old_private_key, old_public_key) = (private_key0, public_key0);
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
    
        return transaction1;
    }


    #[ignore]
    #[test]
    fn run_sender_test_transaction_throughput() {
        let peer = Peer::new();
        test_single_peer_tx_throughput_sender(peer)
    }

    #[test]
    fn run_receiver_test_transaction_throughput(){
        test_single_peer_tx_throughput_receiver()
    }
}
