#!/bin/bash

# RustyManager Installer

if [ "$EUID" -ne 0 ]; then
    echo "EXECUTE COMO ROOT"
    exit 1
fi


# ---->>>> Instalação do MongoDB
apt update -y
apt-get install gnupg curl build-essential git -y

VERSION=$(lsb_release -rs)
case $VERSION in
    24.*)
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-8.0.gpg ] https://repo.mongodb.org/apt/ubuntu noble/mongodb-org/8.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-8.0.list
        ;;
    22.*)
        echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-8.0.gpg ] https://repo.mongodb.org/apt/ubuntu jammy/mongodb-org/8.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-8.0.list
        ;;
    20.*)
       echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-8.0.gpg ] https://repo.mongodb.org/apt/ubuntu focal/mongodb-org/8.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-8.0.list
        ;;
    *)
        echo "Versão do Ubuntu não suportada, use o 20, 22, ou 24"
        exit 1
        ;;
esac

apt update -y
apt-get install -y mongodb-org
systemctl daemon-reload
systemctl start mongod
systemctl enable mongod
mongo --eval 'db = connect("localhost:27017/ssh"); db.createCollection("users");'


# ---->>>> Instalar rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# ---->>>> Instalar o RustyManager
mkdir /opt/
mkdir /opt/rustymanager
git clone https://github.com/UlekBR/RustyManager.git

cd /root/RustyManager/Manager
cargo build --release
mv ./target/release/SshScript /opt/rustymanager/manager

cd /root/RustyManager/HttpProxy
cargo build --release
mv ./target/release/HttpProxy /opt/rustymanager/proxy

chmod +x /opt/rustymanager/manager
chmod +x /opt/rustymanager/proxy

ln -sf /opt/rustymanager/manager /usr/local/bin/menu


