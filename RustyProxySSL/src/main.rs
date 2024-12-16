use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{rustls, TlsAcceptor, server::TlsStream};



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = format!("[::]:{}", get_port());

    let cert = load_certs(PathBuf::from(get_cert()).as_path())?;
    let key = load_key(PathBuf::from(get_key()).as_path())?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .map_err(|err| io::Error::new(ErrorKind::InvalidInput, err))?;

    let acceptor = TlsAcceptor::from(Arc::new(config));

    println!("Proxy server listening on {}", addr);
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (client_socket, _) = listener.accept().await?;
        let acceptor_clone = acceptor.clone();

        tokio::spawn(async move {
            match acceptor_clone.accept(client_socket).await {
                Ok(mut tls_stream) => {
                    if let Err(e) = connect_target("127.0.0.1:80", &mut tls_stream).await {
                        println!("Connection error: {}", e);
                    }
                }
                Err(e) => println!("TLS handshake error: {}", e),
            }
        });
    }
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


fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 80;

    for i in 1..args.len() {
        if args[i] == "--proxy-port" {
            if i + 1 < args.len() {
                port = args[i + 1].parse().unwrap_or(443);
            }
        }
    }

    port
}

fn get_cert() -> String {
    let args: Vec<String> = env::args().collect();
    let mut cert = String::from("/opt/rustymanager/ssl/cert.pem");

    for i in 1..args.len() {
        if args[i] == "--cert" {
            if i + 1 < args.len() {
                cert = args[i + 1].clone();
            }
        }
    }

    cert
}

fn get_key() -> String {
    let args: Vec<String> = env::args().collect();
    let mut key = String::from("/opt/rustymanager/ssl/key.pem");

    for i in 1..args.len() {
        if args[i] == "--key" {
            if i + 1 < args.len() {
                key = args[i + 1].clone();
            }
        }
    }

    key
}
