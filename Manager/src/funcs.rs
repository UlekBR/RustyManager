use std::fs;
use std::net::{TcpListener};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::process::{Command};
use rand::Rng;
use rusqlite::{Connection, OptionalExtension, params};



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
    pub(crate) proxy: HttpProxy,
    pub(crate) stunnel: Stunnel,
    pub(crate) badvpn: BadVpn,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadVpn {
    pub(crate) ports: Option<Vec<u16>>,
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

pub fn get_connections(conn: &Connection) -> Result<Connections, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT http_proxy_enabled, http_proxy_port, stunnel_enabled, stunnel_port, badvpn_ports FROM connections LIMIT 1")?;

    let connection: Option<(Option<bool>, Option<u16>, Option<bool>, Option<u16>, Option<String>)> = stmt.query_row([], |row| {
        Ok((
            row.get(0).ok(),
            row.get(1).ok(),
            row.get(2).ok(),
            row.get(3).ok(),
            row.get(4).ok()
        ))
    }).optional().unwrap();

    match connection {
        Some((http_proxy_enabled, http_proxy_port, stunnel_enabled, stunnel_port, badvpn_ports)) => {
            Ok(Connections {
                proxy: HttpProxy {
                    enabled: Option::from(http_proxy_enabled.unwrap_or(false)),
                    port: Option::from(http_proxy_port.unwrap_or(0)),
                },
                stunnel: Stunnel {
                    enabled: Option::from(stunnel_enabled.unwrap_or(false)),
                    port: Option::from(stunnel_port.unwrap_or(0)),
                },
                badvpn: BadVpn {
                    ports: Option::from(badvpn_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                }
            })
        },
        None => Ok(Connections {
            proxy: HttpProxy {
                enabled: Some(false),
                port: Some(0),
            },
            stunnel: Stunnel {
                enabled: Some(false),
                port: Some(0),
            },
            badvpn: BadVpn {
                ports: Some(Vec::new()),
            },
        })
    }
}


pub fn get_proxy_state(conn: &Connection) -> Result<HttpProxy, Box<dyn std::error::Error>> {
    let connections = get_connections(conn).unwrap();
    Ok(connections.proxy)
}

pub fn get_stunnel_state(conn: &Connection) -> Result<Stunnel, Box<dyn std::error::Error>> {
    let connections = get_connections(conn).unwrap();
    Ok(connections.stunnel)
}

pub fn enable_or_disable_proxy(port: usize, conn: &Connection) -> Result<String, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT http_proxy_enabled, http_proxy_port FROM connections LIMIT 1")?;

    let connection: Option<(Option<bool>, Option<u16>)> = stmt.query_row([], |row| {
        Ok((
            row.get(0).ok(),
            row.get(1).ok()
        ))
    }).optional().unwrap();


    match connection {
        Some((enabled, _port)) => {
            if enabled.unwrap_or(false) {
                // Desativar o proxy
                let commands = [
                    "systemctl disable proxy.service".to_string(),
                    "systemctl stop proxy.service".to_string(),
                ];
                for command in commands {
                    run_command(command);
                }

                conn.execute(
                    "UPDATE connections SET http_proxy_enabled = ?1, http_proxy_port = ?2",
                    params![false, 0],
                ).unwrap();

                Ok("Proxy status updated: disabled".to_string())
            } else {
                // Ativar o proxy
                let commands = [
                    "systemctl enable proxy.service".to_string(),
                    "systemctl start proxy.service".to_string(),
                ];
                for command in commands {
                    run_command(command);
                }

                conn.execute(
                    "UPDATE connections SET http_proxy_enabled = ?1, http_proxy_port = ?2",
                    params![true, port as u16],
                ).unwrap();

                Ok("Proxy status updated: enabled".to_string())
            }
        },
        None => {
            // Ativar stunnel
            let commands = [
                "systemctl enable proxy.service".to_string(),
                "systemctl start proxy.service".to_string(),
            ];
            for command in commands {
                run_command(command);
            }

            conn.execute(
                "INSERT INTO connections (http_proxy_enabled, http_proxy_port) VALUES (?1, ?2)",
                params![true, port as u16],
            ).unwrap();

            Ok("Proxy status updated: enabled (new entry created)".to_string())
        }
    }
}


pub fn enable_or_disable_stunnel(port: usize, conn: &Connection) -> Result<String, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT stunnel_enabled, stunnel_port FROM connections LIMIT 1")?;

    let connection: Option<(Option<bool>, Option<u16>)> = stmt.query_row([], |row| {
        Ok((
            row.get(0).ok(),
            row.get(1).ok()
        ))
    }).optional().unwrap();


    match connection {
        Some((enabled, _port)) => {
            if enabled.unwrap_or(false) {
                // Desativar stunnel
                let commands = [
                    "systemctl disable stunnel4.service".to_string(),
                    "systemctl stop stunnel4.service".to_string(),
                ];
                for command in commands {
                    run_command(command);
                }

                conn.execute(
                    "UPDATE connections SET stunnel_enabled = ?1, stunnel_port = ?2",
                    params![false, 0],
                ).unwrap();

                Ok("Stunnel status updated: disabled".to_string())
            } else {
                // Ativar stunnel
                let stunnel_config = format!(r#"
                    cert = /etc/stunnel/cert.pem
                    key = /etc/stunnel/key.pem
                    client = no
                    [stunnel]
                    connect = 127.0.0.1:22
                    accept = {}
                "#, port);
                fs::write("/etc/stunnel/stunnel.conf", stunnel_config)?;

                let commands = [
                    "systemctl enable stunnel4.service".to_string(),
                    "systemctl start stunnel4.service".to_string(),
                ];
                for command in commands {
                    run_command(command);
                }

                conn.execute(
                    "UPDATE connections SET stunnel_enabled = ?1, stunnel_port = ?2",
                    params![true, port as u16],
                ).unwrap();


                Ok("Stunnel status updated: enabled".to_string())
            }
        },
        None => {
            // Ativar stunnel
            let stunnel_config = format!(r#"
                    cert = /etc/stunnel/cert.pem
                    key = /etc/stunnel/key.pem
                    client = no
                    [stunnel]
                    connect = 127.0.0.1:22
                    accept = {}
                "#, port);
            fs::write("/etc/stunnel/stunnel.conf", stunnel_config)?;

            let commands = [
                "systemctl enable stunnel4.service".to_string(),
                "systemctl start stunnel4.service".to_string(),
            ];
            for command in commands {
                run_command(command);
            }

            conn.execute(
                "INSERT INTO connections (stunnel_enabled, stunnel_port) VALUES (?1, ?2)",
                params![true, port as u16],
            ).unwrap();

            Ok("Stunnel status updated: enabled (new entry created)".to_string())
        }
    }
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

pub fn enable_badvpn_port(port: String) {
    run_command(format!("/opt/rustymanager/badmanager --enable-port {}", port));
}

pub fn disable_badvpn_port(port: String) {
    run_command(format!("/opt/rustymanager/badmanager --disable-port {}", port));
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
        let error_message = std::str::from_utf8(&exec.stderr).unwrap_or("Error converting error message");
        return format!("Error: {}", error_message);
    }

    let output = std::str::from_utf8(&exec.stdout).unwrap_or("Error converting output");
    output.trim().to_string()
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

