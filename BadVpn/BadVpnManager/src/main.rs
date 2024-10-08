use std::env;

use mongodb::sync::Client;
use crate::funcs::{add_port, add_port_in_db, del_port, del_port_in_db, is_port_avaliable};

mod funcs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let uri = "mongodb://127.0.0.1:27017/";
    let client = Client::with_uri_str(uri).expect("error on mongodb connect");
    let database = client.database("ssh");

    if args.len() >= 1 {
        match (&args[1]).as_str() {
            "--enable-port" => {
                let port =  (&args[2]).as_str();
                match port.parse::<usize>() {
                    Ok(port) => {
                        if is_port_avaliable(port).expect("error on check port use") {
                            add_port(port).expect("error on enable port");
                            add_port_in_db(database, port).expect("error on insert port in db");
                        }
                    }
                    Err(..) => {
                        println!("invalid port");
                    }
                }
            }
            "--disable-port" => {
                let port =  (&args[2]).as_str();
                match port.parse::<usize>() {
                    Ok(port) => {
                        if !is_port_avaliable(port).expect("error on check port use")  {
                            del_port(port).expect("error on disable port");
                            del_port_in_db(database, port).expect("error on remove port in db");
                        }
                    }
                    Err(..) => {
                        println!("invalid port");
                    }
                }
            }
            _ => {}
        }
    }
}
