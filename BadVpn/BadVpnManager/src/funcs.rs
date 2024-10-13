use std::{fs, io};
use std::io::Write;
use std::net::TcpListener;
use std::process::Command;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

pub fn is_port_available(port: usize) -> Result<bool, bool> {
    match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn add_port(port: usize) -> Result<(), io::Error> {
    let service_file_content = format!(r#"
[Unit]
Description=BadVpn{}
After=network.target

[Service]
Restart=always
Type=simple
ExecStart=/opt/rustymanager/badvpn --listen-addr 127.0.0.1:{} --max-clients 1000 --max-connections-for-client 1000 --client-socket-sndbuf 0 --udp-mtu 9000

[Install]
WantedBy=multi-user.target
"#, port, port);

    let service_file_path = format!("/etc/systemd/system/badvpn{}.service", port);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(service_file_path).unwrap();

    file.write_all(service_file_content.as_bytes()).unwrap();

    let commands = [
        "systemctl daemon-reload".to_string(),
        format!("systemctl enable badvpn{}.service", port),
        format!("systemctl start badvpn{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}

pub fn del_port(port: usize) -> Result<(), io::Error> {
    let commands = [
        format!("systemctl disable badvpn{}.service", port),
        format!("systemctl stop badvpn{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    fs::remove_file(format!("/etc/systemd/system/badvpn{}.service", port)).unwrap();
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) proxy: HttpProxy,
    pub(crate) stunnel: Stunnel,
    pub(crate) badvpn: BadVpn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadVpn {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpProxy {
    pub(crate) enabled: Option<bool>,
    pub(crate) port: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stunnel {
    pub(crate) enabled: Option<bool>,
    pub(crate) port: Option<u16>,
}

pub fn add_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), io::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT * FROM connections LIMIT 1").unwrap();
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: HttpProxy {
                enabled: row.get(1).unwrap(),
                port: row.get(2).unwrap(),
            },
            stunnel: Stunnel {
                enabled: row.get(3).unwrap(),
                port: row.get(4).unwrap(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(5).ok(), 
            },
        })
    }).unwrap().collect::<Result<_, _>>().unwrap();

    match connections.first() {
        Some(conn) => {
            let mut ports = conn.badvpn.ports.clone().unwrap_or_default();
            if !ports.is_empty() {
                ports.push('|');
            }
            ports.push_str(&port.to_string());
            sqlite_conn.execute("UPDATE connections SET badvpn_ports = ? WHERE id = 1", params![ports]).unwrap();
            Ok(())
        },
        None => {
            sqlite_conn.execute(
                "INSERT INTO connections (badvpn_ports) VALUES (?)", 
                         params![Some(port.to_string()).unwrap()]
            ).unwrap();
            Ok(())
        }
    }
}

pub fn del_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), io::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT * FROM connections LIMIT 1").unwrap();
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: HttpProxy {
                enabled: row.get(1).unwrap(),
                port: row.get(2).unwrap(),
            },
            stunnel: Stunnel {
                enabled: row.get(3).unwrap(),
                port: row.get(4).unwrap(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(5).ok(), 
            },
        })
    }).unwrap().collect::<Result<_, _>>().unwrap();

    match connections.first() {
        Some(conn) => {
            if let Some(ports) = conn.badvpn.ports.clone() {
                let mut ports_vec: Vec<&str> = ports.split('|').collect();
                if let Some(pos) = ports_vec.iter().position(|&p| p == &port.to_string()) {
                    ports_vec.remove(pos);
                    let new_ports = ports_vec.join("|");
                    sqlite_conn.execute("UPDATE connections SET badvpn_ports = ? WHERE id = 1", params![new_ports]).unwrap();
                }
            }
            Ok(())
        },
        None => Err(io::Error::new(io::ErrorKind::NotFound, "No connections found")),
    }
}

fn run_command(command: String) -> &'static str {
    let exec = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .expect("error on run command");

    if !exec.status.success() {
        return "error on run command";
    }
    "success"
}
