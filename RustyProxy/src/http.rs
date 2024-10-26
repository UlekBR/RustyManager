use std::error::Error;
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};


pub async fn start_proxy_http(addr: String, ws_response: Arc<String>, response: Arc<String>) -> Result<(), Box<dyn Error>>  {
    let listener = TcpListener::bind(&addr).await?;
    loop {
        let (client_socket, _) = listener.accept().await?;
        let ws_response_clone = Arc::clone(&ws_response);
        let response_clone = Arc::clone(&response);

        tokio::spawn(async move {
            if let Err(e) = handle_client(client_socket, ws_response_clone, response_clone).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_client(
    mut client_socket: TcpStream,
    ws_response: Arc<String>,
    response: Arc<String>
) -> Result<(), Box<dyn Error>> {
    client_socket.write_all(ws_response.as_bytes()).await?;

    let mut client_buffer = vec![0; 4096];
    let n = client_socket.read(&mut client_buffer).await?;
    let payload = &client_buffer[..n];
    if let Ok(mut payload_str) = String::from_utf8(payload.to_vec()) {
        payload_str = payload_str.to_lowercase();
        if !payload_str.contains("upgrade: websocket") && !payload_str.contains("upgrade: ws") {
            client_socket.write_all(response.as_bytes()).await?;
        }
    }

    connect_target("127.0.0.1:22", &mut client_socket).await?;
    Ok(())
}

async fn connect_target(host: &str, client_socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    match TcpStream::connect(host).await {
        Ok(mut target_socket) => {
            do_forwarding(client_socket, &mut target_socket).await?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Error connecting to target {}: {}", host, e);
            Err(Box::new(e))
        }
    }
}

async fn do_forwarding(client_socket: &mut TcpStream, target_socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let (mut client_reader, mut client_writer) = io::split(client_socket);
    let (mut target_reader, mut target_writer) = io::split(target_socket);
    tokio::select! {
        _ = io::copy(&mut client_reader, &mut target_writer) => {}
        _ = io::copy(&mut target_reader, &mut client_writer) => {}
    }

    Ok(())
}