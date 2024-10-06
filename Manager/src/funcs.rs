use std::net::{TcpListener};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::process::{Command};
use mongodb::sync::{Collection, Database};
use mongodb::bson::{doc};

use rand::Rng;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    login_type: String,
    pub(crate) user: String,
    pub(crate) pass: String,
    pub(crate) limit: String,
    pub(crate)  expiry: String,
}

pub fn create_user(user: &str, pass: &str, days: usize, limit: usize, of_menu: bool, database: Database) -> String {

    if !of_menu {
        if user_already_exists(user) {
            return "user already exists".to_string()
        }
    }

    let collection: Collection<User> = database.collection("users");

    let commands = [
        format!("/usr/sbin/useradd -M -N -s /bin/false {} -e $(date -d \"+{} days\" +%Y-%m-%d\n)", user, days),
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user)
    ];

    for command in commands {
        run_command(command);
    }

    let insert = collection.insert_one(
        User{
            login_type: "user".to_string(),
            user: user.to_string(),
            pass: pass.to_string(),
            limit: limit.to_string(),
            expiry: days_to_expire_date(days).to_string(),
        }
    );


    match insert.run() {
        Ok(_) => {
            "created".to_string()
        }
        Err(err) => {
            println!("{}", err);
            "error on insert user in db".to_string()
        }
    }
}


pub fn remove_user(user: &str, of_menu: bool, database: Database) -> String {
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

    let collection: Collection<User> = database.collection("users");

    let filter =
        doc! { "$and": [
           doc! { "user": user },
       ]
    };
    let result = collection.delete_one(filter).run();

    match result {
        Ok(_) => {
            "removed".to_string()
        }
        Err(err) => {
            println!("{}", err);
            "error on remove user at db".to_string()
        }
    }
}



pub fn generate_test(time: usize, database: Database) -> String {

    let mut rng = rand::thread_rng();
    let n = rng.gen_range(1000..=9999);

    let user = format!("test{}", n);
    let pass = format!("test{}", n);

    let collection: Collection<User> = database.collection("users");

    let commands = [
        format!("/usr/sbin/useradd -M -N -s /bin/false {} -e $(date -d \"+{} minutes\" +%Y-%m-%dT%H:%M:%S)", user, time),
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user),
        format!("echo \"/root/SshScript --remove-user {}\" | at \"now + {} minute\" ", user, time),
    ];

    for command in commands {
        run_command(command);
    }

    let insert = collection.insert_one(
        User{
            login_type: "test".to_string(),
            user: user.to_string(),
            pass: pass.to_string(),
            limit: 1.to_string(),
            expiry: minutes_to_expire_date(time).to_string(),
        }
    );


    match insert.run() {
        Ok(_) => {
            format!("user: {} | pass: {} | limit: {} | minutes remaining: {}", user, pass, 1, time)
        }
        Err(err) => {
            println!("{}", err);
            "error on insert user in db".to_string()
        }
    }
}

pub fn change_validity(user: &str, days: usize, of_menu: bool, database: Database) -> String {
    if !of_menu {
        if !user_already_exists(user) {
            return "user does not exist".to_string()
        }
    }

    let collection: Collection<User> = database.collection("users");

    let commands = [
        format!("sudo chage -E $(date -d \"+{} days\" +%Y-%m-%d) {}", days, user),
    ];


    for command in commands {
        run_command(command);
    }
    let new_expiry_date = days_to_expire_date(days);
    let filter = doc! { "user": user };
    let update = doc! { "$set": doc! {"expiry": new_expiry_date.clone()} };
    match  collection.update_one(filter, update).run() {
        Ok(_) => {
            format!("changed | new expire date: {}", new_expiry_date)
        }
        Err(err) => {
            println!("{}", err);
            "error on update user in db".to_string()
        }
    }
}

pub fn change_limit(user: &str, limit: usize, of_menu: bool,  database: Database) -> String {
    if !of_menu {
        if !user_already_exists(user) {
            return "user does not exist".to_string()
        }
    }

    let collection: Collection<User> = database.collection("users");
    let filter = doc! { "user": user };
    let update = doc! { "$set": doc! {"limit": limit.to_string()} };
    match  collection.update_one(filter, update).run() {
        Ok(_) => {
            format!("changed | new limit: {}", limit)
        }
        Err(err) => {
            println!("{}", err);
            "error on update user in db".to_string()
        }
    }
}

pub fn change_pass(user: &str, pass: &str, of_menu: bool,  database: Database) -> String {
    if !of_menu {
        if !user_already_exists(user) {
            return "user does not exist".to_string()
        }
    }
    let collection: Collection<User> = database.collection("users");

    let commands = [
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user),
        format!("pkill -u {}", user)
    ];


    for command in commands {
        run_command(command);
    }

    let filter = doc! { "user": user };
    let update = doc! { "$set": doc! {"pass": pass} };
    match  collection.update_one(filter, update).run() {
        Ok(_) => {
            format!("changed | new pass: {}", pass)
        }
        Err(err) => {
            println!("{}", err);
            "error on update user in db".to_string()
        }
    }
}

pub fn users_report_json(database: Database) -> String {
    serde_json::to_string_pretty(&users_report_vec(database)).expect("Serialization failed")
}

pub fn users_report_vec(database: Database) -> Vec<User> {
    let collection: Collection<User> = database.collection("users");
    let users = collection.find(doc!{}).run().unwrap();
    let vec_result_users = users.collect::<Vec<_>>();
    vec_result_users.iter().map(|x| x.clone().unwrap()).collect::<Vec<User>>()
}



pub fn expired_report_json(database: Database) -> String {
    let collection: Collection<User> = database.collection("users");
    let users = collection.find(doc!{}).run().unwrap();
    let vec_result_users = users.collect::<Vec<_>>();
    let vec_users = vec_result_users.iter().map(|x| x.clone().unwrap()).collect::<Vec<User>>();
    serde_json::to_string_pretty(&expired_users(vec_users)).expect("Serialization failed")
}

pub fn expired_report_vec(database: Database) -> Vec<User> {
    let collection: Collection<User> = database.collection("users");
    let users = collection.find(doc!{}).run().unwrap();
    let vec_result_users = users.collect::<Vec<_>>();
    let vec_users = vec_result_users.iter().map(|x| x.clone().unwrap()).collect::<Vec<User>>();
    expired_users(vec_users)
}

fn expired_users(users:  Vec<User>) -> Vec<User> {
    let mut vec_expired_users: Vec<User> = Vec::new();
    for user in &users {
        if user.login_type == "user" {
            let now = Local::now();
            let expiry =  DateTime::parse_from_str(&user.expiry, "%Y-%m-%d %H:%M:%S%.3f %z").unwrap();
            if now > expiry {
                vec_expired_users.push(user.clone());
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpProxy {
    pub(crate) enabled: bool,
    pub(crate) port: u16,
}

pub fn get_proxy_state(database: Database) -> HttpProxy {
    let collection: Collection<Connections> = database.collection("connections");

    let filter = doc! {};
    if let Some(conn) = collection.find_one(filter).run().unwrap() {
        conn.proxy
    } else {
        HttpProxy {
            enabled: false,
            port: 0,
        }
    }
}

pub fn enable_or_disable_proxy(port: usize, database: Database) -> Result<String, Box<dyn std::error::Error>> {
    let collection: Collection<Connections> = database.collection("connections");

    let filter = doc! {};
    let connections = collection.find_one(filter.clone()).run().unwrap();

    match connections {
        Some(mut conn) => {
            if conn.proxy.enabled {
                conn.proxy.enabled = false;
                conn.proxy.port = 0;
                let commands = [
                    "systemctl disable proxy.service".to_string(),
                    "systemctl stop proxy.service".to_string(),
                ];
                for command in commands {
                    run_command(command);
                }
            } else {
                conn.proxy.enabled = true;
                conn.proxy.port = port as u16;
                let commands = [
                    "systemctl enable proxy.service".to_string(),
                    "systemctl start proxy.service".to_string(),
                ];
                for command in commands {
                    run_command(command);
                }
            }

            collection.replace_one(filter, conn.clone()).run().unwrap();
            Ok(format!("Proxy status updated: {:?}", conn.proxy))
        },
        None => {
            let new_connection = Connections {
                proxy: HttpProxy {
                    enabled: true,
                    port: port as u16,
                },
            };
            collection.insert_one(new_connection).run().unwrap();
            let commands = [
                "systemctl enable proxy.service".to_string(),
                "systemctl start proxy.service".to_string(),
            ];
            for command in commands {
                run_command(command);
            }
            Ok(format!("Proxy enabled with port: {}", port))
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

