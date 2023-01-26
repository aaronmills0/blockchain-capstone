use core::arch;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use bytes::Bytes;
use local_ip_address::local_ip;
use log::{error, info, warn};
use mini_redis::{Connection, Frame};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
        payload: Option<String>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

// notation for functions that return message typesis get_name_response()

fn get_peerid_response(next_peerid: u32) -> Frame {
    let mut response_vec: Vec<Frame> = Vec::new();
    let bulk = Frame::Integer(next_peerid as u64);
    response_vec.push(bulk);
    return Frame::Array(response_vec);
}

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
            Command::Get { key, resp, payload } => {
                if key == "id" {
                    if payload.is_none() {
                        error!("Invalid command")
                    } else {
                        archive
                            .peer
                            .socketmap
                            .insert(archive.next_peerid, payload.unwrap());
                        resp.send(Ok(Some(String::from(archive.next_peerid.to_string()))));
                        archive.next_peerid += 1;
                        Archive::save_archive(&archive);
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
        let mut archive: Archive = Archive::new();
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
                        info!("GOT: {:?}", frame);
                        // if-else to determine what the command actual is.
                        // For now just assuming it is peerid command
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let cmd = Command::Get {
                            key: "id".to_string(),
                            resp: resp_tx,
                            payload: Some(socket.clone()),
                        };
                        tx.send(cmd).await;
                        let peerid_result = resp_rx.await.unwrap().unwrap().unwrap().parse::<u32>().unwrap();

                        let response = get_peerid_response(peerid_result);
                        connection.write_frame(&response).await;
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
