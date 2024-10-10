#!/bin/bash

# RustyManager Installer

if [ "$EUID" -ne 0 ]; then
    echo "EXECUTE COMO ROOT"
    exit 1
fi

echo "INICIANDO..."

# ---->>>> Verificação do sistema
OS_NAME=$(lsb_release -is)
VERSION=$(lsb_release -rs)

case $OS_NAME in
    Ubuntu)
        case $VERSION in
            24.*)
                echo "Versão suportada: Ubuntu 24"
                REPO_SYSTEM="ubuntu"
                REPO_URL="noble"
                MONGO_VERSION="8.0"
                ;;
            22.*)
                echo "Versão suportada: Ubuntu 22"
                REPO_SYSTEM="ubuntu"
                REPO_URL="jammy"
                MONGO_VERSION="8.0"
                ;;
            20.*)
                echo "Versão suportada: Ubuntu 20"
                REPO_SYSTEM="ubuntu"
                REPO_URL="focal"
                MONGO_VERSION="8.0"
                ;;
            18.*)
                echo "Versão suportada: Ubuntu 18"
                REPO_SYSTEM="ubuntu"
                REPO_URL="bionic"
                MONGO_VERSION="6.0"
                ;;
            *)
                echo "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
                exit 1
                ;;
        esac
        ;;
    Debian)
        case $VERSION in
            12.*)
                echo "Versão suportada: Debian 12"
                REPO_SYSTEM="debian"
                REPO_URL="bookworm"
                MONGO_VERSION="8.0"
                ;;
            11.*)
                echo "Versão suportada: Debian 11"
                REPO_SYSTEM="debian"
                REPO_URL="bullseye"
                MONGO_VERSION="7.0"
                ;;
            10.*)
                echo "Versão suportada: Debian 10"
                REPO_SYSTEM="debian"
                REPO_URL="buster"
                MONGO_VERSION="6.0"
                ;;
            9.*)
                echo "Versão suportada: Debian 9"
                REPO_SYSTEM="debian"
                REPO_URL="stretch"
                MONGO_VERSION="5.0"
                ;;
            *)
                echo "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Sistema não suportado. Use Ubuntu ou Debian."
        exit 1
        ;;
esac

# ---->>>> Instalação de pacotes requisitos e atualização do sistema
export DEBIAN_FRONTEND=noninteractive
apt update -y
apt upgrade -y
apt-get install gnupg curl build-essential git cmake -y

# ---->>>> Instalação do MongoDB
case $MONGO_VERSION in
    8.0)
        wget -qO - https://www.mongodb.org/static/pgp/server-8.0.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-8.0.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-8.0.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/$MONGO_VERSION multiverse" | tee /etc/apt/sources.list.d/mongodb-org-$MONGO_VERSION.list
        ;;
    7.0)
        wget -qO - https://www.mongodb.org/static/pgp/server-7.0.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-7.0.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/$MONGO_VERSION multiverse" | tee /etc/apt/sources.list.d/mongodb-org-$MONGO_VERSION.list
        ;;
    6.0)
        wget -qO - https://www.mongodb.org/static/pgp/server-6.0.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-6.0.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-6.0.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/6.0 multiverse" | tee /etc/apt/sources.list.d/mongodb-org-6.0.list
        ;;
    5.0)
        wget -qO - https://www.mongodb.org/static/pgp/server-5.0.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-5.0.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-5.0.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/5.0 multiverse" | tee /etc/apt/sources.list.d/mongodb-org-5.0.list
        ;;
    *)
        echo "Versão do MongoDB não suportada."
        exit 1
        ;;
esac
apt update -y
apt-get install -y mongodb-org
systemctl daemon-reload
systemctl start mongod
systemctl enable mongod
mongosh --eval 'const db = connect("mongodb://localhost:27017/ssh"); db.createCollection("users"); db.createCollection("connections");'

# ---->>>> Instalar rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

# ---->>>> Instalar o RustyManager
mkdir /opt/
mkdir /opt/rustymanager
git clone --recurse-submodules https://github.com/UlekBR/RustyManager.git

# manager
cd /root/RustyManager/Manager
cargo build --release
mv ./target/release/SshScript /opt/rustymanager/manager

# httproxy
cd /root/RustyManager/HttpProxy
cargo build --release
mv ./target/release/HttpProxy /opt/rustymanager/proxy

# badvpn
cd /root/RustyManager/BadVpn/BadVpnManager
cargo build --release
mv ./target/release/BadVpnManager /opt/rustymanager/badmanager

cd ..
mkdir /root/RustyManager/BadVpn/badvpn/badvpn-build
cd  /root/RustyManager/BadVpn/badvpn/badvpn-build
cmake .. -DBUILD_NOTHING_BY_DEFAULT=1 -DBUILD_UDPGW=1 &
wait
make &
wait
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
LimitNOFILE=65536
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
echo "$SERVICE_FILE_CONTENT" | sudo tee "$SERVICE_FILE" > /dev/null
sudo systemctl daemon-reload > /dev/null

# ---->>>> Removendo o diretorio do RustyManager
rm -rf /root/RustyManager/

# ---->>>> Instalação finalizada XD
clear
echo "digite menu para acessar o menu"
