#!/bin/bash

# RustyManager Installer

if [ "$EUID" -ne 0 ]; then
    echo "EXECUTE COMO ROOT"
    exit 1
fi

echo "INICIANDO..."

SCRIPT_VERSION="beta"

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
                REPO_TYPE="multiverse"
                MONGO_VERSION="8.0"
                ;;
            22.*)
                echo "Versão suportada: Ubuntu 22"
                REPO_SYSTEM="ubuntu"
                REPO_URL="jammy"
                REPO_TYPE="multiverse"
                MONGO_VERSION="8.0"
                ;;
            20.*)
                echo "Versão suportada: Ubuntu 20"
                REPO_SYSTEM="ubuntu"
                REPO_URL="focal"
                REPO_TYPE="multiverse"
                MONGO_VERSION="8.0"
                ;;
            18.*)
                echo "Versão suportada: Ubuntu 18"
                REPO_SYSTEM="ubuntu"
                REPO_URL="bionic"
                REPO_TYPE="multiverse"
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
            12*)
                echo "Versão suportada: Debian 12"
                REPO_SYSTEM="debian"
                REPO_URL="bookworm"
                REPO_TYPE="main"
                MONGO_VERSION="8.0"
                ;;
            11*)
                echo "Versão suportada: Debian 11"
                REPO_SYSTEM="debian"
                REPO_URL="bullseye"
                REPO_TYPE="main"
                MONGO_VERSION="7.0"
                ;;
            10*)
                echo "Versão suportada: Debian 10"
                REPO_SYSTEM="debian"
                REPO_URL="buster"
                REPO_TYPE="main"
                MONGO_VERSION="6.0"
                ;;
            9*)
                echo "Versão suportada: Debian 9"
                REPO_SYSTEM="debian"
                REPO_URL="stretch"
                REPO_TYPE="main"
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
        wget -qO - https://www.mongodb.org/static/pgp/server-$MONGO_VERSION.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/$MONGO_VERSION $REPO_TYPE" | tee /etc/apt/sources.list.d/mongodb-org-$MONGO_VERSION.list
        ;;
    7.0)
        wget -qO - https://www.mongodb.org/static/pgp/server-$MONGO_VERSION.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/$MONGO_VERSION $REPO_TYPE" | tee /etc/apt/sources.list.d/mongodb-org-$MONGO_VERSION.list
        ;;
    6.0)
        wget -qO - https://www.mongodb.org/static/pgp/server-$MONGO_VERSION.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/$MONGO_VERSION $REPO_TYPE" | tee /etc/apt/sources.list.d/mongodb-org-$MONGO_VERSION.list
        ;;
    5.0)
        wget -qO - https://www.mongodb.org/static/pgp/server-$MONGO_VERSION.asc | gpg --dearmor --yes -o /usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-$MONGO_VERSION.gpg ] https://repo.mongodb.org/apt/$REPO_SYSTEM $REPO_URL/mongodb-org/$MONGO_VERSION $REPO_TYPE" | tee /etc/apt/sources.list.d/mongodb-org-$MONGO_VERSION.list
        ;;
    *)
        echo "Versão do MongoDB não suportada."
        exit 1
        ;;
esac
apt update -y
apt-get install -y mongodb-org
systemctl daemon-reload
systemctl enable mongod
systemctl start mongod
sleep 1
mongosh --eval '
const db = connect("mongodb://localhost:27017/ssh");
function ensureFieldsExist() {
    db.connections.updateMany(
        { proxy: { $exists: false } },
        {
            $set: {
                proxy: { enabled: false, port: 0 }
            }
        }
    );
    db.connections.updateMany(
        { stunnel: { $exists: false } },
        {
            $set: {
                stunnel: { enabled: false, port: 0 }
            }
        }
    );
    db.connections.updateMany(
        { badvpn: { $exists: false } },
        {
            $set: {
                badvpn: { ports: [] }
            }
        }
    );
}
if (!db.getCollectionNames().includes("connections")) {
    db.createCollection("connections");
}
ensureFieldsExist();
'

# ---->>>> Instalar rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

# ---->>>> Instalar o RustyManager
mkdir /opt/
mkdir /opt/rustymanager
git clone --branch $SCRIPT_VERSION --recurse-submodules --single-branch https://github.com/UlekBR/RustyManager.git

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
echo "$SERVICE_FILE_CONTENT" | sudo tee "$SERVICE_FILE" > /dev/null
sudo systemctl daemon-reload > /dev/null

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
