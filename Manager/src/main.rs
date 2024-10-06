mod funcs;

use std::{env, io};
use chrono::DateTime;
use mongodb::{
    sync::{Client}
};
use funcs::create_user;
use crate::funcs::{change_limit, change_pass, change_validity, enable_or_disable_proxy, expired_report_json, expired_report_vec, generate_test, get_proxy_state, is_port_avaliable, remove_user, user_already_exists, users_report_json, users_report_vec};

fn main() {

    let args: Vec<String> = env::args().collect();
    let uri = "mongodb://127.0.0.1:27017/";
    let client = Client::with_uri_str(uri).expect("error on mongodb connect");
    let database = client.database("ssh");


    if args.len() <= 1 {
        loop {
            std::process::Command::new("clear").status().unwrap();
            let mut text = " > Sim, mais um gerenciador ssh :0".to_owned();

            text = text + "\n 1 - Criar usuario";
            text = text + "\n 2 - Remover usuario";
            text = text + "\n 3 - Gerar teste";
            text = text + "\n 4 - Alterar limite";
            text = text + "\n 5 - Alterar validade";
            text = text + "\n 6 - Alterar senha";
            text = text + "\n 7 - Relatorio de usuario";
            text = text + "\n 8 - Relatorio de usuarios expirados";
            text = text + "\n 9 - Conexões";
            text = text + "\n 0 - Sair";

            println!("{}", text);

            let mut option = String::new();
            println!("\n --> Selecione uma opção:");
            io::stdin().read_line(&mut option).unwrap();

            match option.trim().parse() {
                Ok(op) => {
                    match op {
                        0 => { break }
                        1 => {
                            std::process::Command::new("clear").status().unwrap();
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
                            std::process::Command::new("clear").status().unwrap();

                            let create = create_user(&*user, &*pass, days.parse().unwrap(), limit.parse().unwrap(), true, database.clone());
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
                            std::process::Command::new("clear").status().unwrap();
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

                            let remove = remove_user(&*user, true, database.clone());
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
                            std::process::Command::new("clear").status().unwrap();
                            println!("--> função selecionada: gerar teste");
                            let mut minutes = String::new();
                            loop {
                                println!("Digite o tempo de expiração em minutos: ");
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

                            let gen = generate_test(minutes.parse().unwrap(), database.clone());
                            match gen.as_str() {
                                "error on insert user in db" => {
                                    println!("o usuario foi criado, mas ocorreu um erro para salvar ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }
                                _ => {
                                    if gen.contains("user: ") {
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
                            std::process::Command::new("clear").status().unwrap();
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

                            let change = change_limit(&*user, limit.parse().unwrap(), false, database.clone());
                            match change.as_str() {
                                "error on update user in db" => {
                                    println!("ocorreu algum erro, tente novamente\n\n> Pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }

                                _ => {
                                    if change.contains("changed") {
                                        println!(">>> Limite alterado com sucesso\n\n> Pressione qualquer tecla para voltar ao menu");
                                        let mut return_string = String::new();
                                        io::stdin().read_line(&mut return_string).expect("");
                                    }

                                }
                            }


                        }
                        5 => {
                            std::process::Command::new("clear").status().unwrap();
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

                            let change = change_validity(&*user, days.parse().unwrap(), false, database.clone());
                            match change.as_str() {
                                "error on update user in db" => {
                                    println!("a validade foi alterada, mas ocorreu algum erro ao tentar atualizar ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }

                                _ => {
                                    if change.contains("changed") {
                                        println!(">>> Validade alterada com sucesso\n\n> Pressione qualquer tecla para voltar ao menu");
                                        let mut return_string = String::new();
                                        io::stdin().read_line(&mut return_string).expect("");
                                    }
                                }
                            }
                        }
                        6 => {
                            std::process::Command::new("clear").status().unwrap();
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


                            let change = change_pass(&*user, &*pass, false, database.clone());
                            match change.as_str() {
                                "error on update user in db" => {
                                    println!("a senha foi alterada, mas ocorreu algum erro ao tentar atualizar ele na db\n\n> Pressione qualquer tecla para voltar ao menu");
                                    let mut return_string = String::new();
                                    io::stdin().read_line(&mut return_string).expect("");
                                }

                                _ => {
                                    if change.contains("changed") {
                                        println!(">>> Senha alterada com sucesso\n\n> Pressione qualquer tecla para voltar ao menu");
                                        let mut return_string = String::new();
                                        io::stdin().read_line(&mut return_string).expect("");
                                    }
                                }
                            }
                        }
                        7 => {
                            std::process::Command::new("clear").status().unwrap();
                            println!("--> função selecionada: relatorio de usuarios");
                            let users = users_report_vec(database.clone());
                            for user in users {
                                println!("Usuario: {} | Senha: {} | Limite: {} | Expira em: {}", user.user, user.pass, user.limit, DateTime::parse_from_str(&user.expiry, "%Y-%m-%d %H:%M:%S%.3f %z").unwrap().format("%Y-%m-%d"));
                            }
                            println!("\n> Pressione qualquer tecla para voltar ao menu");
                            let mut return_string = String::new();
                            io::stdin().read_line(&mut return_string).expect("");

                        }
                        8 => {
                            std::process::Command::new("clear").status().unwrap();
                            println!("--> função selecionada: relatorio de usuarios expirados");
                            let expired = expired_report_vec(database.clone());
                            for user in expired {
                                println!("Usuario: {} | Senha: {} | Limite: {} | Expira em: {}", user.user, user.pass, user.limit, DateTime::parse_from_str(&user.expiry, "%Y-%m-%d %H:%M:%S%.3f %z").unwrap().format("%Y-%m-%d"));
                            }
                            println!("\n> Pressione qualquer tecla para voltar ao menu");
                            let mut return_string = String::new();
                            io::stdin().read_line(&mut return_string).expect("");
                        }
                        9 => {
                            loop {
                                std::process::Command::new("clear").status().unwrap();
                                println!("--> função selecionada: conexões");
                                let proxy = get_proxy_state(database.clone());
                                if proxy.enabled {
                                    println!(" 1 - HttpProxy: {} | Porta: {}", proxy.enabled, proxy.port);
                                } else {
                                    println!(" 1 - HttpProxy")
                                }
                                println!(" 0 - Voltar ao menu");
                                let mut option = String::new();
                                println!("\n --> Selecione uma opção:");
                                io::stdin().read_line(&mut option).unwrap();

                                match option.trim().parse() {
                                    Ok(op) => {
                                        match op {
                                            1 => {
                                                if proxy.enabled {
                                                    std::process::Command::new("clear").status().unwrap();
                                                    println!("desativando, aguarde...");
                                                    match enable_or_disable_proxy(0, database.clone()) {
                                                        Ok(_) => {
                                                            std::process::Command::new("clear").status().unwrap();
                                                            println!("\n> Desativado com sucesso, pressione qualquer tecla para voltar ao menu");
                                                            let mut return_string = String::new();
                                                            io::stdin().read_line(&mut return_string).expect("");
                                                        }
                                                        Err(_) => {
                                                            std::process::Command::new("clear").status().unwrap();
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

                                                    match enable_or_disable_proxy(port.parse::<usize>().unwrap(), database.clone()) {
                                                        Ok(_) => {
                                                            std::process::Command::new("clear").status().unwrap();
                                                            println!("\n> Ativado com sucesso, pressione qualquer tecla para voltar ao menu");
                                                            let mut return_string = String::new();
                                                            io::stdin().read_line(&mut return_string).expect("");
                                                        }
                                                        Err(_) => {
                                                            std::process::Command::new("clear").status().unwrap();
                                                            println!("\n> Algo deu errado, pressione qualquer tecla para voltar ao menu");
                                                            let mut return_string = String::new();
                                                            io::stdin().read_line(&mut return_string).expect("");
                                                        }
                                                    }




                                                }
                                            }
                                            0 => {
                                                break
                                            }
                                            _ => {
                                                std::process::Command::new("clear").status().unwrap();
                                                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                                                let mut return_string = String::new();
                                                io::stdin().read_line(&mut return_string).expect("");
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        std::process::Command::new("clear").status().unwrap();
                                        println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                                        let mut return_string = String::new();
                                        io::stdin().read_line(&mut return_string).expect("");
                                    }
                                }


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

                let string = create_user(&args[2], &args[3], days.parse().unwrap(), limit.parse().unwrap(), false, database);
                println!("{}", string)

            }
            "--remove-user" => {
                if 2 >= args.len() {
                    println!("user empty");
                    return;
                }
                let string = remove_user(&args[2], false, database);
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

                let string = generate_test(days.parse().unwrap(), database);
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

                let string = change_limit(&args[2], limit.parse().unwrap(), false, database);
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

                let string = change_validity(&args[2], days.parse().unwrap(), false, database);
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


                let string = change_pass(&args[2], &args[3], false, database);
                println!("{}", string);
            }

            "--users-report" => {
                let string = users_report_json(database);
                println!("{}", string);
            }

            "--expired-report" => {
                let string = expired_report_json(database);
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