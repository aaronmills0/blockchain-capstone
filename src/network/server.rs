use local_ip_address::local_ip;
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

pub async fn setup_server() {
    let local_ip = local_ip().unwrap();
    let address = local_ip.to_string();

    let socket = address + ":6780";
    let listener = TcpListener::bind(socket).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
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

    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);

        // Respond with an error
        let response = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response).await.unwrap();
    }
}
