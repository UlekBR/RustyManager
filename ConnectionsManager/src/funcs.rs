use std::{fs, io};
use std::io::Write;
use std::net::TcpListener;
use std::process::Command;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) proxy: RustyProxy,
    pub(crate) sslproxy: RustyProxySSL,
    pub(crate) badvpn: BadVpn,
    pub(crate) checkuser: CheckUser,
    pub(crate) openvpn: OpenVpn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustyProxy {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustyProxySSL {
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
LimitCORE=0
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

pub fn add_ssl_proxy_port(port: usize, cert: Option<&String>, key: Option<&String>) -> Result<(), io::Error> {

    let mut command = format!("/opt/rustymanager/rustyproxyssl --proxy-port {}", port);
    if cert.is_some() {
        command = format!("{} --cert {}", command, cert.unwrap().to_string());
    }
    if key.is_some() {
        command = format!("{} --key {}", command, key.unwrap().to_string());
    }
    let service_file_content = format!(r#"
[Unit]
Description=RustyProxySSL{}
After=network.target

[Service]
LimitNOFILE=infinity
LimitNPROC=infinity
LimitMEMLOCK=infinity
LimitSTACK=infinity
LimitCORE=0
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

    let service_file_path = format!("/etc/systemd/system/rustyproxyssl{}.service", port);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(service_file_path)?;

    file.write_all(service_file_content.as_bytes())?;

    let commands = [
        "systemctl daemon-reload".to_string(),
        format!("systemctl enable rustyproxyssl{}.service", port),
        format!("systemctl start rustyproxyssl{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}
pub fn del_ssl_proxy_port(port: usize) -> Result<(), io::Error> {
    let commands = [
        format!("systemctl disable rustyproxyssl{}.service", port),
        format!("systemctl stop rustyproxyssl{}.service", port),
    ];
    for command in commands {
        run_command(command);
    }
    fs::remove_file(format!("/etc/systemd/system/rustyproxyssl{}.service", port))?;
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
LimitCORE=0
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

pub fn enable_openvpn(port: usize, mode: String) -> std::result::Result<(), io::Error> {
    let commands = [
        format!("sed -i 's/^port [^ ]\\+/port {}/g' /etc/openvpn/server.conf", port),
        format!("sed -i 's/^proto [^ ]\\+/proto {}/g' /etc/openvpn/server.conf", mode),
        "systemctl start openvpn".to_string(),
    ];
    for command in commands {
        run_command(command);
    }
    let ca_cert = fs::read_to_string("/etc/openvpn/ca.crt")?;
    let client_cert = fs::read_to_string("/etc/openvpn/easy-rsa/pki/issued/client.crt")?;
    let client_key = fs::read_to_string("/etc/openvpn/easy-rsa/pki/private/client.key")?;
    let ta_key = fs::read_to_string("/etc/openvpn/ta.key")?;

    let client_config = format!(r#"
client
dev tun
proto {}
sndbuf 0
rcvbuf 0
remote 127.0.0.1 {}
resolv-retry 5
nobind
persist-key
persist-tun
remote-cert-tls server
cipher AES-256-CBC
comp-lzo yes
setenv opt block-outside-dns
key-direction 1
verb 3
auth-user-pass
keepalive 10 120
float
<ca>
{}
</ca>
<cert>
{}
</cert>
<key>
{}
</key>
<tls-auth>
{}
</tls-auth>
"#, mode, port, ca_cert, client_cert, client_key, ta_key);

    fs::write("/root/client.ovpn", client_config)?;

    Ok(())
}
pub fn disable_openvpn() -> std::result::Result<(), io::Error> {
    let commands = [
        "sed -i 's/^port [^ ]\\+/port none/g' /etc/openvpn/server.conf".to_string(),
        "sed -i 's/^proto [^ ]\\+/proto none/g' /etc/openvpn/server.conf".to_string(),
        "systemctl stop openvpn".to_string(),
        "rm -f /root/client.ovpn".to_string()
    ];
    for command in commands {
        run_command(command);
    }
    Ok(())
}

pub fn add_proxy_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(0).ok(),
            },
            sslproxy: RustyProxySSL {
                ports: row.get::<_, String>(1).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(2).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(3).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(4).ok(),
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
                "INSERT INTO connections (proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (?, NULL, NULL, NULL, NULL)",
                params![port.to_string()]
            )?;
            Ok(())
        }
    }
}

pub fn add_sslproxy_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(0).ok(),
            },
            sslproxy: RustyProxySSL {
                ports: row.get::<_, String>(1).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(2).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(3).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(4).ok(),
            },
        })
    })?.collect::<Result<_, _>>()?;

    match connections.first() {
        Some(conn) => {
            let mut ports = conn.sslproxy.ports.clone().unwrap_or_default();
            if !ports.is_empty() {
                ports.push('|');
            }
            ports.push_str(&port.to_string());
            sqlite_conn.execute("UPDATE connections SET sslproxy_ports = ? WHERE id = 1", params![ports])?;
            Ok(())
        },
        None => {
            sqlite_conn.execute(
                "INSERT INTO connections (proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, ?, NULL, NULL, NULL)",
                params![port.to_string()]
            )?;
            Ok(())
        }
    }
}


pub fn add_badvpn_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(0).ok(),
            },
            sslproxy: RustyProxySSL {
                ports: row.get::<_, String>(1).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(2).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(3).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(4).ok(),
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
                "INSERT INTO connections (proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, NULL, ?, NULL, NULL)",
                params![port.to_string()]
            )?;
            Ok(())
        }
    }
}

pub fn add_checkuser_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(0).ok(),
            },
            sslproxy: RustyProxySSL {
                ports: row.get::<_, String>(1).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(2).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(3).ok(),
            },
            openvpn: OpenVpn {
                port: row.get::<_, String>(4).ok(),
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
                "INSERT INTO connections (proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, NULL, NULL, ?, NULL)",
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
            "INSERT INTO connections (proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port) VALUES (NULL, NULL, NULL, NULL, ?)",
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

pub fn del_sslproxy_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    let mut stmt = sqlite_conn.prepare("SELECT sslproxy_ports FROM connections LIMIT 1")?;
    let connections: Vec<String> = stmt.query_map(params![], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<_, _>>()?;

    if let Some(existing_ports) = connections.first() {
        let mut ports_vec: Vec<String> = existing_ports.trim().split('|').map(String::from).collect();
        ports_vec.retain(|p| p != &port.to_string());
        let new_ports = ports_vec.join("|");
        sqlite_conn.execute("UPDATE connections SET sslproxy_ports = ? WHERE id = 1", params![new_ports])?;
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
