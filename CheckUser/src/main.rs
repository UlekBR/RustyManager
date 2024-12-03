use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use std::process::Command;
use clap::Parser;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Parser)]
#[command(name = "CheckUser")]
#[command(about = "a simple checkuser")]
struct Args {
    #[arg(long, default_value = "3232")]
    port: u16
}

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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let addr = ([0, 0, 0, 0, 0, 0, 0, 0], args.port).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on http://[::]:{}", args.port);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let sqlite_conn = Connection::open("/opt/rustymanager/db").unwrap();
    let uri = req.uri().to_string();
    println!("{}", uri);

    match *req.method() {
        Method::GET => {
            return if uri.contains("?deviceId") {
                // app = dtunnel
                let user = uri.split("check/").last().unwrap().split("?deviceId").next().unwrap();
                let user_data = get_user(user, &sqlite_conn);
                let response = DtunnelResponse {
                    username: user_data.user,
                    count_connections: user_data.connections.parse().unwrap_or(0),
                    limit_connections: user_data.limit.parse().unwrap_or(0),
                    expiration_date: user_data.expiry_date,
                    expiration_days: user_data.expiry_days.parse().unwrap_or(0),
                    id: 0,
                };
                Ok(Response::new(Body::from(
                    serde_json::to_string_pretty(&response).expect("Serialization failed"),
                )))
            } else if uri.contains("check/") {
                // app = gltunnel
                let user = uri.split("check/").last().unwrap().split("?").next().unwrap();
                let user_data = get_user(user, &sqlite_conn);
                let response = GltunnelResponse {
                    username: user_data.user,
                    count_connection: user_data.connections,
                    limit_connection: user_data.limit,
                    expiration_date: user_data.expiry_date,
                    expiration_days: user_data.expiry_days,
                };
                Ok(Response::new(Body::from(
                    serde_json::to_string_pretty(&response).expect("Serialization failed"),
                )))
            } else {
                Ok(Response::new(Body::from("RustyManager Checkuser :)\nby @UlekBR")))
            }
        }
        Method::POST => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let body_str = String::from_utf8(whole_body.to_vec()).unwrap();

            if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&body_str) {
                if let Some(username) = json_data.get("user").and_then(|v| v.as_str()) {
                    // app = conecta4g
                    let user_data = get_user(username, &sqlite_conn);
                    let response = Conecta4gResponse {
                        username: username.to_string(),
                        count_connection: user_data.connections,
                        limiter_user: user_data.limit,
                        expiration_date: user_data.expiry_date,
                        expiration_days: user_data.expiry_days,
                    };
                    let json_response = serde_json::to_string(&response).unwrap();
                    return Ok(Response::new(Body::from(json_response)));
                }else {
                    Ok(Response::new(Body::from("RustyManager Checkuser :)\nby @UlekBR")))
                }
            } else {
                // app = anyvpn
                let username = body_str.split("username=").last().unwrap().split('&').next().unwrap();
                let device_id = body_str.split("deviceid=").last().unwrap();

                let user_data = get_user(username, &sqlite_conn);
                let response = AnyVpnResponse {
                    username: username.to_string(),
                    is_active: "true".to_string(),
                    expiry: format!("{} dias.", user_data.expiry_days),
                    expiration_date: user_data.expiry_date_anymod,
                    device_id: device_id.to_string(),
                    uuid: "null".to_string(),
                };
                let json_response = serde_json::to_string(&response).unwrap();
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Body::from(json_response))
                    .unwrap());
            }
        }
        _ => {
            return Ok(Response::new(Body::from("RustyManager Checkuser :)\nby @UlekBR")));
        }
    }
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