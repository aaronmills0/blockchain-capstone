use std::{
    cmp::max,
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

use ed25519_dalek::Keypair;
use local_ip_address::local_ip;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

use crate::{
    components::{
        block::{Block, BlockHeader},
        merkle::Merkle,
        transaction::{Outpoint, PublicKeyScript, Transaction, TxOut},
        utxo::UTXO,
    },
    utils::{
        hash::{self, hash_as_string},
        sign_and_verify::{PublicKey, Verifier},
    },
};

use super::{
    messages,
    peer::{self, Command, MemPool, Peer, NUM_PARALLEL_TRANSACTIONS},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Miner {
    peer: Peer,
}

impl Miner {
    pub fn new() -> Miner {
        let miner = Miner { peer: Peer::new() };
        return miner;
    }

    pub async fn create_block(
        tx_peer: Sender<Command>,
        utxo: Arc<Mutex<UTXO>>,
        proc_mempool: Arc<Mutex<MemPool>>,
        verified_mempool: Arc<Mutex<MemPool>>,
        verified: Arc<Mutex<bool>>,
    ) {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: String::from("block_info_query"),
            resp: resp_tx,
        };

        tx_peer.send(cmd).await;

        let result = resp_rx.await;
        let result_vec = result.unwrap().unwrap();
        let prev_hash = result_vec[0].to_owned();
        let mut utxo_ref = utxo.lock().unwrap();
        let proc_mempool_ref = proc_mempool.lock().unwrap();
        let merkle_tree = Merkle::create_merkle_tree(&(*proc_mempool_ref).transactions);
        let (valid, updated_utxo) = (*utxo_ref).parallel_batch_verify_and_update(
            &(*proc_mempool_ref).transactions,
            max((*proc_mempool_ref).transactions.len(), num_cpus::get()) / num_cpus::get(),
        );
        if !valid {
            error!("Received an invalid transaction!"); // We can update this later
            panic!();
        }
        *utxo_ref = updated_utxo.unwrap();
        let mut verified_ref = verified.lock().unwrap();
        *verified_ref = true;
        let block = Block {
            header: BlockHeader {
                previous_hash: prev_hash,
                merkle_root: merkle_tree.tree.first().unwrap().clone(),
                nonce: 0,
            },
            merkle: merkle_tree,
            transactions: (*proc_mempool_ref).transactions.clone(),
        };

        let peer_id: u32 = serde_json::from_str(&result_vec[1]).unwrap();
        let ip_map: HashMap<u32, String> = serde_json::from_str(&result_vec[2]).unwrap();
        let port_map: HashMap<String, Vec<String>> = serde_json::from_str(&result_vec[3]).unwrap();
        tokio::spawn(async move {
            peer::broadcast(
                messages::get_block_msg,
                &block,
                peer_id,
                &ip_map,
                &port_map,
                true,
            )
            .await;
        });

        let proc_mempool_clone = proc_mempool.clone();
        let verified_mempool_clone = verified_mempool.clone();
        tokio::spawn(async move {
            Peer::update_mempool(verified_mempool_clone, proc_mempool_clone).await;
        });
    }

    async fn miner_manager(miner: Miner, mut rx: Receiver<Command>) {
        let (tx_peer, rx_peer) = mpsc::channel(32);
        let mut mempool = MemPool::new();

        let proc_mempool_arc = Arc::new(Mutex::new(MemPool::new()));
        let verified_mempool_arc = Arc::new(Mutex::new(MemPool::new()));
        let verified_arc = Arc::new(Mutex::new(true));

        let mut utxo: UTXO = UTXO(HashMap::new());
        let keypair = Keypair::from_bytes(&[
            9, 75, 189, 163, 133, 148, 28, 198, 139, 3, 56, 182, 118, 26, 250, 201, 129, 109, 104,
            32, 92, 248, 176, 200, 83, 98, 207, 118, 47, 231, 60, 75, 4, 65, 208, 174, 11, 82, 239,
            211, 201, 251, 90, 173, 173, 165, 36, 120, 162, 85, 139, 187, 164, 152, 53, 13, 62,
            219, 144, 86, 74, 205, 134, 25,
        ])
        .unwrap();
        let public_key = PublicKey(keypair.public);
        let outpoint: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };
        let tx_out: TxOut = TxOut {
            value: 500,
            pk_script: PublicKeyScript {
                public_key_hash: hash::hash_as_string(&public_key),
                verifier: Verifier {},
            },
        };
        utxo.insert(outpoint, tx_out);
        let utxo_arc = Arc::new(Mutex::new(utxo));

        tokio::spawn(async move {
            Peer::peer_manager(miner.peer, rx_peer).await;
        });

        loop {
            let command = rx.recv().await.unwrap();
            match command {
                Command::Set { key, resp, payload } => {
                    if key.as_str() == "transaction" {
                        if payload.is_none() {
                            error!("Invalid command: missing payload");
                            panic!();
                        }

                        let payload_vec = payload.unwrap();
                        if payload_vec.len() != 1 {
                            error!("Invalid command: payload is of unexpected size");
                            panic!();
                        }
                        let tx: Transaction = serde_json::from_str(&payload_vec[0])
                            .expect("Could not deserialize string to transaction.");

                        mempool.hashes.insert(hash_as_string(&tx));
                        mempool.transactions.push(tx.to_owned());
                        let verified_result = verified_arc.try_lock();
                        let verified = if verified_result.is_ok() {
                            *(verified_result.unwrap())
                        } else {
                            false
                        };
                        if mempool.transactions.len() < NUM_PARALLEL_TRANSACTIONS || !verified {
                            continue;
                        }
                        *(verified_arc.lock().unwrap()) = false;

                        let mut proc_mempool_ref = proc_mempool_arc.lock().unwrap();
                        *proc_mempool_ref = mempool.to_owned();
                        mempool = MemPool::new();

                        let utxo_arc_clone = utxo_arc.clone();
                        let proc_mempool_arc_clone = proc_mempool_arc.clone();
                        let verified_mempool_arc_clone = verified_mempool_arc.clone();
                        let verified_arc_clone = verified_arc.clone();
                        let tx_peer_clone = tx_peer.clone();
                        tokio::spawn(async move {
                            Miner::create_block(
                                tx_peer_clone,
                                utxo_arc_clone,
                                proc_mempool_arc_clone,
                                verified_mempool_arc_clone,
                                verified_arc_clone,
                            )
                            .await;
                        });
                    } else {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let cmd_peer = Command::Set {
                            key: key,
                            resp: resp_tx,
                            payload: payload,
                        };
                        tx_peer.send(cmd_peer).await;
                        let response = resp_rx.await;
                        resp.send(response.unwrap());
                    }
                }
                Command::Get { key, resp } => {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let cmd_peer = Command::Get {
                        key: key,
                        resp: resp_tx,
                    };
                    tx_peer.send(cmd_peer).await;
                    let response = resp_rx.await;
                    resp.send(response.unwrap());
                }
            }
        }
    }

    pub async fn launch() -> Sender<Command> {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let mut miner: Miner;
        // First load the peer from system/peer.json if it exists.
        if Path::new(&("system".to_owned() + slash + "miner.json")).exists() {
            miner = Miner::load_miner();
        } else {
            miner = Miner::new();
            info!("Miner doesn't exist! Creating new miner.");
            // Get peerid from the server
            let msg = messages::get_header_message_for_peerid_query();
            let response = peer::send_peerid_query(msg).await;
            miner.peer.peerid = response;
            // Set the id obtained as a response to the peer id
            Miner::save_miner(&miner);
        }

        miner.peer.set_ports().await;

        let msg =
            messages::get_ports_msg_for_maps_query(miner.peer.peerid, 1, miner.peer.ports.clone());
        let (ipmap, portmap) = peer::send_maps_query(
            msg,
            peer::SERVER_IP.to_owned(),
            peer::SERVER_PORTS.iter().map(|&s| s.into()).collect(),
        )
        .await;

        for (id, ip) in ipmap {
            miner.peer.ip_map.insert(id, ip);
        }
        for (ip, ports) in portmap {
            miner.peer.ports_map.insert(ip, ports);
        }

        Miner::save_miner(&miner);

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            Miner::miner_manager(miner, rx).await;
        });

        let (resp_tx, resp_rx) = oneshot::channel();
        let tx_clone = tx.clone();
        let cmd = Command::Get {
            key: String::from("ports_query"),
            resp: resp_tx,
        };

        tx_clone.send(cmd).await.ok();

        let result = resp_rx.await;
        let unwrap1 = result.unwrap();
        let result = unwrap1.unwrap();

        if result.is_empty() {
            error!("Empty result from peer");
            panic!();
        }
        let ports: Vec<String> = serde_json::from_str(&result[0]).unwrap();

        info!("Received Ports During Launch: {:?}", ports);

        let local_ip = local_ip().unwrap().to_string();
        for p in ports {
            let ip = local_ip.clone();
            let port = p.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move { Peer::listen(ip, port, tx_clone).await });
        }

        return tx.clone();
    }

    pub fn save_miner(miner: &Miner) {
        let mut map = Map::new();
        let miner_json = serde_json::to_value(miner);

        if miner_json.is_err() {
            error!("Failed to serialize miner peer");
            panic!();
        }

        let mut json = miner_json.unwrap();

        map.insert(String::from("miner"), json);

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

        let file_name: &str = &format!("miner.json");

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
    }

    pub fn load_miner() -> Miner {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let data = fs::read_to_string("system".to_owned() + slash + "miner.json");
        if data.is_err() {
            error!("Failed to load file. {:?}", data.err());
            panic!();
        }
        let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
        let server = serde_json::from_value(json.get("miner").unwrap().to_owned());
        return server.unwrap();
    }
}
