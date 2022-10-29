use crate::simulation::start;

use chrono::prelude::*;
use log::{info, warn};
use log4rs;
use std::env;
use std::fs;
use std::io;
use std::process;
use std::process::exit;
use std::vec::Vec;
use std::{
    fs::{create_dir, File},
    path::Path,
};

static mut SIM_STATUS: bool = false;

pub fn shell() {
    let mut command = String::new();

    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");

    match command.trim() {
        "help" | "Help" | "HELP" => {
            info!("The user selected help");
            display_commands();
        }
        "sim start" | "Sim Start" | "SIM START" => unsafe {
            if !SIM_STATUS {
                start();
                SIM_STATUS = true;
            } else {
                info!("\nSimulation has already begun!\n");
            }
        },
        "exit" | "Exit" | "EXIT" => {
            info!("The user selected exit");
            //cwd, cwdFrom, cwdTo, cwdLog will allow us to access the path to the current directory
            let cwd = std::env::current_dir().unwrap();
            let cwdFrom = std::env::current_dir().unwrap();
            let cwdTo = std::env::current_dir().unwrap();
            let cwdLog = std::env::current_dir().unwrap();

            //dirpath will allow us to access the path where we will store the new log file
            let mut dirpath = cwd.into_os_string().into_string().unwrap();
            //dirpathFrom will allow us to access the path of the orginal log file we copy from
            let mut dirpathFrom = cwdFrom.into_os_string().into_string().unwrap();
            //dirpathTo will allow us to access the path of the log file we copy into
            let mut dirpathTo = cwdTo.into_os_string().into_string().unwrap();
            //dirpathFrom will allow us to access the path of the orginal log file we copy from after we moved dirPathFrom
            let mut dirpathLog = cwdLog.into_os_string().into_string().unwrap();

            if env::consts::OS == "windows" {
                dirpath.push_str("/log");
                dirpathFrom.push_str("\\log\\my.log");
                dirpathTo.push_str("\\log\\");
                dirpathLog.push_str("\\log\\my.log");
            } else {
                dirpath.push_str("/log");
                dirpathFrom.push_str("/log/my.log");
                dirpathTo.push_str("/log/");
                dirpathLog.push_str("/log/my.log");
            }

            let dir_path = Path::new(&dirpath);
            let n1 = Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
            //The new log file contains the current time
            let filename1: &str = &format!("{}.log", n1);
            dirpathTo.push_str(filename1);
            let file_path = dir_path.join(filename1);
            let file = File::create(file_path);
            let copied = fs::copy(dirpathFrom, dirpathTo);
            //we remove the old log file
            let log_file = File::create(&dirpathLog).unwrap();

            exit_program();
        }
        _ => {
            warn!("Invalid Command");
        }
    }
}

fn display_commands() {
    info!("--> help: Displays the availble commands");
    info!("--> sim start: Allows the user to begin the simple 3 node blockchain simulation");
    info!("--> exit: Exits the program with error code 0");
}

fn exit_program() {
    exit(0);
}
