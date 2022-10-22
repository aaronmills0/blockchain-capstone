use crate::simulation::start;

use std::io;
use std::process::exit;

static mut SIM_STATUS: bool = false;

pub fn interpreter() -> bool {
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
            exit_program();
            return false;
        }
        _ => return false,
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
