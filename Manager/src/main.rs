mod funcs;
mod text_funcs;

use std::{env, io, thread};
use std::process::Command;
use std::time::Duration;
use chrono::DateTime;
use rusqlite::Connection;
use crate::text_funcs::{text_to_bold, text_to_green};
use crate::funcs::{create_user, change_limit, change_pass, change_validity, enable_or_disable_proxy, expired_report_json, expired_report_vec, generate_test, get_proxy_state, is_port_avaliable, remove_user, user_already_exists, users_report_json, users_report_vec, run_command_and_get_output, get_connections, enable_badvpn_port, disable_badvpn_port, get_stunnel_state, enable_or_disable_stunnel};

fn main() {
    let sqlite_conn = Connection::open("/opt/rustymanager/db").unwrap();
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        main_menu(&sqlite_conn);
    } else{
        match (&args[1]).as_str() {
            "--create-user" => {
                match args.len() {
                    _i if 2 >= _i  => {
                        println!("user empty");
                        return;
                    }
                    _i if 3 >= _i  => {
                        println!("pass empty");
                        return;
                    }
                    _i if 4 >= _i => {
                        println!("days empty");
                        return;
                    }
                    _i if 5 >= _i => {
                        println!("limit empty");
                        return;
                    }
                    _ => {}
                }


                let days = &args[4];
                let limit = &args[5];

                match days.parse::<usize>() {
                    Ok(..) => {}
                    Err(..) => {
                        println!("invalid digit found in days");
                        return
                    }
                }
                match limit.parse::<usize>() {
                    Ok(..) => {}
                    Err(..) => {
                        println!("invalid digit found in limit");
                        return
                    }
                }

                let string = create_user(&args[2], &args[3], days.parse().unwrap(), limit.parse().unwrap(), false, &sqlite_conn);
                println!("{}", string)

            }
            "--remove-user" => {
                if 2 >= args.len() {
                    println!("user empty");
                    return;
                }
                let string = remove_user(&args[2], false, &sqlite_conn);
                println!("{}", string);
            }

            "--generate-test" => {
                if 2 >= args.len() {
                    println!("minutes empty");
                    return;
                }

                let days = &args[2];

                match days.parse::<usize>() {
                    Ok(..) => {}
                    Err(..) => {
                        println!("invalid digit found in days");
                        return
                    }
                }

                let string = generate_test(days.parse().unwrap(), &sqlite_conn);
                println!("{}", string);
            }
            "--change-limit" => {
                match args.len() {
                    _i if 2 >= _i  => {
                        println!("user empty");
                        return;
                    }
                    _i if 3 >= _i  => {
                        println!("limit empty");
                        return;
                    }
                    _ => {}
                }

                let limit = &args[3];

                match limit.parse::<usize>() {
                    Ok(..) => {}
                    Err(..) => {
                        println!("invalid digit found in limit");
                        return
                    }
                }

                let string = change_limit(&args[2], limit.parse().unwrap(), false, &sqlite_conn);
                println!("{}", string);
            }

            "--change-validity" => {
                match args.len() {
                    _i if 2 >= _i  => {
                        println!("user empty");
                        return;
                    }
                    _i if 3 >= _i  => {
                        println!("days empty");
                        return;
                    }
                    _ => {}
                }

                let days = &args[3];

                match days.parse::<usize>() {
                    Ok(..) => {}
                    Err(..) => {
                        println!("invalid digit found in days");
                        return
                    }
                }

                let string = change_validity(&args[2], days.parse().unwrap(), false, &sqlite_conn);
                println!("{}", string);
            }
            "--change-pass" => {
                match args.len() {
                    _i if 2 >= _i  => {
                        println!("user empty");
                        return;
                    }
                    _i if 3 >= _i  => {
                        println!("pass empty");
                        return;
                    }
                    _ => {}
                }


                let string = change_pass(&args[2], &args[3], false, &sqlite_conn);
                println!("{}", string);
            }

            "--users-report" => {
                let string = users_report_json(&sqlite_conn);
                println!("{}", string);
            }

            "--expired-report" => {
                let string = expired_report_json(&sqlite_conn);
                println!("{}", string);
            }


            "--help" => {

                let mut text = " -- help data".to_owned();
                text = text + "\n   --create-user <user> <pass> <days> <limit>";
                text = text + "\n   --remove-user <user>";
                text = text + "\n   --generate-test <time in minutes>";
                text = text + "\n   --change-limit <user> <limit>";
                text = text + "\n   --change-validity <user> <validity in days>";
                text = text + "\n   --change-pass <user> <pass>";
                text = text + "\n   --users-report";
                text = text + "\n   --expired-report";

                println!("{}", text)
            }

            _ => {
                println!("função invalida selecionada")
            }
        }
    }
}

fn user_dont_exists() {
    println!("esse não existe\n\n> Pressione qualquer tecla para voltar ao menu");
    let mut return_string = String::new();
    io::stdin().read_line(&mut return_string).expect("");
}

fn user_exists() {
    println!("esse usuario já existe\n\n> Pressione qualquer tecla para voltar ao menu");
    let mut return_string = String::new();
    io::stdin().read_line(&mut return_string).expect("");
}


fn main_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        println!("{}", text_to_bold("================= RustyManager ================="));
        let os = run_command_and_get_output("lsb_release -is | tr -d '\"'");
        let version = run_command_and_get_output(" lsb_release -rs | tr -d '\"'");
        let online = run_command_and_get_output("ps -e -o user= -o cmd= | grep '[s]shd: ' | grep -v 'sshd: root@' | awk '{user=$1; if (user != \"root\") print user}' | wc -l");
        let created = run_command_and_get_output("awk -F: '$3 >= 1000 { C++ } END { print C+0 }' /etc/passwd");

        println!("------------------------------------------------");
        println!("| {} {:<16} | {} {:<4} |", text_to_bold("Os:"), os, text_to_bold("Usuarios Criado:"), created);
        println!("| {} {:<12} | {} {:<4} |", text_to_bold("Versão:"), version, text_to_bold("Usuarios Online:"), online);
        println!("------------------------------------------------");
        let options = vec![
            "Gerenciar Usuarios",
            "Gerenciar Conexões",
        ];

        for (i, option) in options.iter().enumerate() {
            println!("| {:02} - {:<39} |", i + 1, option);
        }
        println!("| 00 - {:<39} |", "Sair");
        println!("------------------------------------------------");
        println!("\n --> Selecione uma opção:");

        let mut option = String::new();
        io::stdin().read_line(&mut option).unwrap();


        match option.trim().parse() {
            Ok(op) => {
                match op {
                    0 => { break }
                    1 => {
                        users_menu(&sqlite_conn);
                    }
                    2 => {
                        connection_menu(&sqlite_conn);
                    }

                    _ => {}
                }
            }
            Err(err) => {
                println!("{}", err);
                println!("Selecione uma opção valida")
            }
        }
    }
}
fn users_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        println!("{}", text_to_bold("================= RustyManager ================="));
        let online = run_command_and_get_output("ps -e -o user= -o cmd= | grep '[s]shd: ' | grep -v 'sshd: root@' | awk '{user=$1; if (user != \"root\") print user}' | wc -l");
        let created = run_command_and_get_output("awk -F: '$3 >= 1000 { C++ } END { print C+0 }' /etc/passwd");
        println!("------------------------------------------------");
        println!("| {} {:<12} | {} {:<12} |", text_to_bold("Online:"), online, text_to_bold("Criados:"), created);
        println!("------------------------------------------------");
        println!("|              {}              |", text_to_bold("Gerenciar Usuarios"));
        println!("------------------------------------------------");
        let options = vec![
            "Criar usuario",
            "Remover usuario",
            "Gerar teste",
            "Alterar limite",
            "Alterar validade",
            "Alterar senha",
            "Relatorio de usuario",
            "Relatorio de usuarios expirados",
            "Relatorio de usuarios conectados"
        ];

        for (i, option) in options.iter().enumerate() {
            println!("| {:02} - {:<39} |", i + 1, option);
        }
        println!("| 00 - {:<39} |", "Sair");
        println!("------------------------------------------------");
        println!("\n --> Selecione uma opção:");

        let mut option = String::new();
        io::stdin().read_line(&mut option).unwrap();


        match option.trim().parse() {
            Ok(op) => {
                match op {
                    0 => { break }
                    1 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: criar usuario");
                        println!("Digite o usuario: ");
                        let mut user = String::new();
                        io::stdin().read_line(&mut user).unwrap();
                        user = user.trim().to_string();
                        if user.is_empty() {
                            continue;
                        }

                        if user_already_exists(user.as_str()) {
                            user_exists();
                            continue
                        }

                        println!("Digite a senha: ");
                        let mut pass = String::new();
                        io::stdin().read_line(&mut pass).unwrap();
                        pass = pass.trim().to_string();
                        if pass.is_empty() {
                            continue;
                        }

                        let mut days = String::new();
                        loop {
                            println!("Digite a expiração em dias: ");
                            if !days.is_empty() {
                                days = String::new()
                            }
                            io::stdin().read_line(&mut days).unwrap();
                            days = days.trim().to_string();
                            match days.parse::<usize>() {
                                Ok(..) => {
                                    break
                                }
                                Err(..) => {
                                    println!("digite um numero valido");
                                }
                            }
                        }

                        let mut limit = String::new();
                        loop {
                            println!("Digite o limite de conexões: ");
                            if !limit.is_empty() {
                                limit = String::new()
                            }
                            io::stdin().read_line(&mut limit).unwrap();
                            limit = limit.trim().to_string();
                            match limit.parse::<usize>() {
                                Ok(..) => {
                                    break
                                }
                                Err(..) => {
                                    println!("digite um numero valido");
                                }
                            }
                        }
                        Command::new("clear").status().unwrap();

                        let create = create_user(&*user, &*pass, days.parse().unwrap(), limit.parse().unwrap(), true, &sqlite_conn);
                        match create.as_str() {
                            "created" => {
                                let mut text = ">>> Usuario criado com sucesso".to_owned();
                                text = text + "\n - Usuario: " + &*user;
                                text = text + "\n - Senha: " + &*pass;
                                text = text + "\n - Dias para expirar: " + &*days;
                                text = text + "\n - Limite de conexões: " + &*limit;
                                text = text + "\n\n> Pressione qualquer tecla para voltar ao menu";
                                println!("{}", text);
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }

                            "error on insert user in db" => {
                                println!("o usuario foi criado, mas ocorreu um erro para salvar ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            _ => {}
                        }
                    }
                    2 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: remover usuario");
                        println!("Digite o usuario: ");
                        let mut user = String::new();
                        io::stdin().read_line(&mut user).unwrap();
                        user = user.trim().to_string();
                        if user.is_empty() {
                            continue;
                        }

                        if !user_already_exists(user.as_str()) {
                            user_dont_exists();
                            continue
                        }

                        let remove = remove_user(&*user, true, &sqlite_conn);
                        match remove.as_str() {
                            "removed" => {
                                println!(">>> Usuario removido com sucesso\n\n> Pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            "error on remove user at db" => {
                                println!("o usuario foi removido, mas ocorreu um erro ao tentar remover ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            _ => {}
                        }
                    }
                    3 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: gerar teste");
                        let mut minutes = String::new();
                        loop {
                            println!("Digite o tempo de expiração em minutos: ");
                            if !minutes.is_empty() {
                                minutes = String::new()
                            }
                            io::stdin().read_line(&mut minutes).unwrap();
                            minutes = minutes.trim().to_string();
                            match minutes.parse::<usize>() {
                                Ok(..) => {
                                    break
                                }
                                Err(..) => {
                                    println!("digite um numero valido");
                                }
                            }
                        }

                        let gen = generate_test(minutes.parse().unwrap(), &sqlite_conn);
                        match gen.as_str() {
                            "error on insert user in db" => {
                                Command::new("clear").status().unwrap();
                                println!("o teste foi criado, mas ocorreu um erro para salvar ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            _ => {
                                if gen.contains("user: ") {
                                    Command::new("clear").status().unwrap();
                                    let mut text = ">>> Teste gerado com sucesso".to_owned();
                                    let user = gen.split("user: ").collect::<Vec<&str>>()[1].split(" |").collect::<Vec<&str>>()[0];
                                    text = text + "\n - Usuario: " + &*user;
                                    text = text + "\n - Senha: " + &*user;
                                    text = text + "\n - Expira em: " + &*minutes + " minutos";
                                    text = text + "\n - Limite de conexões: 1";
                                    text = text + "\n\n> Pressione qualquer tecla para voltar ao menu";
                                    println!("{}", text);
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }

                            }
                        }
                    }
                    4 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: alterar limite");
                        println!("Digite o usuario: ");
                        let mut user = String::new();
                        io::stdin().read_line(&mut user).unwrap();
                        user = user.trim().to_string();
                        if user.is_empty() {
                            continue;
                        }

                        if !user_already_exists(user.as_str()) {
                            user_dont_exists();
                            continue
                        }

                        let mut limit = String::new();
                        loop {
                            println!("Digite o novo limit: ");
                            if !limit.is_empty() {
                                limit = String::new()
                            }
                            io::stdin().read_line(&mut limit).unwrap();
                            limit = limit.trim().to_string();
                            match limit.parse::<usize>() {
                                Ok(..) => {
                                    break
                                }
                                Err(..) => {
                                    println!("digite um numero valido");
                                }
                            }
                        }

                        let change = change_limit(&*user, limit.parse().unwrap(), false, &sqlite_conn);
                        match change.as_str() {
                            "error on update user in db" => {
                                Command::new("clear").status().unwrap();
                                println!("ocorreu algum erro, tente novamente\n\n> Pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }

                            _ => {
                                if change.contains("changed") {
                                    Command::new("clear").status().unwrap();
                                    println!(">>> Limite alterado com sucesso\n\n> Pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }

                            }
                        }


                    }
                    5 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: alterar validade");
                        println!("Digite o usuario: ");
                        let mut user = String::new();
                        io::stdin().read_line(&mut user).unwrap();
                        user = user.trim().to_string();
                        if user.is_empty() {
                            continue;
                        }

                        if !user_already_exists(user.as_str()) {
                            user_dont_exists();
                            continue
                        }

                        let mut days = String::new();
                        loop {
                            println!("Digite a nova validade em dias: ");
                            if !days.is_empty() {
                                days = String::new()
                            }
                            io::stdin().read_line(&mut days).unwrap();
                            days = days.trim().to_string();
                            match days.parse::<usize>() {
                                Ok(..) => {
                                    break
                                }
                                Err(..) => {
                                    println!("digite um numero valido");
                                }
                            }
                        }

                        let change = change_validity(&*user, days.parse().unwrap(), false, &sqlite_conn);
                        match change.as_str() {
                            "error on update user in db" => {
                                Command::new("clear").status().unwrap();
                                println!("a validade foi alterada, mas ocorreu algum erro ao tentar atualizar ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }

                            _ => {
                                if change.contains("changed") {
                                    Command::new("clear").status().unwrap();
                                    println!(">>> Validade alterada com sucesso\n\n> Pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                            }
                        }
                    }
                    6 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: alterar senha");
                        println!("Digite o usuario: ");
                        let mut user = String::new();
                        io::stdin().read_line(&mut user).unwrap();
                        user = user.trim().to_string();
                        if user.is_empty() {
                            continue;
                        }

                        if !user_already_exists(user.as_str()) {
                            user_dont_exists();
                            continue
                        }

                        let mut pass = String::new();
                        println!("Digite a nova senha: ");
                        io::stdin().read_line(&mut pass).unwrap();
                        pass = pass.trim().to_string();


                        let change = change_pass(&*user, &*pass, false, &sqlite_conn);
                        match change.as_str() {
                            "error on update user in db" => {
                                Command::new("clear").status().unwrap();
                                println!("a senha foi alterada, mas ocorreu algum erro ao tentar atualizar ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }

                            _ => {
                                if change.contains("changed") {
                                    Command::new("clear").status().unwrap();
                                    println!(">>> Senha alterada com sucesso\n\n> Pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                            }
                        }
                    }
                    7 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: relatorio de usuarios");
                        let users = users_report_vec(&sqlite_conn);
                        for user in users {
                            println!("Usuario: {} | Senha: {} | Limite: {} | Expira em: {}", user.user, user.pass, user.limit, DateTime::parse_from_str(&user.expiry, "%Y-%m-%d %H:%M:%S%.3f %z").unwrap().format("%Y-%m-%d"));
                        }
                        println!("\n> Pressione qualquer tecla para voltar ao menu");
                        let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");

                    }
                    8 => {
                        Command::new("clear").status().unwrap();
                        println!("--> função selecionada: relatorio de usuarios expirados");
                        let expired = expired_report_vec(&sqlite_conn);
                        for user in expired {
                            println!("Usuario: {} | Senha: {} | Limite: {} | Expira em: {}", user.user, user.pass, user.limit, DateTime::parse_from_str(&user.expiry, "%Y-%m-%d %H:%M:%S%.3f %z").unwrap().format("%Y-%m-%d"));
                        }
                        println!("\n> Pressione qualquer tecla para voltar ao menu");
                        let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");
                    }
                    9 => {
                        loop {
                            Command::new("clear").status().unwrap();
                            println!("Monitorando usuários conectados via SSH");
                            println!("------------------------------------------");
                            println!("Usuário           | Conexões");
                            println!("--------------------------");

                            let output = run_command_and_get_output("ps -e -o user= -o cmd= | grep '[s]shd: ' | grep -v 'sshd: root@'");

                            let connections = String::from_utf8_lossy(output.as_ref());
                            let mut user_connections: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
                            for line in connections.lines() {
                                let user = line.split_whitespace().next().unwrap_or("");
                                if user != "root" {
                                    *user_connections.entry(user).or_insert(0) += 1;
                                }
                            }

                            for (user, count) in user_connections.iter() {
                                println!("{:<18} | {}", user, count);
                            }

                            let total_connections: usize = user_connections.values().sum();
                            if total_connections != 0 {
                                println!("--------------------------");
                            }
                            println!("Total de conexões: {}", total_connections);

                            thread::sleep(Duration::from_secs(1));
                        }
                    }
                    _ => {}
                }
            }
            Err(err) => {
                println!("{}", err);
                println!("Selecione uma opção valida")
            }
        }
    }
}

fn connection_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        println!("{}", text_to_bold("================= RustyManager ================="));
        println!("------------------------------------------------");
        println!("|              {}              |", text_to_bold("Gerenciar Conexões"));
        println!("------------------------------------------------");
        let proxy = get_proxy_state(&sqlite_conn).unwrap();
        let proxy_enable = proxy.enabled.expect("error on get proxy status");

        if proxy_enable {
            println!("| 1 - RustyProxy (ws/wss/socks): {:<21}  |", text_to_green("ativo"));
        } else {
            println!("| 1 - {:<40} |", "HttpProxy")
        }
        let stunnel = get_stunnel_state(&sqlite_conn).unwrap();
        let stunnel_enable = stunnel.enabled.expect("error on get proxy status");
        if stunnel_enable  {
            println!("| 2 - Stunnel: {:<40} |", text_to_green("ativo"));
        } else {
            println!("| 2 - {:<40} |", "Stunnel")
        }
        println!("| 3 - {:<40} |", "Badvpn");
        println!("| 0 - {:<40} |", "Voltar ao menu");
        println!("------------------------------------------------");
        let mut option = String::new();
        println!("\n --> Selecione uma opção:");
        io::stdin().read_line(&mut option).unwrap();

        match option.trim().parse() {
            Ok(op) => {
                match op {
                    1 => {
                        if proxy_enable {
                            Command::new("clear").status().unwrap();
                            println!("desativando, aguarde...");
                            match enable_or_disable_proxy(0, &sqlite_conn) {
                                Ok(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Desativado com sucesso, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                                Err(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Algo deu errado, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                            }
                        } else {
                            let mut port = String::new();
                            loop {
                                println!("Digite uma porta: (ex: 80)");
                                io::stdin().read_line(&mut port).unwrap();
                                port = port.trim().to_string();
                                match port.parse::<usize>() {
                                    Ok(port) => {
                                        match is_port_avaliable(port) {
                                            Ok(true) => { break },
                                            _ => { println!("A porta está em uso, digite outra:") }
                                        }
                                    }
                                    Err(..) => {
                                        println!("digite uma porta valida");
                                    }
                                }
                            }
                            match enable_or_disable_proxy(port.parse::<usize>().unwrap(), &sqlite_conn) {
                                Ok(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Ativado com sucesso, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                                Err(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Algo deu errado, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                            }
                        }
                    }
                    2 => {
                        if stunnel_enable {
                            Command::new("clear").status().unwrap();
                            println!("desativando, aguarde...");
                            match enable_or_disable_stunnel(0, &sqlite_conn) {
                                Ok(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Desativado com sucesso, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                                Err(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Algo deu errado, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                            }
                        } else {
                            let mut port = String::new();
                            loop {
                                println!("Digite uma porta: (ex: 443)");
                                io::stdin().read_line(&mut port).unwrap();
                                port = port.trim().to_string();
                                match port.parse::<usize>() {
                                    Ok(port) => {
                                        match is_port_avaliable(port) {
                                            Ok(true) => { break },
                                            _ => { println!("A porta está em uso, digite outra:") }
                                        }
                                    }
                                    Err(..) => {
                                        println!("digite uma porta valida:");
                                    }
                                }
                            }

                            match enable_or_disable_stunnel(port.parse::<usize>().unwrap(), &sqlite_conn) {
                                Ok(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Ativado com sucesso, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                                Err(_) => {
                                    Command::new("clear").status().unwrap();
                                    println!("\n> Algo deu errado, pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                            }

                        }
                    }
                    3 => {
                        badvpn_menu(&sqlite_conn)
                    }
                    0 => {
                        break
                    }
                    _ => {
                        Command::new("clear").status().unwrap();
                        println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                        let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");
                    }
                }
            }
            Err(_) => {
                Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
            }
        }


    }
}

fn badvpn_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        println!("{}", text_to_bold("================= RustyManager ================="));
        println!("------------------------------------------------");
        println!("|                {}                 |", text_to_bold("Portas BadVpn"));
        println!("------------------------------------------------");
        let conn = get_connections(&sqlite_conn).unwrap();
        println!("| {:<44} |", "Portas ativas:");
        let badvpn_ports = conn.badvpn.ports.expect("error on get badvpn ports");
        if badvpn_ports.is_empty() {
            println!("|   - {:<40} |", "Nenhuma porta ativa")
        } else {
            for port in badvpn_ports {
                println!("|   - {:<40} |", port)
            }
        }
        println!("| 1 - {:<40} |", "Abrir Porta");
        println!("| 2 - {:<40} |", "Fechar Porta");
        println!("| 0 - {:<40} |", "Voltar ao menu");
        println!("------------------------------------------------");
        let mut option = String::new();
        println!("\n --> Selecione uma opção:");
        io::stdin().read_line(&mut option).unwrap();
        match option.trim().parse() {
            Ok(op) => {
                match op {
                    1 => {
                        let mut port = String::new();
                        loop {
                            println!("Digite a porta: ");
                            io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                            match port.parse::<usize>() {
                                Ok(port) => {
                                    if !is_port_avaliable(port).unwrap() {
                                        println!("essa porta já está em uso, digite outra:")
                                    } else {
                                        break
                                    }
                                }
                                Err(..) => {
                                    println!("digite uma porta valida");
                                }
                            }

                        }

                        enable_badvpn_port(port);

                        Command::new("clear").status().unwrap();
                        println!("\n> Porta ativada com sucesso, pressione qualquer tecla para voltar ao menu");
                        let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");

                    }
                    2 => {
                        let mut port = String::new();
                        loop {
                            println!("Digite a porta: ");
                            io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                            match port.parse::<usize>() {
                                Ok(port) => {
                                    if is_port_avaliable(port).unwrap() {
                                        println!("essa porta não está em uso, digite outra:")
                                    } else {
                                        break
                                    }
                                }
                                Err(..) => {
                                    println!("digite uma porta valida");
                                }
                            }

                        }

                        disable_badvpn_port(port);

                        Command::new("clear").status().unwrap();
                        println!("\n> Porta desativada com sucesso, pressione qualquer tecla para voltar ao menu");
                        let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");

                    }
                    0 => {
                        break
                    }
                    _ => {
                        continue
                    }
                }
            }
            _ => {
                Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
            }
        }
    }
}