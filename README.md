# RustyManager

**RustyManager** é um gerenciador de usuários SSH leve e eficiente, desenvolvido em Rust. Com ele, você pode criar, remover e administrar usuários SSH de forma simples e segura.

## Instalação

Para instalar o RustyManager, execute o seguinte comando no seu terminal:

```bash
bash <(wget -qO- https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/main/install.sh)
```

Após a instalação, você pode iniciar o gerenciador executando o comando:

```bash
menu
```

## Documentação para Desenvolvedores

Se você estiver desenvolvendo ou quiser entender melhor as funcionalidades do RustyManager, aqui está a documentação dos comandos disponíveis. Você pode visualizar essa documentação usando a opção `--help`:

```bash
/opt/rustymanager/manager --help
```

### Comandos Disponíveis

- **`--create-user <user> <pass> <days> <limit>`**  
Cria um novo usuário SSH com a senha, validade e limite especificados.

- **`--remove-user <user>`**  
Remove um usuário SSH existente.

- **`--generate-test <time in minutes>`**  
Gera um teste de conexão SSH por um tempo especificado em minutos.

- **`--change-limit <user> <limit>`**  
Altera o limite de conexões para o usuário especificado.

- **`--change-validity <user> <validity in days>`**  
Altera o período de validade da conta do usuário especificado.

- **`--change-pass <user> <pass>`**  
Altera a senha do usuário especificado.

- **`--users-report`**  
Gera um relatório de todos os usuários SSH cadastrados.

- **`--expired-report`**  
Gera um relatório de usuários cujo acesso expirou.

- **`--online-report`**  
  Gera um relatório de usuários online.

- **`--userdata <user>`**  
  Retorna os dados de um usuario.

## Grupo de atualizações
https://t.me/rustymanager

## Suporte
https://t.me/rustymanagergroup
