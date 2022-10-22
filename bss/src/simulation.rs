use crate::block::Block;
use crate::transaction::Transaction;
use crate::utxo::UTXO;

use std::{collections::HashMap, sync::mpsc, thread};

static BLOCK_MEAN: f64 = 1.0;
static BLOCK_MULTIPLIER: u64 = 16;
pub static BLOCK_SIZE: u128 = 32;
static MAX_NUM_RECEIVERS: usize = 6;
static TRANSACTION_MEAN: f64 = 1.0;
static TRANSACTION_MULTIPLIER: u64 = 1;

#[allow(dead_code)] // To prevent warning for unused functions
pub fn start() {
    let mut utxo: UTXO = UTXO {
        map: HashMap::new(),
    };
    utxo.map.insert(String::from("a"), 50);
    utxo.map.insert(String::from("b"), 20);

    let (tx, rx) = mpsc::channel();
    let (ty, ry) = mpsc::channel();
    let utxo_copy = utxo.clone();
    let transaction_handle = thread::spawn(|| {
        Transaction::transaction_generator(
            MAX_NUM_RECEIVERS,
            TRANSACTION_MEAN,
            TRANSACTION_MULTIPLIER,
            tx,
            ry,
            utxo,
        );
    });

    let block_handle = thread::spawn(|| {
        Block::block_generator(rx, ty, utxo_copy, BLOCK_MEAN, BLOCK_MULTIPLIER);
    });
}

//Uncomment to run the simulation
// mod tests {
//     use crate::simulation::start;

//     #[test]
//     pub fn test_simulation() {
//         start();
//     }
// }
