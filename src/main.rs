mod components;
mod network;
mod performance_tests;
mod shell;
mod simulation;
mod utils;
use log::info;
use network::client;
use network::server;
use shell::shell;
use std::env;

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

    //client::setup_client().await;
    server::setup_server().await;
    //shell();
}
