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

static SERVER_ADDR: &str = "127.0.0.1:6780";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Peer {
    pub peerid: u32,
    pub socketmap: HashMap<u32, String>, // Socket addresses of neighbors
    pub address: String,
    pub port: Option<String>,
}

async fn send_peerid_query(msg: Frame) -> u32 {
    let stream = TcpStream::connect(&SERVER_ADDR).await.unwrap();
    info!("Successfully connected to {}", SERVER_ADDR);
    let mut connection = Connection::new(stream);

    connection.write_frame(&msg).await.ok();

    let mut response: u32 = 0;
    if let Some(frame) = connection.read_frame().await.unwrap() {
        response =
            decoder::decode_peerid_response(frame).expect("Failed to decode peerid response");
    }

    return response;
}

async fn send_sockets_query(msg: Frame) -> HashMap<u32, String> {
    let stream = TcpStream::connect(&SERVER_ADDR).await.unwrap();
    info!("Successfully connected to {}", SERVER_ADDR);
    let mut connection = Connection::new(stream);

    connection.write_frame(&msg).await.ok();

    let mut response: HashMap<u32, String> = HashMap::new();
    if let Some(frame) = connection.read_frame().await.unwrap() {
        response =
            decoder::decode_sockets_response(frame).expect("Failed to decode sockets response");
    }

    return response;
}

impl Peer {
    pub fn new() -> Peer {
        return Peer {
            peerid: 0,
            socketmap: HashMap::new(),
            address: local_ip().expect("Failed to obtain local ip").to_string(),
            port: None,
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
            // Binding with port 0 tells the OS to find a suitable port. We will save this port.
            peer.set_new_port().await;
            info!("Peer doesn't exist! Creating new peer.");
            // Get peerid from the server
            let msg = messages::get_peerid_query();
            let response = send_peerid_query(msg).await;
            peer.peerid = response;
            // Set the id obtained as a response to the peer id
            Peer::save_peer(&peer);
        }
        // We need to check if our saved socket is available. If not we need to change it.
        // Query the server
        info!("{:?}", peer.socketmap);
        //TODO
        let socket = peer.get_socket().await;
        let msg = messages::get_sockets_query(peer.peerid, socket);
        let response = send_sockets_query(msg).await;
        for (id, socket) in response {
            peer.socketmap.insert(id, socket);
        }
        Peer::save_peer(&peer);

        return peer;
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

    pub async fn get_socket(&mut self) -> String {
        let mut current_port;
        if self.port.is_none() {
            info!("Port is not set. Setting new port...");
            current_port = self.set_new_port().await;
        } else {
            info!("Checking port availability...");
            current_port = self.port.clone().unwrap();
            if !scan_port(current_port.parse::<u16>().unwrap() as u16) {
                info!("Current port unavailable. Setting new port...");
                current_port = self.set_new_port().await;
            } else {
                info!("Current port available.")
            }
        }
        self.port = Some(current_port.clone());
        let mut result = String::new();
        result.push_str(&self.address);
        result.push_str(":");
        result.push_str(&current_port);
        return result;
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
