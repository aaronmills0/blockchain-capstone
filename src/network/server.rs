use crate::network::{decoder, messages};
use local_ip_address::local_ip;
use log::{error, info, warn};
use mini_redis::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
};

use super::peer::Peer;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Server {
    peer: Peer,
    next_peerid: u32,
}

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Vec<String>>,
        payload: Option<Vec<String>>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

impl Server {
    pub fn new() -> Server {
        return Server {
            peer: Peer {
                address: local_ip()
                    .expect("Failed to obtain local ip address")
                    .to_string(),
                peerid: 1,
                ports: vec![
                    String::from("57643"),
                    String::from("34565"),
                    String::from("32578"),
                    String::from("23564"),
                    String::from("13435"),
                ],
                ip_map: HashMap::new(),
                port_map: HashMap::new(),
            },
            next_peerid: 2,
        };
    }

    async fn server_manager(mut server: Server, mut rx: Receiver<Command>) {
        loop {
            let command = rx.recv().await.unwrap();
            match command {
                Command::Get { key, resp, payload } => {
                    if key.as_str() == "id_query" {
                        if payload.is_none() {
                            error!("Invalid command: missing payload");
                        } else {
                            let payload_vec = payload.unwrap();
                            if payload_vec.len() != 1 {
                                error!("Invalid command: payload is of unexpected size");
                            }
                            resp.send(Ok(Vec::from([server.next_peerid.to_string()])))
                                .ok();
                            server.next_peerid += 1;
                            Server::save_server(&server);
                        }
                    } else if key.as_str() == "ports_query" {
                        if payload.is_none() {
                            error!("Invalid command: missing payload");
                        } else {
                            let payload_vec = payload.unwrap();
                            if payload_vec.len() <= 2 {
                                error!("Invalid command: payload is of unexpected size");
                            }

                            let sourceid: u32 = payload_vec[0].parse().unwrap();
                            let ip = payload_vec[1].clone();

                            // Update the server ip_map and port_map
                            server.peer.ip_map.insert(sourceid, ip.clone());
                            server
                                .peer
                                .port_map
                                .insert(ip.clone(), payload_vec[2..].to_vec());

                            let response_vector = vec![
                                serde_json::to_string(&server.peer.ip_map)
                                    .expect("Failed to serialize ip map"),
                                serde_json::to_string(&server.peer.port_map)
                                    .expect("Failed to serialize port map"),
                            ];
                            resp.send(Ok(response_vector)).ok();
                            Server::save_server(&server);
                        }
                    }
                }
            }
        }
    }

    pub async fn launch() {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let server: Server;
        // First load the server peer from system/peer.json if it exists.
        if Path::new(&("system".to_owned() + slash + "server.json")).exists() {
            server = Server::load_server();
        } else {
            server = Server::new();
            // Binding with port 0 tells the OS to find a suitable port. We will save this port.
            Server::save_server(&server);
        }

        let (tx, rx) = mpsc::channel(32);

        let local_ip = local_ip().unwrap();
        let address = local_ip.to_string();
        for p in &server.peer.ports {
            let ip = address.clone();
            let port = p.clone();
            let tx_copy = tx.clone();
            tokio::spawn(async move { Server::listen(ip, port, tx_copy).await });
        }

        Server::server_manager(server, rx).await;
    }

    async fn listen(ip: String, port: String, tx: Sender<Command>) {
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
                Server::process_connection(stream, socket.to_string(), tx_clone).await;
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
                        info!("GOT: {:?}", frame);
                        let (command, sourceid, destid) = decoder::decode_command(&frame);

                        if destid != 1 {
                            warn!("Destination id does not match server id: {}", destid);
                            return;
                        }
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let mut payload_vec = Vec::new();
                        if command == "id_query" {
                            payload_vec.push(socket.clone());
                            cmd = Command::Get {
                                key: command,
                                resp: resp_tx,
                                payload: Some(payload_vec),
                            };
                            tx.send(cmd).await.ok();
                            let result = resp_rx.await;
                            let peerid = result.unwrap().unwrap()[0].parse().unwrap();
                            let response = messages::get_peerid_response(peerid);
                            connection.write_frame(&response).await.ok();
                        } else if command == "ports_query" {
                            let mut ports = decoder::decode_ports_query(&frame);
                            if ports.is_empty() {
                                error!("No ports found when decoding ports query");
                                panic!();
                            }

                            payload_vec.push(sourceid.to_string());
                            payload_vec.push(ip.clone());
                            payload_vec.append(&mut ports);
                            cmd = Command::Get {
                                key: command,
                                resp: resp_tx,
                                payload: Some(payload_vec),
                            };
                            tx.send(cmd).await.ok();

                            let result = resp_rx.await.unwrap().unwrap();
                            if result.is_empty() {
                                error!("Empty result from server manager");
                                panic!();
                            }
                            let ip_map_json = result[0].to_owned();
                            let port_map_json = result[1].to_owned();
                            info!("Sending ip_map: {:?}", ip_map_json);
                            info!("Sending port_map: {:?}", port_map_json);

                            let response =
                                messages::get_ports_response(ip_map_json, port_map_json, sourceid);
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

    pub fn save_server(server: &Server) {
        let mut map = Map::new();
        let server_json = serde_json::to_value(server);

        if server_json.is_err() {
            error!("Failed to serialize server peer");
            panic!();
        }

        let mut json = server_json.unwrap();

        map.insert(String::from("server"), json);

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

        let file_name: &str = &format!("server.json");

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

    pub fn load_server() -> Server {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let data = fs::read_to_string("system".to_owned() + slash + "server.json");
        if data.is_err() {
            error!("Failed to load file. {:?}", data.err());
            panic!();
        }
        let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
        let server = serde_json::from_value(json.get("server").unwrap().to_owned());
        return server.unwrap();
    }
}
