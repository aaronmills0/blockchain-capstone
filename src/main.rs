mod components;
mod network;
mod performance_tests;
mod shell;
mod simulation;
mod utils;
use crate::{network::archive::Archive, shell::shell};
use log::info;
use std::env::{self};

static IS_ARCHIVE: bool = false;

#[tokio::main]
async fn main() {
    let mut cmd_archive = false;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "archive" {
        cmd_archive = true;
    }
    let cwd = std::env::current_dir().unwrap();
    let mut cwd_string = cwd.into_os_string().into_string().unwrap();
    let slash = if env::consts::OS == "windows" {
        "\\"
    } else {
        "/"
    };
    cwd_string.push_str(&(slash.to_owned() + "logging_config.yaml"));

    log4rs::init_file(cwd_string, Default::default()).unwrap();

    info!("Welcome to the minimalist blockchain!\n");
    info!("For list of supported commands enter: 'help'");

    if IS_ARCHIVE || cmd_archive {
        Archive::launch().await;
    } else {
        shell().await;
    }
}
