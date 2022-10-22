mod block;
mod hash;
mod merkle;
mod shell;
mod signer_and_verifier;
mod simulation;
mod transaction;
mod utxo;
mod shell;
use shell::interpreter;
use log::{info, trace, warn};
use log4rs;

use std::collections::HashMap;
use std::vec::Vec;

const BLOCK_SIZE: u128 = 1;

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let mut cwd_string=cwd.into_os_string().into_string().unwrap();
    cwd_string.push_str("\\src\\logging_config.yaml");
    log4rs::init_file(cwd_string, Default::default()).unwrap();

    trace!("Welcome to the simple transaction chain!\n");
    let (mut blockchain, mut transaction_list, mut utxo) = initialize();

    println!("For list of supported commands, enter help");
    loop {
        if !shell() {
            continue;
        }
    }
}
