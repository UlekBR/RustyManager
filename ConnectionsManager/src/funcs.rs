use std::{fs, io};
use std::io::Write;
use std::net::TcpListener;
use std::process::Command;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) proxy: RustyProxy,
    pub(crate) stunnel: Stunnel,
    pub(crate) badvpn: BadVpn,
    pub(crate) checkuser: CheckUser,
    pub(crate) openvpn: OpenVpn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustyProxy {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stunnel {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadVpn {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckUser {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenVpn {
    pub(crate) port: Option<String>,
}


pub fn is_port_available(port: usize) -> Result<bool, bool> {
    match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}



pub fn add_proxy_port(port: usize, status: Option<String>) -> Result<(), io::Error> {
    
    let mut command = format!("/opt/rustymanager/rustyproxy --port {}", port);
    if status.is_some() {
        command = format!("{} --status {}", command, status.unwrap_or("@RustyProxy".to_string()));
    }
    let service_file_content = format!(r#"
[Unit]
Description=RustyProxy{}
After=network.target

[Service]
LimitNOFILE=infinity
LimitNPROC=infinity
LimitMEMLOCK=infinity
LimitSTACK=infinity
LimitCORE=infinity
LimitAS=infinity
LimitRSS=infinity
LimitCPU=infinity
LimitFSIZE=infinity
Type=simple
ExecStart={}
Restart=always

[Install]
WantedBy=multi-user.target
"#, port, command);

    let service_file_path = format!("/etc/systemd/system/rustyproxy{}.service", port);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(service_file_path)?;

    file.write_all(service_file_content.as_bytes())?;

    let commands = [
        "systemctl daemon-reload".to_string(),
        format!("systemctl enable rustyproxy{}.service", port),
        format!("systemctl start rustyproxy{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}
pub fn del_proxy_port(port: usize) -> Result<(), io::Error> {
    let commands = [
        format!("systemctl disable rustyproxy{}.service", port),
        format!("systemctl stop rustyproxy{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    fs::remove_file(format!("/etc/systemd/system/rustyproxy{}.service", port))?;
    Ok(())
}

fn get_stunnel_service() -> String {
    let os_info = fs::read_to_string("/etc/os-release").expect("Failed to read /etc/os-release");

    let mut os_name = String::new();

    for line in os_info.lines() {
        if line.starts_with("ID=") {
            os_name = line.trim_start_matches("ID=").trim_matches('"').to_string();
        }
    }

    if os_name == "ubuntu" || os_name == "debian" {
        "stunnel4".to_string()
    } else if os_name == "almalinux" || os_name == "rockylinux" {
        "stunnel".to_string()
    } else {
        "null".to_string()
    }
}

pub fn add_stunnel_port(port: usize, ipv6: bool) -> std::result::Result<(), io::Error> {
    let port_str = port.to_string();
    let prefix = if ipv6 { ":::" } else { "0.0.0.0:" };
    let stunnel_name = get_stunnel_service();
    let commands = [
        format!("grep -qE '^(::|0\\.0\\.0\\.0:)?{port_str}$' /etc/stunnel/stunnel.conf || echo '\naccept = {prefix}{port_str}' >> /etc/stunnel/stunnel.conf"),
        format!("systemctl is-active --quiet {} && systemctl restart {} || systemctl start {}", stunnel_name, stunnel_name, stunnel_name),
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}
pub fn del_stunnel_port(port: usize) -> std::result::Result<(), io::Error> {
    let port_str = port.to_string();
    let stunnel_name = get_stunnel_service();
    let commands = [
        format!("sed -i '/{port_str}/d' /etc/stunnel/stunnel.conf"),
        format!("grep -q 'accept' /etc/stunnel/stunnel.conf  && systemctl restart {} || systemctl stop {}", stunnel_name, stunnel_name)
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}

pub fn add_badvpn_port(port: usize) -> std::result::Result<(), io::Error> {
    let service_file_content = format!(r#"
[Unit]
Description=BadVpn{}
After=network.target

[Service]
LimitNOFILE=infinity
LimitNPROC=infinity
LimitMEMLOCK=infinity
LimitSTACK=infinity
LimitCORE=infinity
LimitAS=infinity
LimitRSS=infinity
LimitCPU=infinity
LimitFSIZE=infinity
Restart=always
Type=simple
ExecStart=/opt/rustymanager/badvpn --listen-addr [::]:{} --max-clients 1000 --max-connections-for-client 1000 --client-socket-sndbuf 0 --udp-mtu 9000

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
        format!("systemctl start badvpn{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}
pub fn del_badvpn_port(port: usize) -> std::result::Result<(), io::Error> {
    let commands = [
        format!("systemctl disable badvpn{}.service", port),
        format!("systemctl stop badvpn{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    fs::remove_file(format!("/etc/systemd/system/badvpn{}.service", port))?;
    Ok(())
}
pub fn add_checkuser_port(port: usize) -> std::result::Result<(), io::Error> {
    let service_file_content = format!(r#"
[Unit]
Description=Checkuser{}
After=network.target

[Service]
LimitNOFILE=infinity
LimitNPROC=infinity
LimitMEMLOCK=infinity
LimitSTACK=infinity
LimitCORE=infinity
LimitAS=infinity
LimitRSS=infinity
LimitCPU=infinity
LimitFSIZE=infinity
Restart=always
Type=simple
ExecStart=/opt/rustymanager/checkuser --port {}

[Install]
WantedBy=multi-user.target
"#, port, port);

    let service_file_path = format!("/etc/systemd/system/checkuser{}.service", port);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(service_file_path)?;

    file.write_all(service_file_content.as_bytes())?;

    let commands = [
        "systemctl daemon-reload".to_string(),
        format!("systemctl enable checkuser{}.service", port),
        format!("systemctl start checkuser{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}
pub fn del_checkuser_port(port: usize) -> std::result::Result<(), io::Error> {
    let commands = [
        format!("systemctl disable checkuser{}.service", port),
        format!("systemctl stop checkuser{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    fs::remove_file(format!("/etc/systemd/system/checkuser{}.service", port))?;
    Ok(())
}

pub fn enable_openvpn(port: usize) -> std::result::Result<(), io::Error> {
    let port_str = port.to_string();
    let commands = [
        format!("sed -i 's/^port [^ ]\\+/port {}/g' /etc/openvpn/server.conf", port_str),
        "systemctl start openvpn".to_string(),
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}
pub fn disable_openvpn() -> std::result::Result<(), io::Error> {
    let commands = [
        "sed -i 's/^port [^ ]\\+/port none/g' /etc/openvpn/server.conf".to_string(),
        "systemctl stop openvpn".to_string()
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}

pub fn add_proxy_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT * FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(1).ok(),
            },
            stunnel: Stunnel {
                ports: row.get::<_, String>(2).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(3).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(4).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(5).ok(),
            },
        })
    })?.collect::<Result<_, _>>()?;

    match connections.first() {
        Some(conn) => {
            let mut ports = conn.proxy.ports.clone().unwrap_or_default();
            if !ports.is_empty() {
                ports.push('|');
            }
            ports.push_str(&port.to_string());
            sqlite_conn.execute("UPDATE connections SET proxy_ports = ? WHERE id = 1", params![ports])?;
            Ok(())
        },
        None => {
            sqlite_conn.execute(
                "INSERT INTO connections (proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (?, NULL, NULL, NULL, NULL)",
                params![port.to_string()]
            )?;
            Ok(())
        }
    }
}

pub fn add_stunnel_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT * FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(1).ok(),
            },
            stunnel: Stunnel {
                ports: row.get::<_, String>(2).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(3).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(4).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(5).ok(),
            },
        })
    })?.collect::<Result<_, _>>()?;

    match connections.first() {
        Some(conn) => {
            let mut ports = conn.stunnel.ports.clone().unwrap_or_default();
            if !ports.is_empty() {
                ports.push('|');
            }
            ports.push_str(&port.to_string());
            sqlite_conn.execute("UPDATE connections SET stunnel_ports = ? WHERE id = 1", params![ports])?;
            Ok(())
        },
        None => {
            sqlite_conn.execute(
                "INSERT INTO connections (proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, ?, NULL, NULL, NULL)",
                params![port.to_string()]
            )?;
            Ok(())
        }
    }
}


pub fn add_badvpn_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT * FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(1).ok(),
            },
            stunnel: Stunnel {
                ports: row.get::<_, String>(2).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(3).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(4).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(5).ok(),
            },
        })
    })?.collect::<Result<_, _>>()?;

    match connections.first() {
        Some(conn) => {
            let mut ports = conn.badvpn.ports.clone().unwrap_or_default();
            if !ports.is_empty() {
                ports.push('|');
            }
            ports.push_str(&port.to_string());
            sqlite_conn.execute("UPDATE connections SET badvpn_ports = ? WHERE id = 1", params![ports])?;
            Ok(())
        },
        None => {
            sqlite_conn.execute(
                "INSERT INTO connections (proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, NULL, ?, NULL, NULL)",
                params![port.to_string()]
            )?;
            Ok(())
        }
    }
}

pub fn add_checkuser_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT * FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(1).ok(),
            },
            stunnel: Stunnel {
                ports: row.get::<_, String>(2).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(3).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(4).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(5).ok(),
            },
        })
    })?.collect::<Result<_, _>>()?;

    match connections.first() {
        Some(conn) => {
            let mut ports = conn.checkuser.ports.clone().unwrap_or_default();
            if !ports.is_empty() {
                ports.push('|');
            }
            ports.push_str(&port.to_string());
            sqlite_conn.execute("UPDATE connections SET checkuser_ports = ? WHERE id = 1", params![ports])?;
            Ok(())
        },
        None => {
            sqlite_conn.execute(
                "INSERT INTO connections (proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, NULL, NULL, ?, NULL)",
                params![port.to_string()]
            )?;
            Ok(())
        }
    }
}

pub fn add_openvpn_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let port_str = port.to_string();
    let exists = sqlite_conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM connections LIMIT 1)",
        [],
        |row| row.get::<_, bool>(0),
    )?;
    if exists {
        sqlite_conn.execute(
            "UPDATE connections SET openvpn_port = ? WHERE id = 1",
            params![port_str],
        )?;
    } else {
        sqlite_conn.execute(
            "INSERT INTO connections (proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, NULL, NULL, NULL, ?)",
            params![port_str],
        )?;
    }
    Ok(())
}


pub fn del_proxy_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT proxy_ports FROM connections LIMIT 1")?;
    let connections: Vec<String> = stmt.query_map(params![], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<_, _>>()?;

    if let Some(existing_ports) = connections.first() {
        let mut ports_vec: Vec<String> = existing_ports.trim().split('|').map(String::from).collect();
        ports_vec.retain(|p| p != &port.to_string());
        let new_ports = ports_vec.join("|");
        sqlite_conn.execute("UPDATE connections SET proxy_ports = ? WHERE id = 1", params![new_ports])?;
        Ok(())
    } else {
        Err(rusqlite::Error::UnwindingPanic)
    }
}

pub fn del_stunnel_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT stunnel_ports FROM connections LIMIT 1")?;
    let connections: Vec<String> = stmt.query_map(params![], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<_, _>>()?;

    if let Some(existing_ports) = connections.first() {
        let mut ports_vec: Vec<String> = existing_ports.trim().split('|').map(String::from).collect();
        ports_vec.retain(|p| p != &port.to_string());
        let new_ports = ports_vec.join("|");
        sqlite_conn.execute("UPDATE connections SET stunnel_ports = ? WHERE id = 1", params![new_ports])?;
        Ok(())
    } else {
        Err(rusqlite::Error::UnwindingPanic)
    }
}

pub fn del_badvpn_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT badvpn_ports FROM connections LIMIT 1")?;
    let connections: Vec<String> = stmt.query_map(params![], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<_, _>>()?;

    if let Some(existing_ports) = connections.first() {
        let mut ports_vec: Vec<String> = existing_ports.trim().split('|').map(String::from).collect();
        ports_vec.retain(|p| p != &port.to_string());
        let new_ports = ports_vec.join("|");
        sqlite_conn.execute("UPDATE connections SET badvpn_ports = ? WHERE id = 1", params![new_ports])?;
        Ok(())
    } else {
        Err(rusqlite::Error::UnwindingPanic)
    }
}

pub fn del_checkuser_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT checkuser_ports FROM connections LIMIT 1")?;
    let connections: Vec<String> = stmt.query_map(params![], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<_, _>>()?;

    if let Some(existing_ports) = connections.first() {
        let mut ports_vec: Vec<String> = existing_ports.trim().split('|').map(String::from).collect();
        ports_vec.retain(|p| p != &port.to_string());
        let new_ports = ports_vec.join("|");
        sqlite_conn.execute("UPDATE connections SET checkuser_ports = ? WHERE id = 1", params![new_ports])?;
        Ok(())
    } else {
        Err(rusqlite::Error::UnwindingPanic)
    }
}

pub fn del_openvpn_port_in_db(sqlite_conn: &Connection) -> Result<(), rusqlite::Error> {
    sqlite_conn.execute("UPDATE connections SET openvpn_port = '' WHERE id = 1", [])?;
    Ok(())
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
