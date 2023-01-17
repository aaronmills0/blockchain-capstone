#[cfg(test)]
mod tests {
    use crate::components::block::{Block, BlockHeader};
    use crate::components::merkle::Merkle;
    use crate::components::transaction::{Outpoint, PublicKeyScript, Transaction, TxOut};
    use crate::components::utxo::UTXO;
    use crate::simulation::KeyMap;
    use crate::utils::sign_and_verify::Verifier;
    use crate::utils::validator::fork_exists;
    use crate::utils::{hash, sign_and_verify};
    use rand_1::rngs::ThreadRng;
    use std::collections::HashMap;
    use std::time::Instant;

    #[ignore]
    #[test]
    fn test_block_validation_time() {
        let base: u32 = 10;
        let mut multiplicative_index: u32;
        for k in 0..7 {
            multiplicative_index = base.pow(k.try_into().unwrap());
            let mut utxo: UTXO = UTXO(HashMap::new());
            let mut key_map: KeyMap = KeyMap(HashMap::new());
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
            utxo.insert(outpoint0, tx_out0);

            let utxo_copy = utxo.clone();
            let mut rng: ThreadRng = rand_1::thread_rng();
            let max_num_outputs = 1;
            for _ in 0..multiplicative_index {
                let transaction = Transaction::create_transaction(
                    &utxo,
                    &mut key_map,
                    &mut rng,
                    max_num_outputs,
                    false,
                );
                utxo.update(&transaction);
                transactions.push(transaction);
            }

            assert_eq!(transactions.len() as u32, multiplicative_index);

            let genesis_merkle: Merkle = Merkle {
                tree: Vec::from(["0".repeat(64)]),
            };

            let genesis_block: Block = Block {
                header: BlockHeader {
                    previous_hash: "0".repeat(64),
                    merkle_root: genesis_merkle.tree.first().unwrap().clone(),
                    nonce: 0,
                },
                merkle: genesis_merkle,
                transactions: Vec::new(),
            };

            let merkle = Merkle::create_merkle_tree(&transactions);
            let block: Block = Block {
                header: BlockHeader {
                    previous_hash: hash::hash_as_string(&genesis_block.header),
                    merkle_root: merkle.tree.first().unwrap().clone(),
                    nonce: 0,
                },
                merkle,
                transactions,
            };

            let chain: Vec<Block> = vec![genesis_block];

            println!("{} transactions: \n", multiplicative_index);
            println!("\n");
            for r in 0..5 {
                let block_copy = block.clone();
                let utxo_copy2 = utxo_copy.clone();
                let chain_copy = chain.clone();
                print!("{}: ", r);
                assert!(validate_one_block(block_copy, utxo_copy2, chain_copy));
                println!();
            }
        }
    }

    // This helper function validates one block and times the three checks involved as well as the total time
    fn validate_one_block(incoming_block: Block, utxo: UTXO, chain: Vec<Block>) -> bool {
        let full_time = Instant::now();

        let fork_time = Instant::now();
        if fork_exists(&incoming_block, &chain) {
            println!("Received a block that doesn't branch off the head of the chain");
            return false;
        }
        let fork_time_elapsed = fork_time.elapsed().as_micros();
        print!("{}, ", fork_time_elapsed);

        let merkle_time = Instant::now();
        let merkle_tree = Merkle::create_merkle_tree(&incoming_block.transactions);
        if merkle_tree.tree.first().unwrap() != &incoming_block.header.merkle_root {
            println!("Received a block with an invalid merkle root");
            return false;
        }
        let merkle_time_elapsed = merkle_time.elapsed().as_micros();
        print!("{}, ", merkle_time_elapsed);

        let tx_time = Instant::now();
        let (valid, _) = utxo.batch_verify_and_update(&incoming_block.transactions);
        if !valid {
            print!("Received a block containing invalid transactions");
            return false;
        }
        let tx_time_elapsed = tx_time.elapsed().as_micros();
        print!("{}, ", tx_time_elapsed);

        let full_time_elapsed = full_time.elapsed().as_micros();
        print!("{}", full_time_elapsed);

        return true;
    }
}
