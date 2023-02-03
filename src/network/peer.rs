use crate::components::transaction::Transaction;
use crate::network::decoder;
use crate::network::messages;
use local_ip_address::local_ip;
use log::{error, info, warn};
use mini_redis::{Connection, Frame};
use port_scanner::scan_port;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::{env, fs};
use std::{fs::File, io, path::Path};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

static SERVER_IP: &str = "192.168.0.101";
const SERVER_PORTS: &[&str] = &["57643", "34565", "32578", "23564", "13435"];
static NUM_PORTS: usize = 20;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Peer {
    pub address: String,
    pub peerid: u32,
    pub ports: Vec<String>,
    pub ip_map: HashMap<u32, String>, // IP addresses of neighbors
    pub port_map: HashMap<String, Vec<String>>, // Ports used with IP addresses of neighbours
}

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<String>>,
        payload: Option<Vec<String>>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

async fn get_connection(ip: &str, ports: &[&str]) -> Connection {
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
        error!("Could not connect to any server port");
        panic!();
    }
    return connection_wrapped.unwrap();
}

pub async fn send_peerid_query(msg: Frame) -> u32 {
    let mut connection = get_connection(SERVER_IP, SERVER_PORTS).await;
    connection.write_frame(&msg).await.ok();

    let mut response: u32 = 0;
    if let Some(frame) = connection.read_frame().await.unwrap() {
        response =
            decoder::decode_peerid_response(frame).expect("Failed to decode peerid response");
    }

    return response;
}

pub async fn send_ports_query(msg: Frame) -> (HashMap<u32, String>, HashMap<String, Vec<String>>) {
    let mut connection = get_connection(SERVER_IP, SERVER_PORTS).await;
    connection.write_frame(&msg).await.ok();

    let mut ip = None;
    let mut ports = None;
    if let Some(frame) = connection.read_frame().await.unwrap() {
        (ip, ports) = decoder::decode_ports_response(frame);
    }
    if ip.is_none() || ports.is_none() {
        error!("Decoded ip or ports are none from ports query");
        panic!();
    }

    return (ip.unwrap(), ports.unwrap());
}

pub async fn send_transaction(msg: Frame, addr: String, sourceid: u32, destid: u32) -> bool {
    let stream = TcpStream::connect(&addr).await.unwrap();
    info!("Successfully connected to {}", addr);
    let mut connection = Connection::new(stream);

    connection.write_frame(&msg).await.ok();
    if let Some(frame) = connection.read_frame().await.unwrap() {
        let (cmd, sourceid, destid) = decoder::decode_command(&frame);
        if cmd != "ACK" || sourceid != destid || destid != sourceid {
            warn!(
                "Received invalid acknowledgement {} {} {}",
                cmd != "ACK",
                sourceid != destid,
                destid != sourceid
            );
            return false;
        };
    }
    return true;
}

impl Peer {
    pub fn new() -> Peer {
        return Peer {
            address: local_ip().expect("Failed to obtain local ip").to_string(),
            peerid: 0,
            ports: Vec::with_capacity(NUM_PORTS),
            ip_map: HashMap::new(),
            port_map: HashMap::new(),
        };
    }

    pub async fn launch() -> Peer {
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
            let msg = messages::get_peerid_query();
            let response = send_peerid_query(msg).await;
            peer.peerid = response;
            // Set the id obtained as a response to the peer id
            Peer::save_peer(&peer);
        }

        info!("IP map: {:?}", peer.ip_map);
        info!("Port map: {:?}", peer.port_map);

        // We need to ensure all our ports are available. If not we need to change them.
        // Query the server
        peer.set_ports().await;
        let msg = messages::get_ports_query(peer.peerid, peer.ports.clone());
        let (ipmap, portmap) = send_ports_query(msg).await;
        for (id, ip) in ipmap {
            peer.ip_map.insert(id, ip);
        }
        for (ip, ports) in portmap {
            peer.port_map.insert(ip, ports);
        }

        Peer::save_peer(&peer);

        return peer;
    }

    async fn peer_manager(peer: Peer, mut rx: Receiver<Command>) {
        loop {
            let command = rx.recv().await.unwrap();
            match command {
                Command::Get { key, resp, payload } => {
                    if key.as_str() == "transaction" {
                        resp.send(Ok(Some(String::from("Received!"))));
                    }
                }
            }
        }
    }

    pub async fn listen(peer: Peer, ip: String, port: String) {
        let socket = ip + ":" + &port;
        let listener = TcpListener::bind(&socket).await.unwrap();
        info!("Successfully setup listener at {}", socket);

        // The server should now continuously listen and respond to queries
        // Each time it gets a request it should update its socketmap accordingly
        let peerid = peer.peerid;
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            Peer::peer_manager(peer, rx).await;
        });

        loop {
            info!("Waiting for connection...");
            let (stream, socket) = listener.accept().await.unwrap();

            info!("{:?}", &stream);
            // A new task is spawned for each inbound socket. The socket is
            // moved to the new task and processed there.
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                Peer::process_connection(stream, socket.to_string(), tx_clone, peerid).await;
            });
        }
    }

    async fn process_connection(
        stream: TcpStream,
        socket: String,
        tx: Sender<Command>,
        peerid: u32,
    ) {
        let mut connection = Connection::new(stream);
        loop {
            match connection.read_frame().await {
                Ok(opt_frame) => {
                    if let Some(frame) = opt_frame {
                        let cmd;
                        info!("GOT: {:?}", frame);
                        let (command, sourceid, destid) = decoder::decode_command(&frame);

                        if destid != peerid {
                            warn!("Destination id does not match server id: {}", destid);
                            return;
                        }
                        let (resp_tx, resp_rx) = oneshot::channel();
                        if command == "transaction" {
                            let transaction: Transaction = decoder::decode_transactions_msg(frame)
                                .expect("Cannot decode frame as transaction frame");
                            cmd = Command::Get {
                                key: command,
                                resp: resp_tx,
                                payload: None,
                            };
                            tx.send(cmd).await.ok();
                            let result = resp_rx.await;
                            let response = messages::get_ack_msg(destid, sourceid);
                            connection.write_frame(&response).await.ok();
                        } else {
                            warn!("invalid command for server");
                            return;
                        }
                    }
                }
                Err(e) => {
                    warn!("{}", e);
                    break;
                }
            }
        }
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
        for i in 0..self.ports.len() {
            if !scan_port(self.ports[i].parse::<u16>().unwrap()) {
                info!("Port {} if unavailable. Setting new port...", i);
                self.ports[i] = self.set_new_port().await;
            }
        }

        // Add new ports until there are `NUM_PORTS` ports
        while self.ports.len() < NUM_PORTS {
            let new_port = self.set_new_port().await;
            self.ports.push(new_port);
        }
    }

    pub fn shutdown(peer: Peer) {
        Peer::save_peer(&peer);
    }
    pub async fn spawn_connection(socket: &str) {
        info!("External: {}", &socket);
        // The 'await' expression suspends the operation until it is ready to be processed. It continues to the next operation.
        let stream = TcpStream::connect(&socket).await.unwrap();
        info!("Successfully connected to {}", socket);
        let mut connection = Connection::new(stream);

        loop {
            let mut command = String::new();
            io::stdin()
                .read_line(&mut command)
                .expect("Failed to read line");

            let frame = Frame::Simple(command);

            connection.write_frame(&frame).await.unwrap();
        }
    }

    pub async fn spawn_listener(peer: Arc<Peer>) {
        let local_ip = local_ip().unwrap();
        let address = local_ip.to_string();
        let socket = address + ":6780";

        let listener = TcpListener::bind(&socket).await.unwrap();
        info!("Successfully setup listener at {}", socket);
        loop {
            let (socket, _) = listener.accept().await.unwrap();

            info!("{:?}", &socket);
            // A new task is spawned for each inbound socket. The socket is
            // moved to the new task and processed there.
            tokio::spawn(async move {
                Peer::process(socket).await;
            });
        }
    }

    async fn process(socket: TcpStream) {
        // The `Connection` lets us read/write redis **frames** instead of
        // byte streams. The `Connection` type is defined by mini-redis.
        let mut connection = Connection::new(socket);
        loop {
            if let Some(frame) = connection.read_frame().await.unwrap() {
                info!("GOT: {:?}", frame);
            }
        }
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
