use crate::simulation::start;

use std::io;
use std::process;
use std::vec::Vec;
use std::fs;
static mut sim_status: bool = false;
use log::{info, trace, warn};
use log4rs;
use std::{fs::{File, create_dir}, path::Path};
use chrono::prelude::*;

pub fn interpreter(
    utxo: &mut HashMap<String, u128>,
    transaction_list: &mut Vec<Transaction>,
    blockchain: &mut Vec<Block>,
    ) -> bool{
use std::process::exit;

static mut SIM_STATUS: bool = false;

pub fn shell() -> bool {
    let mut command = String::new();

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");

    match command.trim() {
        "help" | "Help" | "HELP" => {
            display_commands();
            return true;
        }
        "sim start" | "Sim Start" | "Simulation Start | simulation start" | "SIM START" => unsafe {
            if !SIM_STATUS {
                start();
                SIM_STATUS = true;
                return true;
            } else {
                println!();
                println!("Simulation has already begun!");
                println!();
                return false;
            }
        },
        "exit" | "Exit" | "EXIT" => {
            info!("The user entered 'exit'");
            let cwd = std::env::current_dir().unwrap();
            let cwdFrom = std::env::current_dir().unwrap();
            let cwdTo = std::env::current_dir().unwrap();
            let cwdLog = std::env::current_dir().unwrap();
            let mut dirpath=cwd.into_os_string().into_string().unwrap();
            let mut dirpathFrom=cwdFrom.into_os_string().into_string().unwrap();
            let mut dirpathTo=cwdTo.into_os_string().into_string().unwrap();
            let mut dirpathLog=cwdLog.into_os_string().into_string().unwrap();

            dirpath.push_str("/log");
            dirpathFrom.push_str("\\log\\my.log");
            dirpathTo.push_str("\\log\\");
            dirpathLog.push_str("\\log\\my.log");
            
            
            let dir_path=Path::new(&dirpath);
            let n1=Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
            let filename1:&str=&format!("sam{}.log",n1);
            dirpathTo.push_str(filename1);
            let file_path=dir_path.join(filename1);
            let file=File::create(file_path);
            let copied= fs::copy(dirpathFrom, dirpathTo);
            let log_file = File::create(&dirpathLog).unwrap();
            
            exit_program();
            return true;
        }
        _ => {
            println!("Invalid Command");
            return false;
        }
    }
}

fn display_commands() {
    println!("--> help: Displays the availble commands");
    println!("--> sim start: Allows the user to begin the simple 3 node blockchain simulation");
    println!("--> exit: Exits the program with error code 0");
}

fn exit_program() {
    exit(0);
}
