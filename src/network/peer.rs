use crate::components::block::Block;
use crate::components::block::BlockHeader;
use crate::components::merkle::Merkle;
use crate::components::transaction::Outpoint;
use crate::components::transaction::PublicKeyScript;
use crate::components::transaction::Transaction;
use crate::components::transaction::TxOut;
use crate::components::utxo::UTXO;
use crate::network::decoder;
use crate::network::messages;
use crate::shell::get_example_transaction;
use crate::utils::hash;
use crate::utils::hash::hash_as_string;
use crate::utils::save_and_load::load_object;
use crate::utils::save_and_load::save_object;
use crate::utils::sign_and_verify;
use crate::utils::sign_and_verify::PrivateKey;
use crate::utils::sign_and_verify::PublicKey;
use crate::utils::sign_and_verify::Verifier;
use ed25519_dalek::Keypair;
use local_ip_address::local_ip;
use log::{error, info, warn};
use mini_redis::{Connection, Frame};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use std::{env, fs, io};
use std::{fs::File, path::Path};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub static SERVER_IP: &str = "192.168.0.15";
pub const SERVER_PORTS: &[&str] = &["57643", "34565", "32578", "23564", "13435"];
pub static NUM_PORTS: usize = 5;
pub static NUM_PARALLEL_TRANSACTIONS: usize = 1024;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Peer {
    pub address: String,
    pub peerid: u32,
    pub ports: Vec<String>,
    pub ip_map: HashMap<u32, String>, // IP addresses of neighbors
    pub ports_map: HashMap<String, Vec<String>>, // Ports used with IP addresses of neighbours
    pub blockchain: Vec<Block>,       // Blocks
    pub block_map: HashMap<String, usize>, // Map block hashes to indices in the blockchain for quick access.
    pub utxo: UTXO,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MemPool {
    pub hashes: HashSet<String>,
    pub transactions: Vec<Transaction>,
}

impl MemPool {
    pub fn new() -> MemPool{
        return MemPool {
            hashes: HashSet::new(),
            transactions: Vec::new(),
        };
    }
}

#[derive(Debug)]
pub enum Command {
    Get {
        key: String,
        resp: Responder<Vec<String>>,
    },
    Set {
        key: String,
        resp: Responder<Vec<String>>,
        payload: Option<Vec<String>>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

pub async fn get_connection(ip: &str, ports: &[&str]) -> Option<Connection> {
    let mut connection_wrapped: Option<Connection> = None;
    for port in ports {
        let socket = String::from(ip) + ":" + port;

        let conn = TcpStream::connect(&socket).await;
        if conn.is_err() {
            continue;
        };
        let stream = conn.unwrap();

        info!("Successfully connected to {}", socket);
        connection_wrapped = Some(Connection::new(stream));
        break;
    }

    if connection_wrapped.is_none() {
        error!("Could not connect to any port");
    }
    return connection_wrapped;
}

pub async fn send_peerid_query(header_msg: Frame) -> u32 {
    let connection_opt = get_connection(SERVER_IP, SERVER_PORTS).await;
    if connection_opt.is_none() {
        panic!("Cannot connect to the server");
    }
    let mut connection = connection_opt.unwrap();

    connection.write_frame(&header_msg).await.ok();

    let mut response: u32 = 0;
    if let Some(frame) = connection.read_frame().await.unwrap() {
        response =
            decoder::decode_peerid_response(frame).expect("Failed to decode peerid response");
    }

    return response;
}

pub async fn send_maps_query(
    ports_msg: Frame,
    ip: String,
    ports: Vec<String>,
) -> (HashMap<u32, String>, HashMap<String, Vec<String>>) {
    let ports: Vec<&str> = ports.iter().map(AsRef::as_ref).collect();
    let connection_opt = get_connection(&ip, ports.as_slice()).await;
    if connection_opt.is_none() {
        panic!("Cannot connect to the server");
    }
    let mut connection = connection_opt.unwrap();
    connection.write_frame(&ports_msg).await.ok();

    let mut ip_map = None;
    let mut ports_map = None;
    if let Some(frame) = connection.read_frame().await.unwrap() {
        (ip_map, ports_map) = decoder::decode_maps_response(frame);
    }
    if ip_map.is_none() || ports_map.is_none() {
        error!("Decoded ip_map or ports_map are none from maps query");
        panic!();
    }

    return (ip_map.unwrap(), ports_map.unwrap());
}

pub async fn broadcast<T: Clone>(
    msg_fn: fn(u32, u32, T) -> Frame,
    payload: T,
    peerid: u32,
    ip_map: &HashMap<u32, String>,
    port_map: &HashMap<String, Vec<String>>,
) {
    for (id, ip) in ip_map {
        if *id == peerid {
            continue;
        }
        let frame = msg_fn(peerid, *id, payload.clone());
        let ports: Vec<&str> = port_map[ip].iter().map(AsRef::as_ref).collect();
        let connection_opt = get_connection(ip, ports.as_slice()).await;
        if connection_opt.is_none() {
            panic!("Cannot connect to the server");
        }
        let mut connection = connection_opt.unwrap();
        connection.write_frame(&frame).await.ok();
    }

    info!("Broadcasting transactions was successful!!!");
}

impl Peer {
    pub fn new() -> Peer {
        let mut utxo = UTXO(HashMap::new());

        let keypair = Keypair::from_bytes(&[
            9, 75, 189, 163, 133, 148, 28, 198, 139, 3, 56, 182, 118, 26, 250, 201, 129, 109, 104,
            32, 92, 248, 176, 200, 83, 98, 207, 118, 47, 231, 60, 75, 4, 65, 208, 174, 11, 82, 239,
            211, 201, 251, 90, 173, 173, 165, 36, 120, 162, 85, 139, 187, 164, 152, 53, 13, 62,
            219, 144, 86, 74, 205, 134, 25,
        ])
        .unwrap();
        let private_key0 = PrivateKey(keypair.secret);
        let public_key0 = PublicKey(keypair.public);

        let hash_public_key0 = hash::hash_as_string(&public_key0);
        let outpoint0: Outpoint = Outpoint {
            txid: "0".repeat(64),
            index: 0,
        };

        let tx_out0: TxOut = TxOut {
            value: 100000,
            pk_script: PublicKeyScript {
                public_key_hash: hash_public_key0,
                verifier: Verifier {},
            },
        };
        utxo.insert(outpoint0, tx_out0);

        let mut peer = Peer {
            address: local_ip().expect("Failed to obtain local ip").to_string(),
            peerid: 0,
            ports: Vec::with_capacity(NUM_PORTS),
            ip_map: HashMap::new(),
            ports_map: HashMap::new(),
            blockchain: vec![Block {
                header: BlockHeader {
                    previous_hash: "0".repeat(32),
                    merkle_root: "0".repeat(32),
                    nonce: 0,
                },
                merkle: Merkle { tree: Vec::new() },
                transactions: Vec::new(),
            }],
            block_map: HashMap::new(),
            utxo,
        };
        peer.block_map
            .insert(hash_as_string(&peer.blockchain[0]), 0);
        return peer;
    }

    pub async fn launch() -> Sender<Command> {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let mut peer: Peer;
        // First load the peer from system/peer.json if it exists.
        if Path::new(&("system".to_owned() + slash + "peer.json")).exists() {
            peer = Peer::load_peer();
        } else {
            peer = Peer::new();
            info!("Peer doesn't exist! Creating new peer.");
            // Get peerid from the server
            let msg = messages::get_header_message_for_peerid_query();
            let response = send_peerid_query(msg).await;
            peer.peerid = response;
            // Set the id obtained as a response to the peer id
            Peer::save_peer(&peer);

            // We create a new wallet for each peer
            let (private_key_initial, public_key_initial) = sign_and_verify::create_keypair();
            let wallet: Vec<(PrivateKey, PublicKey, Outpoint, u32)> = vec![(
                private_key_initial,
                public_key_initial,
                Outpoint {
                    txid: "0".repeat(64),
                    index: 0,
                },
                500,
            )];
            save_object(&wallet, String::from("wallet"), String::from("system"));
        }

        info!("IP map: {:?}", peer.ip_map);
        info!("Port map: {:?}", peer.ports_map);

        // We need to ensure all our ports are available. If not we need to change them.
        // Query the server

        peer.set_ports().await;
        let msg = messages::get_ports_msg_for_maps_query(peer.peerid, 1, peer.ports.clone());
        let (ipmap, ports_map) = send_maps_query(
            msg,
            SERVER_IP.to_owned(),
            SERVER_PORTS.iter().map(|&s| s.into()).collect(),
        )
        .await;

        for (id, ip) in ipmap {
            peer.ip_map.insert(id, ip);
        }
        for (ip, ports) in ports_map {
            peer.ports_map.insert(ip, ports);
        }

        Peer::save_peer(&peer);

        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            Peer::peer_manager(peer, rx).await;
        });

        let (resp_tx, resp_rx) = oneshot::channel();
        let tx_clone = tx.clone();
        let cmd = Command::Get {
            key: String::from("ports_query"),
            resp: resp_tx,
        };

        tx_clone.send(cmd).await.ok();

        let result = resp_rx.await.unwrap().unwrap();
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

    pub async fn get_peer_info(
        tx_to_manager: &Sender<Command>,
    ) -> (
        u32,
        Vec<String>,
        HashMap<u32, String>,
        HashMap<String, Vec<String>>,
    ) {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: String::from("all"),
            resp: resp_tx,
        };
        tx_to_manager.send(cmd).await.ok();

        let result = resp_rx.await.unwrap().unwrap();
        if result.is_empty() {
            error!("Empty result from peer");
            panic!();
        }

        let peerid: u32 = serde_json::from_str(&result[0]).unwrap();
        let ports: Vec<String> = serde_json::from_str(&result[1]).unwrap();
        let ip_map: HashMap<u32, String> = serde_json::from_str(&result[2]).unwrap();
        let ports_map: HashMap<String, Vec<String>> = serde_json::from_str(&result[3]).unwrap();

        return (peerid, ports, ip_map, ports_map);
    }

    pub async fn set_new_port(&mut self) -> String {
        let listener = TcpListener::bind(self.address.clone() + ":0")
            .await
            .unwrap();
        return listener
            .local_addr()
            .expect("Failed to unwrap listener socket address")
            .port()
            .to_string();
    }

    pub async fn set_ports(&mut self) {
        // Update any set ports that are unavailable
        // for i in 0..self.ports.len() {
        //     let socket = self.address.clone() + ":" + &self.ports[i];
        //     let conn = TcpStream::connect(&socket).await;
        //     if conn.is_err() {
        //         info!("Port {} is unavailable. Setting new port...", i);
        //         self.ports[i] = self.set_new_port().await;
        //     };
        // }

        // Add new ports until there are `NUM_PORTS` ports
        while self.ports.len() < NUM_PORTS {
            let new_port = self.set_new_port().await;
            self.ports.push(new_port);
        }
    }
    
    async fn update_mempool(verified_mempool: Arc<Mutex<MemPool>>, proc_mempool: Arc<Mutex<MemPool>>, time: Arc<Mutex<Instant>>) {
        let mut verified_mempool_ref = verified_mempool.lock().unwrap();
        let mut proc_mempool_ref = proc_mempool.lock().unwrap();
        (*verified_mempool_ref).hashes.extend((*proc_mempool_ref).hashes.to_owned());
        (*verified_mempool_ref)
            .transactions
            .append(&mut (*proc_mempool_ref).transactions); // Removes all elements from mempool
        if (*verified_mempool_ref).transactions.len() == 65536 {
            println!("Total time for batch size of {} txs: {}us, ", NUM_PARALLEL_TRANSACTIONS, (*(time.lock().unwrap())).elapsed().as_micros());
        };
    }
    
    async fn verify_mempool(utxo: Arc<Mutex<UTXO>>, proc_mempool: Arc<Mutex<MemPool>>, verified_mempool: Arc<Mutex<MemPool>>, verified: Arc<Mutex<bool>>, time: Arc<Mutex<Instant>>) {
        let mut utxo_ref = utxo.lock().unwrap();
        let proc_mempool_ref = proc_mempool.lock().unwrap();
        let (valid, updated_utxo) = (*utxo_ref).parallel_batch_verify_and_update(
            &(*proc_mempool_ref).transactions,
            max((*proc_mempool_ref).transactions.len(), num_cpus::get()) / num_cpus::get()
        );
        if !valid {
            error!("Received an invalid transaction!"); // We can update this later
            panic!();
        }
        *utxo_ref = updated_utxo.unwrap();
        let mut verified_ref = verified.lock().unwrap();
        *verified_ref = true;
             
        let proc_mempool_clone = proc_mempool.clone();
        let verified_mempool_clone = verified_mempool.clone();
        let time_clone = time.clone();
        tokio::spawn(async move {
            Peer::update_mempool(verified_mempool_clone, proc_mempool_clone, time_clone).await;
        });
    }

    pub async fn peer_manager(mut peer: Peer, mut rx: Receiver<Command>) {
        let mut mempool = MemPool::new();

        let proc_mempool_arc = Arc::new(Mutex::new(MemPool::new()));
        let verified_mempool_arc = Arc::new(Mutex::new(MemPool::new()));
        let verified_arc = Arc::new(Mutex::new(true));

        let time_arc = Arc::new(Mutex::new(Instant::now()));
        
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
        utxo.insert(outpoint.clone(), tx_out.clone());
        let utxo_arc = Arc::new(Mutex::new(utxo));

        let mut first_tx = true;
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
                        
                        if first_tx {
                            println!("Received first tx!");
                            first_tx = false;
                            *(time_arc.lock().unwrap()) = Instant::now();
                        }
                        
                        mempool.hashes.insert(hash_as_string(&tx));
                        mempool.transactions.push(tx.to_owned());
                        let verified_result = verified_arc.try_lock();
                        let verified = if verified_result.is_ok() {
                            *(verified_result.unwrap())
                        } else {
                            false
                        };
                        if mempool.transactions.len() < NUM_PARALLEL_TRANSACTIONS || !verified
                        {
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
                        let time_arc_clone = time_arc.clone();
                        tokio::spawn(async move {
                            Peer::verify_mempool(utxo_arc_clone, proc_mempool_arc_clone, verified_mempool_arc_clone, verified_arc_clone, time_arc_clone).await;
                        });
                    } else if key.as_str() == "block" {
                        if payload.is_none() {
                            error!("Invalid command: missing payload");
                            panic!();
                        }

                        let payload_vec = payload.unwrap();
                        if payload_vec.len() != 1 {
                            error!("Invalid command: payload is of unexpected size");
                            panic!();
                        }
                        let block: Block = serde_json::from_str(&payload_vec[0])
                            .expect("Could not deserialize string to block.");
                        info!("Block: {:?}", block);

                        if peer.block_map.contains_key(&hash_as_string(&block)) {
                            continue;
                        }
                        let (valid, utxo_option) = peer.verify_block(&block);

                        if !valid {
                            continue;
                        }

                        broadcast(
                            messages::get_block_msg,
                            &block,
                            peer.peerid,
                            &peer.ip_map,
                            &peer.ports_map,
                        )
                        .await;

                        peer.block_map
                            .insert(hash_as_string(&block), peer.blockchain.len());
                        peer.blockchain.push(block.to_owned());

                        peer.utxo = utxo_option.unwrap().to_owned();
                    } else if key.as_str() == "maps_query" {
                        if payload.is_none() {
                            error!("Invalid command: missing payload");
                            panic!();
                        }

                        let payload_vec = payload.unwrap();
                        if payload_vec.len() <= 2 {
                            error!("Invalid command: payload is of unexpected size");
                            panic!();
                        }

                        let sourceid: u32 = payload_vec[0].parse().unwrap();
                        let ip = payload_vec[1].clone();

                        // Update the server ip_map and ports_map
                        peer.ip_map.insert(sourceid, ip.clone());
                        peer.ports_map.insert(ip.clone(), payload_vec[2..].to_vec());

                        let response_vector = vec![
                            serde_json::to_string(&peer.ip_map)
                                .expect("Failed to serialize ip map"),
                            serde_json::to_string(&peer.ports_map)
                                .expect("Failed to serialize ports map"),
                        ];
                        resp.send(Ok(response_vector)).ok();
                        Peer::save_peer(&peer);
                    } else if key.as_str() == "BD_query" {
                        if payload.is_none() {
                            error!("Invalid command: missing payload");
                            panic!();
                        }

                        let payload_vec = payload.unwrap();
                        if payload_vec.len() != 1 {
                            error!("Invalid command: payload is of unexpected size");
                            panic!();
                        }
                        let hash = payload_vec[0].to_owned();
                        let response_vector: Vec<String>;
                        if !peer.block_map.contains_key(&hash) {
                            response_vector = Vec::new();
                        } else {
                            let index = peer.block_map[&hash];
                            let blocks_ref = &peer.blockchain[index + 1..];
                            response_vector = vec![serde_json::to_string(blocks_ref).unwrap()];
                        }
                        resp.send(Ok(response_vector)).ok();
                    } else {
                        warn!("invalid command for peer");
                        continue;
                    }
                }
                Command::Get { key, resp } => {
                    let response_vector: Vec<String> = match key.as_str() {
                        "id_query" => {
                            vec![serde_json::to_string(&peer.peerid)
                                .expect("Failed to serialize id")]
                        }
                        "ports_query" => {
                            vec![serde_json::to_string(&peer.ports)
                                .expect("Failed to serialize ports")]
                        }
                        "ip_map_query" => {
                            vec![serde_json::to_string(&peer.ip_map)
                                .expect("Failed to serialize ip map")]
                        }
                        "ports_map_query" => {
                            vec![serde_json::to_string(&peer.ports_map)
                                .expect("Failed to serialize ports map")]
                        }
                        "block_info_query" => {
                            vec![
                                hash_as_string(&peer.blockchain.last().unwrap()),
                                serde_json::to_string(&peer.peerid)
                                    .expect("Failed to serialize id"),
                                serde_json::to_string(&peer.ip_map)
                                    .expect("Failed to serialize ip map"),
                                serde_json::to_string(&peer.ports_map)
                                    .expect("Failed to serialize ports map"),
                            ]
                        }
                        "all" => {
                            vec![
                                serde_json::to_string(&peer.peerid)
                                    .expect("Failed to serialize id"),
                                serde_json::to_string(&peer.ports)
                                    .expect("Failed to serialize ports"),
                                serde_json::to_string(&peer.ip_map)
                                    .expect("Failed to serialize ip map"),
                                serde_json::to_string(&peer.ports_map)
                                    .expect("Failed to serialize ports map"),
                            ]
                        }
                        _ => {
                            warn!("invalid command for peer");
                            continue;
                        }
                    };
                    resp.send(Ok(response_vector)).ok();
                }
            }
        }
    }

    pub async fn listen(ip: String, port: String, tx: Sender<Command>) {
        let socket = ip + ":" + &port;
        let listener = TcpListener::bind(&socket).await.unwrap();
        info!("Successfully setup listener at {}", socket);

        // The server should now continuously listen and respond to queries
        // Each time it gets a request it should update its socketmap accordingly

        loop {
            info!("Waiting for connection...");
            let (stream, socket) = listener.accept().await.unwrap();

            info!("{:?}", &stream);
            // A new task is spawned for each inbound socket. The socket is
            // moved to the new task and processed there.
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                Peer::process_connection(stream, socket.to_string(), tx_clone).await;
            });
        }
    }

    async fn process_connection(stream: TcpStream, socket: String, tx: Sender<Command>) {
        let ip = stream.peer_addr().unwrap().ip().to_string();
        let mut connection = Connection::new(stream);
        loop {
            match connection.read_frame().await {
                Ok(opt_frame) => {
                    if let Some(frame) = opt_frame {
                        let cmd;
                        // info!("GOT: {:?}", frame);
                        let (command, sourceid, destid) = decoder::decode_command(&frame);

                        let (resp_tx, resp_rx) = oneshot::channel();
                        if command == "transaction" || command == "block" {
                            let json = decoder::decode_json_msg(frame);
                            if json.is_none() {
                                error!("Missing json");
                                panic!()
                            }
                            cmd = Command::Set {
                                key: command,
                                resp: resp_tx,
                                payload: Some(vec![json.unwrap()]),
                            };
                            tx.send(cmd).await.ok();
                        } else if command == "maps_query" {
                            let mut ports = decoder::decode_ports(&frame);
                            if ports.is_empty() {
                                error!("No ports found when decoding ports for maps query");
                                panic!();
                            }

                            let mut payload_vec = Vec::new();
                            payload_vec.push(sourceid.to_string());
                            payload_vec.push(ip.clone());
                            payload_vec.append(&mut ports);

                            cmd = Command::Set {
                                key: command,
                                resp: resp_tx,
                                payload: Some(payload_vec),
                            };
                            tx.send(cmd).await.ok();

                            let result = resp_rx.await.unwrap().unwrap();
                            if result.is_empty() {
                                error!("Empty result from peer");
                                panic!();
                            }
                            let ip_map_json = result[0].to_owned();
                            let ports_map_json = result[1].to_owned();
                            info!("Sending ip_map: {:?}", ip_map_json);
                            info!("Sending ports_map: {:?}", ports_map_json);

                            let response = messages::get_maps_response(
                                sourceid,
                                destid,
                                ip_map_json,
                                ports_map_json,
                            );
                            connection.write_frame(&response).await.ok();
                        } else if command == "BD_query" {
                            let hash = decoder::decode_head_hash(frame);
                            if hash.is_none() {
                                let frame = messages::get_termination_msg(sourceid, destid);
                                connection.write_frame(&frame).await.ok();
                                return;
                            }
                            cmd = Command::Set {
                                key: command,
                                resp: resp_tx,
                                payload: Some(vec![hash.unwrap()]),
                            };
                            tx.send(cmd).await.ok();
                            let result = resp_rx.await;
                            let blocks_json = result.unwrap().unwrap()[0].clone();
                            let frame: Frame;
                            if blocks_json.is_empty() {
                                frame = messages::get_termination_msg(sourceid, destid)
                            } else {
                                frame = messages::get_bd_response(sourceid, destid, blocks_json);
                            }
                            connection.write_frame(&frame).await.ok();
                        } else {
                            warn!("invalid command for peer");
                            return;
                        }
                    }
                }
                Err(e) => {
                    warn!("{}", e);
                    return;
                }
            }
        }
    }

    pub async fn download_blocks(&mut self) -> bool {
        let mut connection_opt: Option<Connection> = None;
        let mut destid = 0;
        for (id, ip) in &self.ip_map {
            let ports: Vec<&str> = self.ports_map[ip].iter().map(AsRef::as_ref).collect();
            connection_opt = get_connection(ip, &ports).await;
            if connection_opt.is_some() {
                destid = *id;
                break;
            }
        }

        if connection_opt.is_none() {
            panic!("Failed to connect to any peer");
        }

        let mut connection = connection_opt.unwrap();

        let msg = messages::get_head_hash_msg_for_bd_query(
            self.peerid,
            destid,
            hash_as_string(self.blockchain.last().unwrap()),
        );

        connection.write_frame(&msg).await.ok();

        let mut blocks = Vec::new();
        if let Some(frame) = connection.read_frame().await.unwrap() {
            blocks = decoder::decode_bd_response(frame);
        }
        let mut utxo_option = None;
        for block in &blocks {
            let valid;
            (valid, utxo_option) = self.verify_block(block);
            if !valid {
                error!("One of the blocks received is invalid");
                return false;
            }
        }
        self.utxo = utxo_option.unwrap();
        for block in blocks {
            self.block_map
                .insert(hash_as_string(&block), self.blockchain.len());
            self.blockchain.push(block);
        }
        return true;
    }

    pub fn verify_block(&self, block: &Block) -> (bool, Option<UTXO>) {
        if block.header.previous_hash != hash_as_string(&self.blockchain.last().unwrap()) {
            return (false, None);
        }

        let merkle_tree = Merkle::create_merkle_tree(&block.transactions);

        if !merkle_tree
            .tree
            .first()
            .unwrap()
            .eq(&block.header.merkle_root)
        {
            return (false, None);
        }

        let (valid, utxo_option) = self.utxo.parallel_batch_verify_and_update(
            &block.transactions,
            max(block.transactions.len(), num_cpus::get()) / num_cpus::get(),
        );
        if !valid {
            warn!("Received invalid block");
            return (false, None);
        }
        return (true, utxo_option);
    }

    pub fn shutdown(peer: Peer) {
        Peer::save_peer(&peer);
    }

    pub fn save_peer(peer: &Peer) {
        let mut map = Map::new();
        let peer_json = serde_json::to_value(peer);

        if peer_json.is_err() {
            error!("Failed to serialize peer");
            panic!();
        }

        let mut json = peer_json.unwrap();

        map.insert(String::from("peer"), json);

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

        let file_name: &str = &format!("peer.json");

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

    pub fn load_peer() -> Peer {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let data = fs::read_to_string("system".to_owned() + slash + "peer.json");
        if data.is_err() {
            error!("Failed to load file. {:?}", data.err());
            panic!();
        }
        let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
        let peer = serde_json::from_value(json.get("peer").unwrap().to_owned());
        return peer.unwrap();
    }
}
