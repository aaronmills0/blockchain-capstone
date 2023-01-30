use crate::network::decoder;
use crate::network::messages;
use local_ip_address::local_ip;
use log::{error, info, warn};
use mini_redis::{Connection, Frame};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::{env, fs};
use std::{fs::File, io, path::Path};
use tokio::net::{TcpListener, TcpStream};

static ARCHIVE_SERVER_ADDR: &str = "192.168.0.12:6780";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Peer {
    pub peerid: u32,
    pub socketmap: HashMap<u32, String>, // Socket addresses of neighbors
}

async fn send_peerid_query(msg: Frame) -> u32 {
    let stream = TcpStream::connect(&ARCHIVE_SERVER_ADDR).await.unwrap();
    info!("Successfully connected to {}", ARCHIVE_SERVER_ADDR);
    let mut connection = Connection::new(stream);

    connection.write_frame(&msg).await.ok();

    let mut response: u32 = 0;
    if let Some(frame) = connection.read_frame().await.unwrap() {
        response = decoder::decode_peerid_response(frame);
    }

    return response;
}

async fn send_sockets_query(msg: Frame) -> HashMap<u32, String> {
    let stream = TcpStream::connect(&ARCHIVE_SERVER_ADDR).await.unwrap();
    info!("Successfully connected to {}", ARCHIVE_SERVER_ADDR);
    let mut connection = Connection::new(stream);

    connection.write_frame(&msg).await.ok();

    let mut response: HashMap<u32, String> = HashMap::new();
    if let Some(frame) = connection.read_frame().await.unwrap() {
        response = decoder::decode_sockets_response(frame);
    }

    return response;
}

impl Peer {
    pub fn new() -> Peer {
        return Peer {
            peerid: 0,
            socketmap: HashMap::new(),
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
            // Get peerid from the archive server
            let msg = messages::get_peerid_query();
            let response = send_peerid_query(msg).await;
            peer.peerid = response;
            // Set the id obtained as a response to the peer id
            Peer::save_peer(&peer);
        }

        // Query the archive server
        info!("{:?}", peer.socketmap);
        let msg = messages::get_sockets_query(peer.peerid);
        let response = send_sockets_query(msg).await;
        for (id, socket) in response {
            peer.socketmap.insert(id, socket);
        }
        Peer::save_peer(&peer);

        return peer;
    }

    pub fn shutdown(peer: Peer) {
        Peer::save_peer(&peer);
        return;
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
