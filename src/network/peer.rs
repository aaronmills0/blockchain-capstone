use local_ip_address::local_ip;
use log::info;
use mini_redis::{Connection, Frame};
use std::io;
use tokio::net::{TcpListener, TcpStream};

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

    let (socket, _) = listener.accept().await.unwrap();

    info!("{:?}", &socket);
    // A new task is spawned for each inbound socket. The socket is
    // moved to the new task and processed there.
    tokio::spawn(async move {
        process(socket).await;
    });
}

async fn process(socket: TcpStream) {
    // The `Connection` lets us read/write redis **frames** instead of
    // byte streams. The `Connection` type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    if let Some(frame) = connection.read_frame().await.unwrap() {
        info!("GOT: {:?}", frame);
    }
}
