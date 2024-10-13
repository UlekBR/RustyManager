#!/bin/bash

# RustyManager Installer

if [ "$EUID" -ne 0 ]; then
    echo "EXECUTE COMO ROOT"
else
    echo "INICIANDO..."
    export DEBIAN_FRONTEND=noninteractive
    apt update -y

    SCRIPT_VERSION="main"
    # ---->>>> Verificação do sistema
    if ! command -v lsb_release &> /dev/null; then
        apt install lsb-release -y
        return 
    fi

    # ---->>>> Verificação do sistema
    clear
    OS_NAME=$(lsb_release -is)
    VERSION=$(lsb_release -rs)

    case $OS_NAME in
        Ubuntu)
            case $VERSION in
                24.*|22.*|20.*|18.*)
                    echo "Sistema suportado, vamos lá !"
                    ;;
                *)
                    echo "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
                    return
                    ;;
            esac
            ;;
        Debian)
            case $VERSION in
                12*|11*|10*|9*)
                    echo "Sistema suportado, vamos lá !"
                    ;;
                *)
                    echo "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
                    return
                    ;;
            esac
            ;;
        *)
            echo "Sistema não suportado. Use Ubuntu ou Debian."
            return
            ;;
    esac

    # ---->>>> Instalação de pacotes requisitos e atualização do sistema
    apt upgrade -y
    apt-get install gnupg curl build-essential git cmake sqlite3 -y


    # ---->>>> Criando as colunas no banco de dados
    sqlite3 /opt/rustymanager/db "
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        login_type TEXT NOT NULL,
        login_user TEXT NOT NULL,
        login_pass TEXT NOT NULL,
        login_limit TEXT NOT NULL,
        login_expiry TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS connections (
        id INTEGER PRIMARY KEY,
        http_proxy_enabled BOOLEAN,
        http_proxy_port INTEGER,
        stunnel_enabled BOOLEAN,
        stunnel_port INTEGER,
        badvpn_ports TEXT
    );
    "


    # ---->>>> Instalar rust
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        . "$HOME/.cargo/env"
    fi

    # ---->>>> Instalar o RustyManager
    mkdir /opt/
    mkdir /opt/rustymanager
    git clone --branch "$SCRIPT_VERSION" --recurse-submodules --single-branch https://github.com/UlekBR/RustyManager.git /root/RustyManager

    cd /root/RustyManager/
    cargo build --release --jobs $(nproc)
    mv ./target/release/SshScript /opt/rustymanager/manager
    mv ./target/release/HttpProxy /opt/rustymanager/proxy
    mv ./target/release/BadVpnManager /opt/rustymanager/badmanager


    mkdir /root/RustyManager/BadVpn/badvpn/badvpn-build
    cd  /root/RustyManager/BadVpn/badvpn/badvpn-build
    cmake .. -DBUILD_NOTHING_BY_DEFAULT=1 -DBUILD_UDPGW=1
    make
    mv udpgw/badvpn-udpgw /opt/rustymanager/badvpn

    cd ../../../
    chmod +x /opt/rustymanager/manager
    chmod +x /opt/rustymanager/proxy
    chmod +x /opt/rustymanager/badmanager
    chmod +x /opt/rustymanager/badvpn
    ln -sf /opt/rustymanager/manager /usr/local/bin/menu

    # ---->>>> Criar o serviço do proxy
    SERVICE_FILE_CONTENT="
    [Unit]
    Description=HttpProxy
    After=network.target

    [Service]
    LimitNOFILE=infinity
    Type=simple
    ExecStart=/opt/rustymanager/proxy
    Restart=always
    StandardOutput=syslog
    StandardError=syslog
    SyslogIdentifier=proxy
    User=root
    Environment=PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
    Environment=HOME=/root
    WorkingDirectory=/opt/rustymanager

    [Install]
    WantedBy=multi-user.target
    "
    SERVICE_FILE="/etc/systemd/system/proxy.service"
    echo "$SERVICE_FILE_CONTENT" | tee "$SERVICE_FILE" > /dev/null
    systemctl daemon-reload > /dev/null

    # ---->>>> Instalando STunnel
    apt install -y stunnel4

    # baixando certificado
    wget -O /etc/stunnel/cert.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/cert.pem
    wget -O /etc/stunnel/key.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/key.pem

    # colocando o enable para os serviços do stunnel
    sed -i 's/ENABLED=0/ENABLED=1/g' /etc/default/stunnel4

    # setar como desativado por padrão
    systemctl stop stunnel4
    systemctl disable stunnel4

    # ---->>>> Removendo o diretorio do Instalador RustyManager
    rm -rf /root/RustyManager/

    # ---->>>> Instalação finalizada :)
    clear
    echo "digite menu para acessar o menu"

fi

