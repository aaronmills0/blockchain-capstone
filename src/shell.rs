use crate::graph::create_block_graph;
use crate::save_and_load::deserialize_json;
use crate::simulation::start;
use crate::validate_blockchain::validate_chain_performance_test;

use chrono::Local;
use log::{info, warn};
use std::env;
use std::fs;
use std::io;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::{fs::File, path::Path};

static mut SIM_STATUS: bool = false;

pub fn shell() {
    let mut tx_sim_option: Option<Sender<String>> = None;
    loop {
        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        match command.to_lowercase().trim() {
            "help" => {
                info!("The user selected help");
                display_commands();
            }
            "sim start" => unsafe {
                if !SIM_STATUS {
                    let (tx_sim_temp, rx_sim) = mpsc::channel();
                    tx_sim_option = Some(tx_sim_temp);
                    let _sim_handle = thread::spawn(|| start(rx_sim));
                    SIM_STATUS = true;
                } else {
                    info!("\nSimulation has already begun!\n");
                }
            },
            "save" => unsafe {
                if SIM_STATUS && tx_sim_option.is_some() {
                    let tx_sim = tx_sim_option.unwrap();
                    if tx_sim.send(String::from("save")).is_err() {
                        warn!("Failed to send command to the simulation");
                    }
                    tx_sim_option = Some(tx_sim);
                } else {
                    warn!("Simulation has not started");
                }
            },
            "graph" => {
                info!("Please enter a file path");
                let mut filepath = String::new();

                io::stdin()
                    .read_line(&mut filepath)
                    .expect("Failed to read line");

                let f = filepath.trim();
                if !Path::new(f).exists() {
                    warn!("The filepath {} doesn't exist. Going back to shell", f);
                    continue;
                }

                let (initial_tx_outs, blockchain, _, _, _, _, _) = deserialize_json(f);
                create_block_graph(initial_tx_outs, blockchain);
            }
            "exit" | "Exit" | "EXIT" => {
                info!("The user selected exit");

                write_log();
                exit(0);
            }
            "test" => {
                info!("The user selected: validate blockchain test");
                validate_chain_performance_test("./config/blockchain_200.json");
            }
            _ => {
                warn!("Invalid Command");
            }
        }
    }
}

fn display_commands() {
    info!("--> help: Displays the availble commands");
    info!("--> sim start: Allows the user to begin the simple 3 node blockchain simulation");
    info!("--> save: Saves the configurations of the system to the config folder");
    info!("--> graph: Creates a dot file graph that visualizes the blockchain for a given config file");
    info!("--> exit: Exits the program with error code 0");
}

fn write_log() {
    //cwd, cwdFrom, cwdTo, cwdLog will allow us to access the path to the current directory
    let cwd = std::env::current_dir().unwrap();
    let cwd_from = std::env::current_dir().unwrap();
    let cwd_to = std::env::current_dir().unwrap();
    let cwd_log = std::env::current_dir().unwrap();

    //dirpath will allow us to access the path where we will store the new log file
    let mut dirpath = cwd.into_os_string().into_string().unwrap();
    //dirpathFrom will allow us to access the path of the orginal log file we copy from
    let mut dirpath_from = cwd_from.into_os_string().into_string().unwrap();
    //dirpathTo will allow us to access the path of the log file we copy into
    let mut dirpath_to = cwd_to.into_os_string().into_string().unwrap();
    //dirpathFrom will allow us to access the path of the orginal log file we copy from after we moved dirPathFrom
    let mut dirpath_log = cwd_log.into_os_string().into_string().unwrap();

    if env::consts::OS == "windows" {
        dirpath.push_str("/log");
        dirpath_from.push_str("\\log\\my.log");
        dirpath_to.push_str("\\log\\");
        dirpath_log.push_str("\\log\\my.log");
    } else {
        dirpath.push_str("/log");
        dirpath_from.push_str("/log/my.log");
        dirpath_to.push_str("/log/");
        dirpath_log.push_str("/log/my.log");
    }

    let dir_path = Path::new(&dirpath);
    let n1 = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    //The new log file contains the current time
    let filename1: &str = &format!("{}.log", n1);
    dirpath_to.push_str(filename1);
    let file_path = dir_path.join(filename1);
    if let Err(e) = File::create(file_path) {
        println!("{:?}", e)
    }
    if let Err(e) = fs::copy(dirpath_from, dirpath_to) {
        println!("{:?}", e)
    }
    //we remove the old log file
    File::create(&dirpath_log).unwrap();
}
