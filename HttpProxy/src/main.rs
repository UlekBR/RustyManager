use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use mongodb::bson::{doc};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) proxy: HttpProxy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpProxy {
    pub(crate) enabled: bool,
    pub(crate) port: u16,
}

const BUFLEN: usize = 4096 * 16;
const DEFAULT_HOST: &str = "127.0.0.1:22";
const WS_RESPONSE: &[u8] = b"HTTP/1.1 101 @RustyManager\r\n\r\n";
const RESPONSE: &[u8] = b"HTTP/1.1 200 @RustyManager\r\n\r\n";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let uri = "mongodb://127.0.0.1:27017/";
    let client = Client::with_uri_str(uri).await?;
    let database = client.database("ssh");
    let collection: Collection<Connections> = database.collection("connections");

    let filter = doc! {};
    let connection = collection.find_one(filter).await?;

    if let Some(connections) = connection {
        let proxy: HttpProxy = connections.proxy;
        if proxy.enabled {
            let addr = format!("[::]:{}", proxy.port);
            let listener = TcpListener::bind(&addr).await?;
            println!("Proxy server listening on {}", addr);

            loop {
                let (client_socket, _) = listener.accept().await?;
                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_socket).await {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
        }
    } else {
        eprintln!("No connection settings found in the database.");
    }

    Ok(())
}


async fn handle_client(mut client_socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut client_buffer = vec![0; BUFLEN];
    client_socket.write_all(WS_RESPONSE).await?;

    let n = client_socket.read(&mut client_buffer).await?;
    let payload = &client_buffer[..n];
    if let Ok(payload_str) = String::from_utf8(payload.to_vec()) {
        if !payload_str.to_lowercase().contains("upgrade: websocket") && !payload_str.contains("upgrade: ws") {
            client_socket.write_all(RESPONSE).await?;
        }
    }

    connect_target(DEFAULT_HOST, &mut client_socket).await?;
    Ok(())
}


async fn connect_target(host: &str, client_socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut retries = 3;
    while retries > 0 {
        match TcpStream::connect(host).await {
            Ok(mut target_socket) => {
                do_forwarding(client_socket, &mut target_socket).await?;
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error connecting to target {}: {}", host, e);
                retries -= 1;
                if retries == 0 {
                    return Err(Box::new(e));
                }
            }
        }
    }
    Ok(())
}

async fn do_forwarding(client_socket: &mut TcpStream, target_socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut client_buf = vec![0; BUFLEN];
    let mut target_buf = vec![0; BUFLEN];

    loop {
        tokio::select! {
            result = client_socket.read(&mut client_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        target_socket.write_all(&client_buf[..n]).await?;
                    }
                    Err(e) => {
                        eprintln!("Forwarding error from client: {}", e);
                        break;
                    }
                }
            }
            result = target_socket.read(&mut target_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        client_socket.write_all(&target_buf[..n]).await?;
                    }
                    Err(e) => {
                        eprintln!("Forwarding error from target: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
