mod block;
mod hash;
mod merkle;
mod shell;
mod sign_and_verify;
mod simulation;
mod transaction;
mod utxo;
use shell::shell;

fn main() {
    println!("Welcome to the simple transaction chain!\n");

    println!("For list of supported commands, enter help");
    loop {
        shell();
    }
}
