mod http;
mod https;

use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::{arg, Parser};
use crate::http::start_proxy_http;
use crate::https::start_proxy_https;

#[derive(Parser)]
#[command(name = "RustyProxy")]
#[command(about = "a simple proxy")]
struct Args {
    #[arg(long, default_value = "http")]
    mode: String,
    #[arg(long, default_value = "80")]
    port: u16,
    #[arg(long, default_value = "@RustyProxy")]
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let addr = format!("[::]:{}", args.port);
    println!("Proxy server listening on {}", addr);

    let ws_response = Arc::new(format!("HTTP/1.1 101 {}\r\n\r\n", args.status));
    let response = Arc::new(format!("HTTP/1.1 200 {}\r\n\r\n", args.status));
    if args.mode == "http" {
        start_proxy_http(addr, ws_response, response).await
    } else {
        start_proxy_https(addr, ws_response).await
    }
}

