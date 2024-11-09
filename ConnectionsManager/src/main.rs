use std::env;
use rusqlite::{Connection, Result};
use crate::funcs::{add_badvpn_port, add_badvpn_port_in_db, add_checkuser_port, add_checkuser_port_in_db, add_openvpn_port_in_db, add_proxy_port, add_proxy_port_in_db, add_stunnel_port, add_stunnel_port_in_db, del_badvpn_port, del_badvpn_port_in_db, del_checkuser_port, del_checkuser_port_in_db, del_openvpn_port_in_db, del_proxy_port, del_proxy_port_in_db, del_stunnel_port, del_stunnel_port_in_db, disable_openvpn, enable_openvpn, is_port_available};

mod funcs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let sqlite_conn = Connection::open("/opt/rustymanager/db").unwrap();

    if args.len() >= 4 {
        let connection_arg = args.get(1).unwrap();

        if connection_arg == "--conn" {
            let conn = args.get(2).unwrap();
            match conn.as_str() {
                "proxy" => {
                    let action_arg = args.get(3).unwrap();
                    match action_arg.as_str() {
                        "--enable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if is_port_available(port).expect("error on check port use") {
                                            if let Some(_status_arg) = args.get(5) {
                                                if let Some(status) = args.get(6) {
                                                    add_proxy_port(port, Some(status.clone())).expect("error on enable port");
                                                } else {
                                                    add_proxy_port(port, None).expect("error on enable port");
                                                }
                                            } else {
                                                add_proxy_port(port, None).expect("error on enable port");
                                            }
                                            add_proxy_port_in_db(&sqlite_conn, port as u16).expect("error on insert port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        "--disable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if !is_port_available(port).expect("error on check port use") {
                                            del_proxy_port(port).expect("error on disable port");
                                            del_proxy_port_in_db(&sqlite_conn, port as u16).expect("error on delete port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("specify a valid action [--enable-port, --disable-port]");
                        }
                    }
                },
                "stunnel" => {
                    let action_arg = args.get(3).unwrap();
                    match action_arg.as_str() {
                        "--enable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if is_port_available(port).expect("error on check port use") {
                                            let ipv6 = args.get(6).map_or(false, |arg| arg == "true");
                                            add_stunnel_port(port, ipv6).expect("error on enable port");
                                            add_stunnel_port_in_db(&sqlite_conn, port as u16).expect("error on insert port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        "--disable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if !is_port_available(port).expect("error on check port use") {
                                            del_stunnel_port(port).expect("error on disable port");
                                            del_stunnel_port_in_db(&sqlite_conn, port as u16).expect("error on delete port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("specify a valid action [--enable-port, --disable-port]");
                        }
                    }
                },
                "badvpn" => {
                    let action_arg = args.get(3).unwrap();
                    match action_arg.as_str() {
                        "--enable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if is_port_available(port).expect("error on check port use") {
                                            add_badvpn_port(port).expect("error on enable port");
                                            add_badvpn_port_in_db(&sqlite_conn, port as u16).expect("error on insert port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        "--disable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if !is_port_available(port).expect("error on check port use") {
                                            del_badvpn_port(port).expect("error on disable port");
                                            del_badvpn_port_in_db(&sqlite_conn, port as u16).expect("error on delete port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("specify a valid action [--enable-port, --disable-port]");
                        }
                    }
                },
                "openvpn" => {
                    let action_arg = args.get(3).unwrap();
                    match action_arg.as_str() {
                        "--enable" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if is_port_available(port).expect("error on check port use") {
                                            enable_openvpn(port).expect("error on enable port");
                                            add_openvpn_port_in_db(&sqlite_conn, port as u16).expect("error on insert port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        "--disable" => {
                            disable_openvpn().expect("error on disable port");
                            del_openvpn_port_in_db(&sqlite_conn).expect("error on delete port in db");
                        }
                        _ => {
                            println!("specify a valid action [--enable, --disable]");
                        }
                    }
                },
                "checkuser" => {
                    let action_arg = args.get(3).unwrap();
                    match action_arg.as_str() {
                        "--enable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if is_port_available(port).expect("error on check port use") {
                                            add_checkuser_port(port).expect("error on enable port");
                                            add_checkuser_port_in_db(&sqlite_conn, port as u16).expect("error on insert port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        "--disable-port" => {
                            if let Some(port_str) = args.get(4) {
                                match port_str.parse::<usize>() {
                                    Ok(port) => {
                                        if !is_port_available(port).expect("error on check port use") {
                                            del_checkuser_port(port).expect("error on disable port");
                                            del_checkuser_port_in_db(&sqlite_conn, port as u16).expect("error on delete port in db");
                                        }
                                    }
                                    Err(_) => {
                                        println!("invalid port");
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("specify a valid action [--enable-port, --disable-port]");
                        }
                    }
                },
                _ => {
                    println!("specify a valid connection [proxy, stunnel, badvpn]");
                }
            }
        } else {
            println!("it is necessary to specify a connection [proxy, stunnel, badvpn]");
        }
    } else {
        let text = "\
        Options:\n
         --conn [proxy, stunnel, badvpn, checkuser, openvpn]\n\
         --enable port (only for openvpn)\n
         --disable (only for openvpn)\n
         --enable-port port\n
         --disable-port port\n
         --status connections_status (only for proxy)\n
         --ipv6 true|false (only for stunnel)";
        println!("{}", text);
    }

    Ok(())
}