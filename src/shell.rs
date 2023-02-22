use crate::components::transaction::{
    Outpoint, PublicKeyScript, SignatureScript, Transaction, TxIn, TxOut,
};
use crate::network::messages;
use crate::network::peer::{self, Command, Peer};
use crate::utils::hash;
use crate::utils::save_and_load::deserialize_json;
use crate::utils::sign_and_verify::{self, Verifier};
use crate::utils::sign_and_verify::{PrivateKey, PublicKey};
use crate::wallet::Wallet;
use chrono::Local;
use local_ip_address::local_ip;
use log::{error, info, warn};
use port_scanner::scan_port;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::{env, fs, io};
use tokio::sync::oneshot;

static mut SIM_STATUS: bool = false;

pub async fn shell() {
    let mut tx_sim_option: Option<Sender<String>> = None;
    let tx_to_manager = Peer::launch().await;
    info!("Successfully launched peer!");

    loop {
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        match command.to_lowercase().trim() {
            // "help" => {
            //     info!("The user selected help");
            //     display_commands();
            // }

            // "sim start" => unsafe {
            //     if !SIM_STATUS {
            //         let (tx_sim_temp, rx_sim) = mpsc::channel();
            //         tx_sim_option = Some(tx_sim_temp);
            //         let _sim_handle = thread::spawn(|| start(rx_sim));
            //         SIM_STATUS = true;
            //     } else {
            //         info!("\nSimulation has already begun!\n");
            //     }
            // },

            // "save" => unsafe {
            //     if SIM_STATUS && tx_sim_option.is_some() {
            //         let tx_sim = tx_sim_option.unwrap();
            //         if tx_sim.send(String::from("save")).is_err() {
            //             warn!("Failed to send command to the simulation");
            //         }
            //         tx_sim_option = Some(tx_sim);
            //     } else {
            //         warn!("Simulation has not started");
            //     }
            // },

            // "graph" => {
            //     info!("Please enter a file path");
            //     let mut filepath = String::new();
            //     io::stdin()
            //         .read_line(&mut filepath)
            //         .expect("Failed to read line");

            //     let f = filepath.trim();
            //     if !Path::new(f).exists() {
            //         warn!("The filepath {} doesn't exist. Going back to shell", f);
            //         continue;
            //     }

            //     let (blockchain, _, initial_tx_outs, _, _, _, _) = deserialize_json(f);
            //     create_block_graph(initial_tx_outs, blockchain);
            // }
            "neighbors" | "neighbours" | "-n" => {
                info!("Neighbors:");
                let (resp_tx, resp_rx) = oneshot::channel();
                let cmd = Command::Get {
                    key: String::from("ip_map_query"),
                    resp: resp_tx,
                };
                tx_to_manager.send(cmd).await.ok();

                let result = resp_rx.await.unwrap().unwrap();
                if result.is_empty() {
                    error!("Empty result from peer");
                    panic!();
                }
                let ip_map: HashMap<u32, String> = serde_json::from_str(&result[0]).unwrap();

                for (id, ip) in ip_map {
                    info!("{} : {}", id, ip);
                }
            }
            "transaction" | "tx" | "-t" => {
                let mut transaction = Transaction {
                    tx_inputs: Vec::from([]),
                    tx_outputs: Vec::from([]),
                };

                // This is a test for loading the transaction and broadcatsing it. This block of code creates transaction.json
                let example_transaction = get_example_transaction();

                let mut map = Map::new();
                let server_json = serde_json::to_value(example_transaction);

                if server_json.is_err() {
                    error!("Failed to serialize server peer");
                    panic!();
                }

                let mut json = server_json.unwrap();

                map.insert(String::from("transaction"), json);

                json = serde_json::Value::Object(map);

                let slash = if env::consts::OS == "windows" {
                    "\\"
                } else {
                    "/"
                };
                if fs::create_dir_all("system".to_owned() + slash).is_err() {
                    warn!("Failed to create directory! It may already exist, or permissions are needed.");
                }

                let cwd = std::env::current_dir().unwrap();
                let mut dirpath = cwd.into_os_string().into_string().unwrap();
                dirpath.push_str(&(slash.to_owned() + "system"));

                let dir_path = Path::new(&dirpath);

                let file_name: &str = &format!("transaction.json");

                let file_path = dir_path.join(file_name);
                let file = File::create(file_path);
                if file.is_err() {
                    error!("Failed to create new file.");
                    panic!();
                }
                if file
                    .unwrap()
                    .write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
                    .is_err()
                {
                    error!("Failed to write to file.");
                    panic!();
                }

                // This is a test for loading the transaction and broadcatsing it. It creates wallet.json
                let (private_key_initial, public_key_initial) = sign_and_verify::create_keypair();
                let values = vec![(
                    private_key_initial,
                    public_key_initial,
                    Outpoint {
                        txid: "0".repeat(64),
                        index: 0,
                    },
                    500,
                )];
                let wallet: Wallet = Wallet(values);

                let mut map = Map::new();
                let server_json = serde_json::to_value(wallet);

                if server_json.is_err() {
                    error!("Failed to serialize server peer");
                    panic!();
                }

                let mut json = server_json.unwrap();

                map.insert(String::from("wallet"), json);

                json = serde_json::Value::Object(map);

                let slash = if env::consts::OS == "windows" {
                    "\\"
                } else {
                    "/"
                };
                if fs::create_dir_all("system".to_owned() + slash).is_err() {
                    warn!("Failed to create directory! It may already exist, or permissions are needed.");
                }

                let cwd = std::env::current_dir().unwrap();
                let mut dirpath = cwd.into_os_string().into_string().unwrap();
                dirpath.push_str(&(slash.to_owned() + "system"));

                let dir_path = Path::new(&dirpath);

                let file_name: &str = &format!("wallet.json");

                let file_path = dir_path.join(file_name);
                let file = File::create(file_path);
                if file.is_err() {
                    error!("Failed to create new file.");
                    panic!();
                }
                if file
                    .unwrap()
                    .write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
                    .is_err()
                {
                    error!("Failed to write to file.");
                    panic!();
                }

                // start of the transaction creator
                info!("Would you like to load a transaction from a file (f) or create it manually (m)?");
                let mut choice = String::new();
                io::stdin()
                    .read_line(&mut choice)
                    .expect("Failed to read line");

                match choice.to_lowercase().trim() {
                    // In this case, we simply need to be provided with a transaction.json file that we deserialize and directly send the transaction
                    "f" => {
                        let slash = if env::consts::OS == "windows" {
                            "\\"
                        } else {
                            "/"
                        };
                        let data =
                            fs::read_to_string("system".to_owned() + slash + "transaction.json");
                        if data.is_err() {
                            error!("Failed to load file. {:?}", data.err());
                            panic!();
                        }
                        let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
                        transaction =
                            serde_json::from_value(json.get("transaction").unwrap().to_owned())
                                .unwrap();
                    }
                    "m" => {
                        // In this case, we need to be provided with a wallet.json file which we deserialize to obtain certain
                        // parameters (public, private keys) we need to create our transaction
                        let slash = if env::consts::OS == "windows" {
                            "\\"
                        } else {
                            "/"
                        };
                        let data = fs::read_to_string("system".to_owned() + slash + "wallet.json");
                        if data.is_err() {
                            error!("Failed to load file. {:?}", data.err());
                            panic!();
                        }
                        let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
                        let wallet: Wallet =
                            serde_json::from_value(json.get("wallet").unwrap().to_owned()).unwrap();

                        // We will obtain the indices of wallet entries to only select certain keys and their outpoints
                        info!(
                            "Enter the indices of wallet entries you would like to enter delimited by a comma (e.g., 4,6,8,9) "
                        );
                        let mut indices = Vec::new();
                        let mut indices_out: String = String::new();
                        io::stdin()
                            .read_line(&mut indices_out)
                            .expect("Failed to read line");
                        let trimmed_indices = indices_out.trim();
                        for s in trimmed_indices.split(',') {
                            if let Ok(n) = s.trim().parse::<usize>() {
                                indices.push(n);
                            }
                        }

                        // We create the tx_inputs
                        for i in indices {
                            let (private_key, public_key, outpoint, value_from_outpoint) =
                                wallet.0[i].clone();

                            let tx_out: TxOut = TxOut {
                                value: value_from_outpoint,
                                pk_script: PublicKeyScript {
                                    public_key_hash: hash::hash_as_string(&public_key),
                                    verifier: Verifier {},
                                },
                            };

                            let message = String::from(&outpoint.txid)
                                + &outpoint.index.to_string()
                                + &tx_out.pk_script.public_key_hash;

                            let sig_script = SignatureScript {
                                signature: sign_and_verify::sign(
                                    &message,
                                    &private_key,
                                    &public_key,
                                ),
                                full_public_key: public_key,
                            };

                            let tx_in: TxIn = TxIn {
                                outpoint,
                                sig_script,
                            };

                            transaction.tx_inputs.append(&mut vec![tx_in]);
                        }

                        // We need the recipients to create the tx_outputs
                        info!(
                            "Enter the number of receipients you would like for your transaction: "
                        );
                        let mut str_out: String = String::new();
                        io::stdin()
                            .read_line(&mut str_out)
                            .expect("Failed to read line");
                        let trimmed_out = str_out.trim();
                        let num_out = match trimmed_out.parse::<u32>() {
                            Ok(i) => i,
                            Err(..) => {
                                error!("Period needs to be a u64");
                                panic!();
                            }
                        };

                        // We create the tx_outputs
                        for _i in 0..num_out {
                            info!("Enter the hash of the public key associated with the next recipient:");
                            let mut public_key = String::new();
                            io::stdin()
                                .read_line(&mut public_key)
                                .expect("Failed to read line");

                            info!("Enter the value associated with the next recipient:");
                            let mut str_value: String = String::new();
                            io::stdin()
                                .read_line(&mut str_value)
                                .expect("Failed to read line");
                            let trimmed_value = str_value.trim();
                            let value = match trimmed_value.parse::<u32>() {
                                Ok(i) => i,
                                Err(..) => {
                                    error!("Period needs to be a u64");
                                    panic!();
                                }
                            };

                            let tx_out: TxOut = TxOut {
                                value,
                                pk_script: PublicKeyScript {
                                    public_key_hash: hash::hash_as_string(&public_key),
                                    verifier: Verifier {},
                                },
                            };

                            transaction.tx_outputs.append(&mut vec![tx_out]);
                        }
                    }
                    _ => {
                        warn!("Invalid Command");
                        exit(0);
                    }
                }

                // We broadcast the transaction
                let (resp_tx, resp_rx) = oneshot::channel();
                let (resp_tx_1, resp_rx_1) = oneshot::channel();

                let cmd = Command::Get {
                    key: String::from("id_query"),
                    resp: resp_tx,
                };
                let cmd1 = Command::Get {
                    key: String::from("ports_query"),
                    resp: resp_tx_1,
                };

                tx_to_manager.send(cmd).await.ok();
                tx_to_manager.send(cmd1).await.ok();

                let result = resp_rx.await.unwrap().unwrap();
                let result1 = resp_rx_1.await.unwrap().unwrap();

                if result.is_empty() {
                    error!("Empty result from peer");
                    panic!();
                }
                if result1.is_empty() {
                    error!("Empty result from peer");
                    panic!();
                }

                let peerid: u32 = serde_json::from_str(&result[0]).unwrap();
                let ports: Vec<String> = serde_json::from_str(&result1[0]).unwrap();
                let local_ip = local_ip().unwrap().to_string();
                let frame = messages::get_transaction_msg(peerid, peerid, &transaction);
                peer::send_transaction(frame, local_ip, ports.to_owned()).await;

                /*let (peerid, _, ip_map, ports_map) = Peer::get_peer_info(&tx_to_manager).await;
                peer::broadcast(
                    messages::get_transaction_msg,
                    &transaction,
                    peerid,
                    &ip_map,
                    &ports_map,
                )
                .await;*/
            }
            "exit" | "Exit" | "EXIT" => {
                info!("The user selected exit");
                //Peer::shutdown(peer);
                write_log();
                exit(0);
            }

            _ => {
                warn!("Invalid Command");
            }
        }
    }
}

/**
 * TO BE DELETED. USED TO CREATE AN EXAMPLE TRANSACTION TO TEST NETWORKING
 */

fn get_example_transaction() -> Transaction {
    let (private_key0, public_key0) = sign_and_verify::create_keypair();
    let outpoint0: Outpoint = Outpoint {
        txid: "0".repeat(64),
        index: 0,
    };

    let tx_out0: TxOut = TxOut {
        value: 500,
        pk_script: PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key0),
            verifier: Verifier {},
        },
    };

    let (old_private_key, old_public_key) = (private_key0, public_key0);
    let message = String::from(&outpoint0.txid)
        + &outpoint0.index.to_string()
        + &tx_out0.pk_script.public_key_hash;

    let sig_script1 = SignatureScript {
        signature: sign_and_verify::sign(&message, &old_private_key, &old_public_key),
        full_public_key: old_public_key,
    };

    let tx_in1: TxIn = TxIn {
        outpoint: outpoint0,
        sig_script: sig_script1,
    };

    // We create a new keypair corresponding to our new transaction which allows us to create its tx_out
    let (_, public_key1) = sign_and_verify::create_keypair();
    let tx_out1: TxOut = TxOut {
        value: 500,
        pk_script: PublicKeyScript {
            public_key_hash: hash::hash_as_string(&public_key1),
            verifier: Verifier {},
        },
    };

    let transaction1: Transaction = Transaction {
        tx_inputs: Vec::from([tx_in1]),
        tx_outputs: Vec::from([tx_out1]),
    };

    return transaction1;
}

fn display_commands() {
    info!("--> help: Displays the availble commands");
    info!("--> sim start: Allows the user to begin the simple 3 node blockchain simulation");
    info!("--> save: Saves the configurations of the system to the config folder");
    info!("--> graph: Creates a dot file graph that visualizes the blockchain for a given config file");
    info!("--> exit: Exits the program with error code 0");
}

fn write_log() {
    // Allow us to access the path to the current directory
    let cwd = std::env::current_dir().unwrap();
    let cwd_from = std::env::current_dir().unwrap();
    let cwd_to = std::env::current_dir().unwrap();
    let cwd_log = std::env::current_dir().unwrap();

    // Allows us to access the path where we will store the new log file
    let mut dirpath = cwd.into_os_string().into_string().unwrap();
    // Allows us to access the path of the orginal log file we copy from
    let mut dirpath_from = cwd_from.into_os_string().into_string().unwrap();
    // Allows us to access the path of the log file we copy into
    let mut dirpath_to = cwd_to.into_os_string().into_string().unwrap();
    // Allows us to access the path of the orginal log file we copy from after we moved dirPathFrom
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
    // The new log file contains the current time
    let filename1: &str = &format!("{}.log", n1);
    dirpath_to.push_str(filename1);
    let file_path = dir_path.join(filename1);
    if let Err(e) = File::create(file_path) {
        println!("{:?}", e)
    }
    if let Err(e) = fs::copy(dirpath_from, dirpath_to) {
        println!("{:?}", e)
    }
    File::create(&dirpath_log).unwrap();
}
