use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use clap::Parser;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, private_key};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{rustls, TlsAcceptor, server::TlsStream};



#[derive(Parser)]
#[command(name = "RustyProxySSL")]
#[command(about = "a simple ssl proxy")]
struct Args {
    #[arg(long, default_value = "443")]
    proxy_port: u16,
    #[arg(long, default_value = "/opt/rustymanager/ssl/cert.pem")]
    cert: String,
    #[arg(long, default_value = "/opt/rustymanager/ssl/key.pem")]
    key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let addr = format!("[::]:{}", args.proxy_port);

    let cert = load_certs(PathBuf::from(args.cert).as_path())?;
    let key = load_key(PathBuf::from(args.key).as_path())?;

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
