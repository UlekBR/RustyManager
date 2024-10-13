use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) http_proxy_enabled: bool,
    pub(crate) http_proxy_port: u16,
}

const BUFLEN: usize = 4096 * 16;
const DEFAULT_HOST: &str = "127.0.0.1:22";
const WS_RESPONSE: &[u8] = b"HTTP/1.1 101 @RustyManager\r\n\r\n";
const RESPONSE: &[u8] = b"HTTP/1.1 200 @RustyManager\r\n\r\n";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open("/opt/rustymanager/db")?;

    let connection = get_connection_settings(&conn)?;

    if let Some(connections) = connection {
        if connections.http_proxy_enabled {
            let addr = format!("[::]:{}", connections.http_proxy_port);
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

fn get_connection_settings(conn: &Connection) -> Result<Option<Connections>> {
    let mut stmt = conn.prepare("SELECT http_proxy_enabled, http_proxy_port FROM connections LIMIT 1")?;
    let connection_iter = stmt.query_map(params![], |row| {
        Ok(Connections {
            http_proxy_enabled: row.get(0)?,
            http_proxy_port: row.get(1)?,
        })
    })?;

    for connection in connection_iter {
        return Ok(Some(connection?));
    }
    Ok(None)
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
