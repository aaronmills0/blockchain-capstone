use local_ip_address::local_ip;
use mini_redis::{client, Result};

pub async fn setup_client() -> Result<()> {
    let local_ip = local_ip().unwrap();
    let address = local_ip.to_string();
    let socket = address + ":6780";
    println!("{}", &socket);
    // The 'await' expression suspends the operation until it is ready to be processed. It continues to the next operation.
    let mut client = client::connect(&socket).await?;
    println!("Client successfully connected to {}", socket);

    client.set("hello", "world".into()).await?;

    let result = client.get("hello").await?;

    println!("received {:?}", result);

    Ok(())
}
