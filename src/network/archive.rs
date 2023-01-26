use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use local_ip_address::local_ip;
use log::{error, info, warn};
use mini_redis::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::net::{TcpListener, TcpStream};

use super::peer::Peer;

#[derive(Clone, Serialize, Deserialize)]
pub struct Archive {
    peer: Peer,
    next_peerid: u32,
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

        let local_ip = local_ip().unwrap();
        let address = local_ip.to_string();
        let socket = address + ":6780";

        let listener = TcpListener::bind(&socket).await.unwrap();
        info!("Successfully setup listener at {}", socket);

        // The archive server should now continuously listen and respond to queries
        // Each time it gets a request it should update its socketmap accordingly

        loop {
            let (socket, _) = listener.accept().await.unwrap();

            info!("{:?}", &socket);
            // A new task is spawned for each inbound socket. The socket is
            // moved to the new task and processed there.
            tokio::spawn(async move {
                Archive::process(socket).await;
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
