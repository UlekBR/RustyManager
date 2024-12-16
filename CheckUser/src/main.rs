use std::{env, thread};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct DtunnelResponse {
    username: String,
    count_connections: i32,
    limit_connections: i32,
    expiration_date: String,
    expiration_days: i32,
    id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GltunnelResponse {
    username: String,
    count_connection: String,
    limit_connection: String,
    expiration_date: String,
    expiration_days: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Conecta4gRequest {
    user: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Conecta4gResponse {
    username: String,
    count_connection: String,
    limiter_user: String,
    expiration_date: String,
    expiration_days: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnyVpnResponse {
    #[serde(rename = "USER_ID")]
    username: String,
    is_active: String,
    expiry: String,
    expiration_date: String,
    #[serde(rename = "DEVICE")]
    device_id: String,
    uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UlkCheckuserResponse {
    username: String,
    user_connected: String,
    user_limit: String,
    remaining_days: String,
    formatted_expiration_date: String,
    formatted_expiration_date_for_anymod: String,
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            if let Ok(request) = std::str::from_utf8(&buffer) {
                let method = request.split(" ").collect::<Vec<&str>>()[0];
                let uri = request.split(" ").collect::<Vec<&str>>()[1].split("HTTP/").collect::<Vec<&str>>()[0];

                let mut app = "";
                let mut user = "";
                let mut device_id = "";


                let mut post_user = String::new();
                if method == "GET" {
                    if uri.contains("?deviceId") {
                        app = "dtunnel";
                        user = uri.split("check/").last().unwrap().split("?deviceId").next().unwrap();
                    } else if uri.contains("check/") {
                        app = "gltunnel";
                        user = uri.split("check/").last().unwrap().split("?").next().unwrap();
                    } else if uri.contains("user=") {
                        app = "ulkcheckuser";
                        user = uri.split("user=").last().unwrap().trim();
                    }
                } else if  method == "POST" {
                    let post_str = if let Some(pos) = request.rfind('\n') {
                        &request[pos + 1..].split('\0').collect::<Vec<&str>>()[0].trim()
                    } else {
                        "".trim()
                    };

                    if let Ok(data) = serde_json::from_str::<Conecta4gRequest>(&post_str) {
                        app = "conecta4g";
                        post_user = data.user;
                    } else {
                        app = "anyvpn";
                        post_user = post_str.split("username=").last().unwrap().split('&').next().unwrap().to_string();
                        device_id = post_str.split("deviceid=").last().unwrap();
                    }
                    user = &*post_user;
                }

                if !user.is_empty() && user != "root" && user_already_exists(user) {
                    let sqlite_conn = Connection::open("/opt/rustymanager/db").unwrap();
                    let user_data = get_user(user, &sqlite_conn);

                    let checker_response = match app {
                        "dtunnel" => {
                            let response = DtunnelResponse {
                                username: user_data.user,
                                count_connections: user_data.connections.parse().unwrap_or(0),
                                limit_connections: user_data.limit.parse().unwrap_or(0),
                                expiration_date: user_data.expiry_date,
                                expiration_days: user_data.expiry_days.parse().unwrap_or(0),
                                id: 0,
                            };
                            serde_json::to_string_pretty(&response).expect("Serialization failed")
                        },
                        "gltunnel" => {
                            let response = GltunnelResponse {
                                username: user_data.user,
                                count_connection: user_data.connections,
                                limit_connection: user_data.limit,
                                expiration_date: user_data.expiry_date,
                                expiration_days: user_data.expiry_days,
                            };
                            serde_json::to_string_pretty(&response).expect("Serialization failed")
                        },
                        "conecta4g" => {
                            let response = Conecta4gResponse {
                                username: user.to_string(),
                                count_connection: user_data.connections,
                                limiter_user: user_data.limit,
                                expiration_date: user_data.expiry_date,
                                expiration_days: user_data.expiry_days,
                            };
                            serde_json::to_string_pretty(&response).expect("Serialization failed")
                        },
                        "anyvpn" => {
                            let response = AnyVpnResponse {
                                username: user.to_string(),
                                is_active: "true".to_string(),
                                expiry: format!("{} dias.", user_data.expiry_days),
                                expiration_date: user_data.expiry_date_anymod,
                                device_id: device_id.to_string(),
                                uuid: "null".to_string(),
                            };
                            serde_json::to_string_pretty(&response).expect("Serialization failed")
                        }

                        "ulkcheckuser" => {
                            let response = UlkCheckuserResponse {
                                username: user.to_string(),
                                user_connected: user_data.connections,
                                user_limit: user_data.limit,
                                formatted_expiration_date: user_data.expiry_date,
                                remaining_days: user_data.expiry_days,
                                formatted_expiration_date_for_anymod: user_data.expiry_date_anymod,
                            };
                            serde_json::to_string_pretty(&response).expect("Serialization failed")
                        }
                        _ => { "".to_string() }
                    };


                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}", checker_response);
                    stream.write_all(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
                } else {
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nhello :)".to_string();
                    stream.write_all(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
                }
            }
        }
        Err(e) => eprintln!("erro on read client: {}", e),
    }
}

fn main() {
    let port = get_port();
    let listener = TcpListener::bind(format!("[::]:{}", port)).expect("error on init server");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("error in accept user {}", e);
            }
        }
    }
}


fn get_port() -> u16 {
    let args: Vec<String> = env::args().collect();
    let mut port = 5454;

    for i in 1..args.len() {
        if args[i] == "--port" {
            if i + 1 < args.len() {
                port = args[i + 1].parse().unwrap_or(5454);
            }
        }
    }

    port
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
pub struct User {
    login_type: String,
    pub(crate) user: String,
    pub(crate) pass: String,
    pub(crate) limit: String,
    pub(crate)  expiry: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserForCheckUser {
    pub(crate) user: String,
    pub(crate) connections: String,
    pub(crate) limit: String,
    pub(crate) expiry_days: String,
    pub(crate) expiry_date: String,
    pub(crate) expiry_date_anymod: String
}


fn get_user(user: &str, sqlite_conn: &Connection) -> UserForCheckUser{
    let mut stmt = sqlite_conn.prepare("SELECT login_type, login_user, login_pass, login_limit, login_expiry FROM users WHERE login_user = ?1").unwrap();
    let db_user = stmt.query_row([user], |row| {
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
    let current_date = Utc::now();
    let expiry_date = DateTime::parse_from_str(db_user.expiry.as_str(), "%Y-%m-%d %H:%M:%S%.f %:z")
        .expect("Failed to parse date");
    let duration = expiry_date.with_timezone(&Utc).signed_duration_since(current_date);

    let formatted_expiry_date = expiry_date.format("%d/%m/%Y").to_string();
    let formatted_expiry_date_anymod = expiry_date.format("%Y-%m-%d").to_string();
    let user_expiry_days = duration.num_days().to_string();
    let user_connections = run_command_and_get_output(format!("ps -u {} | grep sshd | wc -l", user));

    UserForCheckUser {
        user: db_user.user,
        connections: user_connections,
        limit: db_user.limit,
        expiry_days: user_expiry_days,
        expiry_date: formatted_expiry_date,
        expiry_date_anymod: formatted_expiry_date_anymod,
    }
}


pub fn run_command_and_get_output(command: String) -> String {
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