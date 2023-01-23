use chrono::Local;
use local_ip_address::local_ip;
use log::{error, info, warn};
use mini_redis::{Connection, Frame};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::io::Write;
use std::{collections::HashSet, fs::File, io, path::Path};
use std::{env, fs};
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Serialize, Deserialize)]
pub struct Peer {
    pub neighbors: HashSet<String>, // Socket addresses of neighbors
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

pub async fn spawn_listener() {
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
            process(socket).await;
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

impl Peer {
    pub fn new() -> Peer {
        return Peer {
            neighbors: HashSet::new(),
        };
    }

    // async fn query_archive(&self, addr: &str) {
    //     let mut connection = Connection::new(TcpStream::connect(&addr).await.unwrap());
    //     let message = String::from("retreive full node IPs");
    //     let frame = Frame::Simple(message);

    //     connection.write_frame(&frame);

    //     if let Some(response) = connection.read_frame().await.unwrap() {
    //         // For ip_address in response...
    //     }
    // }

    pub fn save_peer(peer: Peer) {
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
        dirpath.push_str("/system");

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
        let data = fs::read_to_string("system/peer.json");
        if data.is_err() {
            error!("Failed to load file. {:?}", data.err());
            panic!();
        }
        let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
        let peer = serde_json::from_value(json.get("peer").unwrap().to_owned());
        return peer.unwrap();
    }
}
