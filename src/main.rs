mod components;
mod network;
mod performance_tests;
mod shell;
mod simulation;
mod utils;
use log::info;
use network::peer;
use shell::shell;
use std::{env, thread};

#[tokio::main]
async fn main() {
    let cwd = std::env::current_dir().unwrap();
    let mut cwd_string = cwd.into_os_string().into_string().unwrap();
    if env::consts::OS == "windows" {
        cwd_string.push_str("\\logging_config.yaml");
    } else {
        cwd_string.push_str("/logging_config.yaml");
    }
    log4rs::init_file(cwd_string, Default::default()).unwrap();

    info!("Welcome to the minimalist blockchain!\n");
    info!("For list of supported commands enter: 'help'");

    // peer::spawn_listener().await;

    peer::spawn_connection("10.121.226.81:6780").await;
    //shell();
}
