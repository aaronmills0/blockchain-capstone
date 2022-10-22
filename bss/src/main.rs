mod block;
mod hash;
mod merkle;
mod shell;
mod sign_and_verify;
mod simulation;
mod transaction;
mod utxo;
use shell::shell;
use log::{info, trace, warn};
use log4rs;

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let mut cwd_string = cwd.into_os_string().into_string().unwrap();
    cwd_string.push_str("\\src\\logging_config.yaml");
    log4rs::init_file(cwd_string, Default::default()).unwrap();

    trace!("Welcome to the simple transaction chain!\n");

    println!("For list of supported commands, enter help");
    loop {
        if !shell() {
            continue;
        }
    }
}
