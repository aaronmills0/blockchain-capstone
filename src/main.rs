mod block;
mod hash;
mod merkle;
mod shell;
mod sign_and_verify;
mod simulation;
mod transaction;
mod utxo;
use log::info;
use log4rs;
use shell::shell;
use std::env;

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let mut cwd_string = cwd.into_os_string().into_string().unwrap();
    if env::consts::OS == "windows" {
        cwd_string.push_str("\\src\\logging_config.yaml");
    } else {
        cwd_string.push_str("/src/logging_config.yaml");
    }
    log4rs::init_file(cwd_string, Default::default()).unwrap();

    info!("Welcome to the minimalist blockchain!\n");

    info!("For list of supported commands enter: 'help'");
    loop {
        shell();
    }
}
