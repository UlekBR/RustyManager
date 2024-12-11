use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::net::{TcpListener};
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::process::{Command};
use rand::Rng;
use rusqlite::{Connection, OptionalExtension};


pub fn create_user(user: &str, pass: &str, days: usize, limit: usize, of_menu: bool, conn: &Connection) -> String {
    if !of_menu {
        if user_already_exists(user) {
            return "user already exists".to_string()
        }
    }

    let commands = [
        format!("/usr/sbin/useradd -M -N -s /bin/false {} -e $(date -d \"+{} days\" +%Y-%m-%d\n)", user, days),
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user)
    ];

    for command in commands {
        run_command(command);
    }

    conn.execute(
        "INSERT INTO users (login_type, login_user, login_pass, login_limit, login_expiry) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("user", user, pass, limit, days_to_expire_date(days).to_string()),
    ).expect("error on insert user in db");

    "created".to_string()
}


pub fn remove_user(user: &str, of_menu: bool, conn: &Connection) -> String {
    if !of_menu {
        if !user_already_exists(user) {
            return "user does not exist".to_string()
        }
    }

    let commands = [
        format!("userdel --force {}", user),
        format!("pkill -u {}", user),
    ];

    for command in commands {
        run_command(command);
    }

    conn.execute(
        "DELETE FROM users WHERE login_user = ?1",
        [user],
    ).expect("error on remove of db");

    "removed".to_string()
}



pub fn generate_test(time: usize, conn: &Connection) -> String {
    let mut rng = rand::thread_rng();
    let n = rng.gen_range(1000..=9999);

    let user = format!("test{}", n);
    let pass = format!("test{}", n);

    let commands = [
        format!("/usr/sbin/useradd -M -N -s /bin/false {} -e $(date -d \"+{} minutes\" +%Y-%m-%dT%H:%M:%S)", user, time),
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user),
        format!("echo \"/opt/rustymanager/manager --remove-user {}\" | at \"now + {} minute\"", user, time),
    ];

    for command in commands {
        run_command(command);
    }

    conn.execute(
        "INSERT INTO users (login_type, login_user, login_pass, login_limit, login_expiry) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("test", &user, &pass, 1, minutes_to_expire_date(time).to_string()),
    ).expect("error on insert user in db");

    format!("user: {} | pass: {} | limit: {} | minutes remaining: {}", user, pass, 1, time)
}


pub fn change_validity(user: &str, days: usize, of_menu: bool, conn: &Connection) -> String {
    if !of_menu {
        if !user_already_exists(user) {
            return "user does not exist".to_string();
        }
    }
    run_command(format!("sudo chage -E $(date -d \"+{} days\" +%Y-%m-%d) {}", days, user));
    let new_expiry_date = days_to_expire_date(days);
    conn.execute(
        "UPDATE users SET login_expiry = ?1 WHERE login_user = ?2",
        (&new_expiry_date, user),
    ).expect("error on update user");

    format!("changed | new expire date: {}", new_expiry_date)
}

pub fn change_limit(user: &str, limit: usize, of_menu: bool,  conn: &Connection) -> String {
    if !of_menu {
        if !user_already_exists(user) {
            return "user does not exist".to_string()
        }
    }
    conn.execute(
        "UPDATE users SET login_limit = ?1 WHERE login_user = ?2",
        (limit, user),
    ).expect("error on update user");

    format!("changed | new limit: {}", limit)
}

pub fn change_pass(user: &str, pass: &str, of_menu: bool, conn: &Connection) -> String {
    if !of_menu {
        if !user_already_exists(user) {
            return "user does not exist".to_string()
        }
    }

    let commands = [
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user),
        format!("pkill -u {}", user)
    ];

    for command in commands {
        run_command(command);
    }
    conn.execute(
        "UPDATE users SET login_pass = ?1 WHERE login_user = ?2",
        (pass, user),
    ).expect("error on update user");
    format!("changed | new pass: {}", pass)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    login_type: String,
    pub(crate) user: String,
    pub(crate) pass: String,
    pub(crate) limit: String,
    pub(crate)  expiry: String,
}

pub fn userdata(user: &str, conn: &Connection) -> String {
    let mut stmt = conn.prepare("SELECT login_type, login_user, login_pass, login_limit, login_expiry FROM users WHERE login_user = ?1").unwrap();
    let user = stmt.query_row([user], |row| {
        Ok(User {
            login_type: row.get(0)?,
            user: row.get(1)?,
            pass: row.get(2)?,
            limit: row.get(3)?,
            expiry: row.get(4)?,
        })
    }).unwrap_or_else(|_| User {
        login_type: String::from("not found"),
        user: String::from("not found"),
        pass: String::from("not found"),
        limit: String::from("not found"),
        expiry: String::from("not found"),
    });
    serde_json::to_string_pretty(&user).expect("Serialization failed")
}

pub fn users_report_json(conn: &Connection) -> String {
    serde_json::to_string_pretty(&users_report_vec(conn)).expect("Serialization failed")
}

pub fn users_report_vec(conn: &Connection) -> Vec<User> {
    let mut stmt = conn.prepare("SELECT login_type, login_user, login_pass, login_limit, login_expiry FROM users").unwrap();
    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            login_type: row.get(0)?,
            user: row.get(1)?,
            pass: row.get(2)?,
            limit: row.get(3)?,
            expiry: row.get(4)?,
        })
    }).unwrap();

    user_iter.filter_map(Result::ok).collect()
}

pub fn expired_report_json(conn: &Connection) -> String {
    let expired_users = expired_report_vec(conn);
    serde_json::to_string_pretty(&expired_users).expect("Serialization failed")
}

pub fn expired_report_vec(conn: &Connection) -> Vec<User> {
    let all_users = users_report_vec(conn);
    expired_users(all_users)
}

fn expired_users(users: Vec<User>) -> Vec<User> {
    let mut vec_expired_users: Vec<User> = Vec::new();
    for user in &users {
        if user.login_type == "user" {
            let now = Local::now();
            if let Ok(expiry) = DateTime::parse_from_str(&user.expiry, "%Y-%m-%d %H:%M:%S%.3f %z") {
                if now > expiry {
                    vec_expired_users.push(user.clone());
                }
            }
        }
    }
    vec_expired_users
}

pub fn user_already_exists(user: &str) -> bool {
    let exec = Command::new("bash")
        .arg("-c")
        .arg(format!("getent passwd {}", user))
        .output()
        .expect("error on run command");

    if exec.status.success() {
        if !exec.stdout.is_empty() {
            return true
        }
    }
    false
}

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
    pub(crate) ports: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustyProxySSL {
    pub(crate) ports: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadVpn {
    pub(crate) ports: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckUser {
    pub(crate) ports: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenVpn {
    pub(crate) port: Option<String>,
}



pub fn get_connections(conn: &Connection) -> Result<Connections, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT proxy_ports, sslproxy_ports, badvpn_ports, checkuser_ports, openvpn_port FROM connections LIMIT 1")?;

    let connection: Option<(Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = stmt.query_row([], |row| {
        Ok((
            row.get(0).ok(),
            row.get(1).ok(),
            row.get(2).ok(),
            row.get(3).ok(),
            row.get(4).ok(),
        ))
    }).optional().unwrap();

    match connection {
        Some((proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports, openvpn_port)) => {
            Ok(Connections {
                proxy: RustyProxy {
                    ports: Option::from(proxy_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                },
                sslproxy: RustyProxySSL {
                    ports: Option::from(stunnel_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                },
                badvpn: BadVpn {
                    ports: Option::from(badvpn_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                },
                checkuser: CheckUser {
                    ports: Option::from(checkuser_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                },
                openvpn: OpenVpn {
                    port: Option::from(openvpn_port),
                }
            })
        },
        None => Ok(Connections {
            proxy: RustyProxy {
                ports: Some(Vec::new()),
            },
            sslproxy: RustyProxySSL {
                ports: Some(Vec::new()),
            },
            badvpn: BadVpn {
                ports: Some(Vec::new()),
            },
            checkuser: CheckUser {
                ports: Some(Vec::new()),
            },
            openvpn: OpenVpn {
                port: Some(String::new()),
            },
        })
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnlineUser {
    pub(crate) user: String,
    pub(crate) connected: String,
    pub(crate) limit: String,
}
pub fn online_report_json(conn: &Connection) -> String {
    serde_json::to_string_pretty(&online_report(conn)).expect("Serialization failed")
}
pub fn online_report(conn: &Connection) -> Vec<OnlineUser> {
    let ssh_users = run_command_and_get_output("ps -e -o user= -o cmd= | grep '[s]shd: ' | grep -v 'sshd: root@'");
    let ovpn_users = run_command_and_get_output("sed -n '/Common Name/,/ROUTING TABLE/{/Common Name/d;/ROUTING TABLE/q;s/,.*//p}' /etc/openvpn/openvpn-status.log 2>/dev/null || true");
    let users = format!("{}\n{}", ssh_users, ovpn_users);

    let mut online_users: Vec<OnlineUser> = Vec::new();
    let connections = String::from_utf8_lossy(users.as_ref());
    let mut user_connections: HashMap<&str, usize> = HashMap::new();
    for line in connections.lines() {
        let user = line.split_whitespace().next().unwrap_or("");
        if user != "root" && user != "sshd" && !user.is_empty() {
            *user_connections.entry(user).or_insert(0) += 1;
        }
    }

    for (user, count) in user_connections.iter() {
        let mut stmt = conn.prepare("SELECT login_type, login_user, login_pass, login_limit, login_expiry FROM users WHERE login_user = ?1").unwrap();
        let db_user: User = stmt.query_row([user], |row| {
            Ok(User {
                login_type: row.get(0)?,
                user: row.get(1)?,
                pass: row.get(2)?,
                limit: row.get(3)?,
                expiry: row.get(4)?,
            })
        }).unwrap_or_else(|_| User {
            login_type: String::new(),
            user: String::new(),
            pass: String::new(),
            limit: String::from("0"),
            expiry: String::new(),
        });

        online_users.push(OnlineUser {
            user: user.to_string(),
            connected: count.to_string(),
            limit: db_user.limit,
        });
    }
    online_users
}


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
pub fn enable_proxy_port(port: String, status: String) {
    if status.is_empty() {
        run_command(format!("/opt/rustymanager/connectionsmanager --conn proxy --enable-port {}", port));
    } else {
        run_command(format!("/opt/rustymanager/connectionsmanager --conn proxy --enable-port {} --status {}", port, status));
    }
}

pub fn disable_proxy_port(port: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn proxy --disable-port {}", port));
}



pub fn enable_sslproxy_port(port: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn sslproxy --enable-port {}", port));
}
pub fn disable_sslproxy_port(port: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn sslproxy --disable-port {}", port));
}

pub fn enable_badvpn_port(port: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn badvpn --enable-port {}", port));
}

pub fn disable_badvpn_port(port: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn badvpn --disable-port {}", port));
}

pub fn enable_checkuser_port(port: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn checkuser --enable-port {}", port));
}

pub fn disable_checkuser_port(port: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn checkuser --disable-port {}", port));
}

pub fn enable_openvpn(port: String, mode: String) {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn openvpn --enable {} --mode {}", port, mode));
}

pub fn disable_openvpn() {
    run_command("/opt/rustymanager/connectionsmanager --conn openvpn --disable".to_string());
}

pub fn journald_status() -> bool {
    let output = run_command_and_get_output("systemctl is-active systemd-journald.service");
    if output == "active" {
        true
    } else {
        false
    }
}

pub fn enable_journald() {
    let commands = [
        "systemctl start --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string(),
        "systemctl enable --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string()
    ];
    for command in commands {
        run_command(command);
    }
}

pub fn disable_journald() {
    let commands = [
        "systemctl stop --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string(),
        "systemctl disable --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string()
    ];
    for command in commands {
        run_command(command);
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SpeedTestData {
    pub(crate) r#type: String,
    pub(crate) timestamp: String,
    pub(crate) ping: Ping,
    pub(crate)  download: Transfer,
    pub(crate)  upload: Transfer,
    #[serde(rename = "packetLoss")]
    pub(crate)  packet_loss: f64,
    pub(crate)   isp: String,
    pub(crate)   interface: NetworkInterface,
    pub(crate)  server: Server,
    pub(crate)  result: ResultInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ping {
    pub(crate) jitter: f64,
    pub(crate)  latency: f64,
    pub(crate)  low: f64,
    pub(crate)  high: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transfer {
    pub(crate)   bandwidth: u64,
    pub(crate)   bytes: u64,
    pub(crate)   elapsed: u64,
    pub(crate)   latency: Latency,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Latency {
    pub(crate)   iqm: f64,
    pub(crate)   low: f64,
    pub(crate)   high: f64,
    pub(crate)   jitter: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInterface {
    #[serde(rename = "internalIp")]
    pub(crate)  internal_ip: String,
    pub(crate)  name: String,
    #[serde(rename = "macAddr")]
    pub(crate)  mac_addr: String,
    #[serde(rename = "isVpn")]
    pub(crate)    is_vpn: bool,
    #[serde(rename = "externalIp")]
    pub(crate)  external_ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub(crate)  id: u64,
    pub(crate)  host: String,
    pub(crate)  port: u16,
    pub(crate)  name: String,
    pub(crate)  location: String,
    pub(crate)   country: String,
    pub(crate)  ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultInfo {
    pub(crate)   id: String,
    pub(crate)    url: String,
    pub(crate)  persisted: bool,
}
pub fn speedtest_data() -> SpeedTestData {
    let json = run_command_and_get_output("speedtest --accept-license --accept-gdpr -f json");
    serde_json::from_str(&json).unwrap()
}



#[derive(Debug)]
pub struct Service {
    pub(crate) name: String,
    pub(crate) ports: Vec<u16>,
}

pub fn get_services() -> Vec<Service> {
    let command = "netstat -tulnp | awk '/LISTEN/ {split($4, a, \":\"); split($7, b, \"/\"); gsub(\":\", \"\", b[2]); if (!seen[b[2] a[length(a)]]++) ports[b[2]] = ports[b[2]] \" \" a[length(a)]} END {for (service in ports) print service, ports[service]}' | sort -u";

    let output_str = run_command_and_get_output(command);
    let mut services_map: HashMap<String, Vec<u16>> = HashMap::new();

    for line in output_str.lines() {
        let mut parts = line.split_whitespace();
        if let Some(service_name) = parts.next() {
            let service_name = service_name.to_string();
            let ports: Vec<u16> = parts
                .filter_map(|port_str| port_str.parse::<u16>().ok())
                .collect();

            services_map.insert(service_name, ports);
        }
    }

    services_map
        .into_iter()
        .map(|(name, ports)| Service { name, ports })
        .collect()
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

pub fn run_command_and_get_output(command: &str) -> String {
    let exec = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute command");

    if !exec.status.success() {
        return String::new();
    }

    let output = std::str::from_utf8(&exec.stdout).unwrap_or("Error converting output");
    output.trim().to_string()
}

pub fn make_backup(conn: &Connection) -> String {
    let json = users_report_json(conn);
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("/root/backup.json")
        .unwrap();

    io::Write::write_all(&mut file, json.as_ref()).expect("error on write backup in file");
    "backup done in /root/backup.json".to_string()
}

pub fn restore_backup(conn: &Connection, path: String) -> String {
    if path.ends_with(".vps") {
        restore_backup_sshplus(conn, path)
    } else if path.ends_with(".json") {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let users: Vec<User> = serde_json::from_str(contents.as_str()).unwrap();
        for user in users {
            let create = create_user(user.user.as_str(), user.pass.as_str(), expire_date_to_days(user.expiry), user.limit.parse().unwrap(), false, conn);
            match create.as_str() {
                "created" => println!("created user: {}", user.user),
                "user already exists" => println!("already exists: {}", user.user),
                _ => {}
            }
        }
        "backup restored".to_string()
    } else {
        "invalid file".to_string()
    }
}

pub fn restore_backup_sshplus(conn: &Connection, path: String) -> String {
    let commands = vec![
        "mkdir /root/backup".to_string(),
        format!("tar -xvf {} -C /root/backup", path),
    ];
    for command in commands {
        run_command(command);
    }

    let db_file = File::open("/root/backup/root/usuarios.db").unwrap();
    let db_reader = BufReader::new(db_file);
    let db_lines = db_reader.lines();

    let mut users: Vec<User> = Vec::new();
    for line in db_lines {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        let user = parts[0].trim();
        if users.iter().any(|u| u.user == user) {
            continue
        }
        let mut limit = parts[1].trim();
        match limit.parse::<i32>() {
            Ok(_) => {},
            Err(_) => {
                limit = "1";
            },
        }

        let mut pass_file = match File::open(format!("/root/backup/etc/SSHPlus/senha/{}", user)) {
            Ok(file) => file,
            Err(_) => {
                continue;
            },
        };
        let mut pass = String::new();
        pass_file.read_to_string(&mut pass).unwrap();
        pass = pass.trim().to_string();

        let mut expiration_date = String::new();
        let shadow_file = File::open("/root/backup/etc/shadow").unwrap();
        let shadow_reader = BufReader::new(shadow_file);
        let shadow_lines = shadow_reader.lines();
        for shadow_line in shadow_lines {
            let shadow_line = shadow_line.unwrap();
            if shadow_line.starts_with(user) {
                let parts: Vec<&str> = shadow_line.split(':').collect();
                if parts.len() > 7 {
                    let expiration_days = parts[7].parse::<i64>().unwrap_or(0);
                    if expiration_days > 0 {

                        let epoch_start = NaiveDateTime::parse_from_str("1970-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
                        let epoch_seconds = expiration_days * 86400;
                        let expiration_datetime = epoch_start + Duration::seconds(epoch_seconds);

                        expiration_date = Local.from_local_datetime(&expiration_datetime).unwrap().to_rfc3339();
                    } else {
                        continue;
                    }
                }
            }
        }

        users.push(User{
            login_type: "user".to_string(),
            user: user.to_string(),
            pass: pass.to_string(),
            limit: limit.to_string(),
            expiry: expiration_date.to_string()
        })
    }

    run_command("rm -rf /root/backup".to_string());

    for user in users {
        let create = create_user(user.user.as_str(), user.pass.as_str(), expire_date_to_days(user.expiry), user.limit.parse().unwrap(), false, conn);
        match create.as_str() {
            "created" => println!("created user: {}", user.user),
            "user already exists" => println!("already exists: {}", user.user),
            _ => {}
        }
    }
    "backup restored".to_string()
}


fn expire_date_to_days(expiry: String) -> usize {
    let dt = DateTime::parse_from_str(expiry.as_str(), "%+");

    if let Ok(dt) = dt {
        let now = Utc::now();
        let duration = dt.with_timezone(&Utc) - now;
        duration.num_days() as usize
    } else {
        0usize
    }
}

fn days_to_expire_date(days: usize) -> String {
    let now: DateTime<Local> = Local::now();
    let expiry_date = now + Duration::days(days as i64);
    expiry_date.to_string()
}

fn minutes_to_expire_date(minutes: usize) -> String {
    let now: DateTime<Local> = Local::now();
    let expiry_date = now + Duration::minutes(minutes as i64);
    expiry_date.to_string()
}



