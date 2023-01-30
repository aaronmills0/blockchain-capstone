use crate::network::{decoder::decode_command, messages};
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
pub struct Archive {
    peer: Peer,
    next_peerid: u32,
}

enum Command {
    Get {
        key: String,
        resp: Responder<Option<String>>,
        payload: Option<Vec<String>>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

impl Archive {
    pub fn new() -> Archive {
        return Archive {
            peer: Peer {
                peerid: 1,
                socketmap: HashMap::new(),
            },
            next_peerid: 2,
        };
    }

    async fn archive_manager(mut archive: Archive, mut rx: Receiver<Command>) {
        let command = rx.recv().await.unwrap();
        match command {
            Command::Get { key, resp, payload } => match key.as_str() {
                "id_query" => {
                    if payload.is_none() {
                        error!("Invalid command: missing payload");
                    } else {
                        let payload_vec = payload.unwrap();
                        if payload_vec.len() != 1 {
                            error!("Invalid command: payload is of unexpected size");
                        }
                        archive
                            .peer
                            .socketmap
                            .insert(archive.next_peerid, payload_vec[0].clone());
                        resp.send(Ok(Some(String::from(archive.next_peerid.to_string()))))
                            .ok();
                        archive.next_peerid += 1;
                        Archive::save_archive(&archive);
                    }
                }
                "sockets_query" => {
                    if payload.is_none() {
                        error!("Invalid command: missing payload");
                    } else {
                        let payload_vec = payload.unwrap();
                        if payload_vec.len() != 2 {
                            error!("Invalid command: payload is of unexpected size");
                        }
                        let sourceid: u32 = payload_vec[0].parse().unwrap();
                        let socket = payload_vec[1].clone();
                        // Update the archive socketmap
                        archive.peer.socketmap.insert(sourceid, socket);
                        resp.send(Ok(Some(
                            serde_json::to_string(&archive.peer.socketmap).unwrap(),
                        )))
                        .ok();
                        Archive::save_archive(&archive);
                    }
                }
                _ => {}
            },
        }
    }

    pub async fn launch() {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let archive: Archive;
        // First load the archive peer from system/peer.json if it exists.
        if Path::new(&("system".to_owned() + slash + "archive.json")).exists() {
            archive = Archive::load_archive();
        } else {
            archive = Archive::new();
            Archive::save_archive(&archive);
        }

        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            Archive::archive_manager(archive, rx).await;
        });

        let local_ip = local_ip().unwrap();
        let address = local_ip.to_string();
        let socket = address + ":6780";

        let listener = TcpListener::bind(&socket).await.unwrap();
        info!("Successfully setup listener at {}", socket);

        // The archive server should now continuously listen and respond to queries
        // Each time it gets a request it should update its socketmap accordingly

        loop {
            let (stream, socket) = listener.accept().await.unwrap();

            info!("{:?}", &stream);
            // A new task is spawned for each inbound socket. The socket is
            // moved to the new task and processed there.
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                Archive::process_connection(stream, socket.to_string(), tx_clone).await;
            });
        }
    }

    async fn process_connection(stream: TcpStream, socket: String, tx: Sender<Command>) {
        let mut connection = Connection::new(stream);
        loop {
            match connection.read_frame().await {
                Ok(opt_frame) => {
                    if let Some(frame) = opt_frame {
                        let cmd;
                        info!("GOT: {:?}", frame);
                        let (command, sourceid, destid) = decode_command(&frame);

                        if destid != 1 {
                            warn!("Destination id does not match archive server id");
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
                            let peerid = result.unwrap().unwrap().unwrap().parse().unwrap();
                            let response = messages::get_peerid_response(peerid);
                            connection.write_frame(&response).await.ok();
                        } else if command == "sockets_query" {
                            payload_vec.push(sourceid.to_string());
                            payload_vec.push(socket.clone());
                            cmd = Command::Get {
                                key: command,
                                resp: resp_tx,
                                payload: Some(payload_vec),
                            };
                            tx.send(cmd).await.ok();
                            let result = resp_rx.await;
                            let socketmap: HashMap<u32, String> =
                                serde_json::from_str(&result.unwrap().unwrap().unwrap()).unwrap();
                            let response = messages::get_sockets_response(socketmap, sourceid);
                            connection.write_frame(&response).await.ok();
                        } else {
                            warn!("invalid command for archive server");
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

    pub fn save_archive(archive: &Archive) {
        let mut map = Map::new();
        let archive_json = serde_json::to_value(archive);

        if archive_json.is_err() {
            error!("Failed to serialize archive peer");
            panic!();
        }

        let mut json = archive_json.unwrap();

        map.insert(String::from("archive"), json);

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

        let file_name: &str = &format!("archive.json");

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

    pub fn load_archive() -> Archive {
        let slash = if env::consts::OS == "windows" {
            "\\"
        } else {
            "/"
        };
        let data = fs::read_to_string("system".to_owned() + slash + "archive.json");
        if data.is_err() {
            error!("Failed to load file. {:?}", data.err());
            panic!();
        }
        let json: Value = serde_json::from_str(&data.unwrap()).unwrap();
        let archive = serde_json::from_value(json.get("archive").unwrap().to_owned());
        return archive.unwrap();
    }
}
