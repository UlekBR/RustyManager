use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::time::Duration;
use std::{env, thread};

fn main() {
    // Iniciando o proxy
    let port = get_port();
    let listener = TcpListener::bind(format!("[::]:{}", port)).unwrap();
    println!("Iniciando serviço na porta: {}", port);
    start_http(listener);
}

fn start_http(listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(mut client_stream) => {
                thread::spawn(move || {
                    handle_client(&mut client_stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

fn handle_client(client_stream: &mut TcpStream) {
    let status = get_status();
    if client_stream.write_all(format!("HTTP/1.1 101 {}\r\n\r\n", status).as_bytes()).is_err() {
        return;
    }

    match peek_stream(&client_stream) {
        Ok(data_str) => {
            if data_str.contains("HTTP") {
                let _ = client_stream.read(&mut vec![0; 1024]);
                let payload_str = data_str.to_lowercase();
                if payload_str.contains("websocket") || payload_str.contains("ws") {
                    if client_stream.write_all(format!("HTTP/1.1 200 {}\r\n\r\n", status).as_bytes()).is_err() {
                        return;
                    }
                }
            }
        }
        Err(..) => return,
    }


    let mut addr_proxy = "0.0.0.0:22";

    let (tx, rx) = mpsc::channel();

    let clone_client = client_stream.try_clone().unwrap();
    let read_handle = thread::spawn(move || {
        let result = peek_stream(&clone_client);
        tx.send(result).ok();
    });

    let read_result = rx.recv_timeout(Duration::from_secs(1));

    match read_result {
        Ok(Ok(data_str)) => {
            if !data_str.contains("SSH") {
                addr_proxy = "0.0.0.0:1194";
            }
        }
        Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {
            read_handle.thread().unpark()
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            read_handle.thread().unpark()
        }
    }
    let _ = read_handle.thread();


    let server_connect = TcpStream::connect(&addr_proxy);
    if server_connect.is_err() {
        return;
    }

    let server_stream = server_connect.unwrap();

    let (mut client_read, mut client_write) = (client_stream.try_clone().unwrap(), client_stream.try_clone().unwrap());
    let (mut server_read, mut server_write) = (server_stream.try_clone().unwrap(), server_stream);

    thread::spawn(move || {
        transfer_data(&mut client_read, &mut server_write);
    });

    thread::spawn(move || {
        transfer_data(&mut server_read, &mut client_write);
    });
}

fn transfer_data(read_stream: &mut TcpStream, write_stream: &mut TcpStream) {
    let mut buffer = [0; 2048];
    loop {
        match read_stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                if write_stream.write_all(&buffer[..n]).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    write_stream.shutdown(Shutdown::Both).ok();
}

fn peek_stream(read_stream: &TcpStream) -> Result<String, Error> {
    let mut peek_buffer = vec![0; 1024];
    let bytes_peeked = read_stream.peek(&mut peek_buffer)?;
    let data = &peek_buffer[..bytes_peeked];
    let data_str = String::from_utf8_lossy(data);
    Ok(data_str.to_string())
}

fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 80;

    for i in 1..args.len() {
        if args[i] == "--port" {
            if i + 1 < args.len() {
                port = args[i + 1].parse().unwrap_or(80);
            }
        }
    }

    port
}

fn get_status() -> String {
    let args: Vec<String> = env::args().collect();
    let mut status = String::from("@RustyManager");

    for i in 1..args.len() {
        if args[i] == "--status" {
            if i + 1 < args.len() {
                status = args[i + 1].clone();
            }
        }
    }

    status
}
