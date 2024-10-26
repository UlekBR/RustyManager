use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use tokio::io::{AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{rustls, TlsAcceptor, server::TlsStream};


pub async fn start_proxy_https(addr: String, ws_response: Arc<String>) -> Result<(), Box<dyn Error>>  {
    let certs = load_certs(PathBuf::from("/opt/rustymanager/ssl/cert.pem").as_path())?;
    let key = load_key(PathBuf::from("/opt/rustymanager/ssl/key.pem").as_path())?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(ErrorKind::InvalidInput, err))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (client_socket, _) = listener.accept().await?;
        let acceptor_clone = acceptor.clone();
        let ws_response_clone = Arc::clone(&ws_response);

        tokio::spawn(async move {
            match acceptor_clone.accept(client_socket).await {
                Ok(tls_stream) => {
                    if let Err(e) = handle_client(tls_stream, ws_response_clone).await {
                        println!("Connection error: {}", e);
                    }
                }
                Err(e) => println!("TLS handshake error: {}", e),
            }
        });
    }
}


async fn handle_client(
    mut client_socket: TlsStream<TcpStream>,
    ws_response: Arc<String>
) -> Result<(), Box<dyn Error>> {
    client_socket.write_all(ws_response.as_bytes()).await?;
    connect_target("127.0.0.1:22", &mut client_socket).await?;
    Ok(())
}

async fn connect_target(host: &str, client_socket: &mut TlsStream<TcpStream>) -> Result<(), Box<dyn Error>> {
    match TcpStream::connect(host).await {
        Ok(mut target_socket) => {
            do_forwarding(client_socket, &mut target_socket).await?;
            Ok(())
        }
        Err(e) => {
            println!("Error connecting to target {}: {}", host, e);
            Err(Box::new(e))
        }
    }
}

async fn do_forwarding(client_socket: &mut TlsStream<TcpStream>, target_socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let (mut client_reader, mut client_writer) = tokio::io::split(client_socket);
    let (mut target_reader, mut target_writer) = tokio::io::split(target_socket);
    tokio::select! {
        _ = tokio::io::copy(&mut client_reader, &mut target_writer) => {}
        _ = tokio::io::copy(&mut target_reader, &mut client_writer) => {}
    }

    Ok(())
}

fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_key(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    Ok(private_key(&mut BufReader::new(File::open(path)?))
        .unwrap()
        .ok_or(io::Error::new(
            ErrorKind::Other,
            "no private key found".to_string(),
        ))?)
}
