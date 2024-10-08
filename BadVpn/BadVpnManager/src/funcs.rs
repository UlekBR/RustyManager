use std::fmt::format;
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::process::Command;
use mongodb::bson::doc;
use mongodb::sync::{Collection, Database};
use serde::{Deserialize, Serialize};

pub fn is_port_avaliable(port: usize) -> Result<bool, bool> {
    match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(_) => {
            Ok(true)
        },
        Err(_) => {
            Ok(false)
        }
    }
}

pub fn add_port(port: usize) -> Result<Ok(), Err()> {
    let service_file_content = format!(r#"
[Unit]
Description=BadVpn{}
After=network.target

[Service]
Restart=always
Type=simple
ExecStart=/opt/rustymanager/badvpn --loglevel warning --listen-addr 127.0.0.1:{}

[Install]
WantedBy=multi-user.target
"#, port, port);

    let service_file_path = format!("/etc/systemd/system/badvpn{}.service", port);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(service_file_path)?;

    file.write_all(service_file_content.as_bytes())?;

    let commands = [
        "systemctl daemon-reload".to_string(),
        format!("systemctl enable badvpn{}.service", port),
        format!("systemctl start badvpn{}.service", port)
    ];
    for command in commands {
        run_command(command)
    }
    Ok(())
}

pub fn del_port(port: usize) -> Result<Ok(), Err()> {
    let commands = [
        format!("systemctl disable badvpn{}.service", port),
        format!("systemctl stop badvpn{}.service", port)
    ];
    for command in commands {
        run_command(command)
    }
    fs::remove_file(format!("/etc/systemd/system/badvpn{}.service", port))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) proxy: HttpProxy,
    pub(crate) badvpn: BadVpn
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadVpn {
    pub(crate) ports: Vec<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpProxy {
    pub(crate) enabled: bool,
    pub(crate) port: u16,
}

pub fn add_port_in_db(database: Database, port: usize) -> Result<Ok(), Err()> {
    let collection: Collection<Connections> = database.collection("connections");

    let filter = doc! {};
    let connections = collection.find_one(filter.clone()).run().unwrap();

    match connections {
        Some(mut conn) => {
            conn.badvpn.ports.push(port);
            collection.replace_one(filter, conn.clone()).run().unwrap();
            Ok(())
        },
        None => {
            let new_connection = Connections {
                proxy: HttpProxy {
                    enabled: false,
                    port: 0,
                },
                badvpn: BadVpn {
                    ports: Vec::from([port])
                }
            };
            collection.insert_one(new_connection).run().unwrap();
            Ok(())
        }
    }
}

pub fn del_port_in_db(database: Database, port: usize) -> Result<(), Err()> {
    let collection: Collection<Connections> = database.collection("connections");

    let filter = doc! {};
    let connections = collection.find_one(filter.clone()).run().unwrap();

    match connections {
        Some(mut conn) => {
            if let Some(pos) = conn.badvpn.ports.iter().position(|&p| p == port) {
                conn.badvpn.ports.remove(pos);
                collection.replace_one(filter, conn.clone()).run().unwrap();
            }
            Ok(())
        },
        None => {
            Err(())
        }
    }
}


fn run_command(command: String) -> &'static str {
    let exec = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .expect("error on run command");

    if !exec.status.success() {
        return "error on run command"
    }
    "sucess"
}