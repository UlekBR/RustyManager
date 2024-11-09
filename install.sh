#!/bin/bash
# RustyManager Installer

TOTAL_STEPS=13
CURRENT_STEP=0

show_progress() {
    PERCENT=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo "Progresso: [${PERCENT}%] - $1"
}

error_exit() {
    echo -e "\nErro: $1"
    return
}

increment_step() {
    CURRENT_STEP=$((CURRENT_STEP + 1))
}

if [ "$EUID" -ne 0 ]; then
    error_exit "EXECUTE COMO ROOT"
else
    clear
    show_progress "Atualizando repositorios..."
    export DEBIAN_FRONTEND=noninteractive
    SCRIPT_VERSION="beta"
    increment_step

    # ---->>>> Verificação do sistema
    show_progress "Verificando o sistema..."
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS_NAME=$ID
        VERSION=$VERSION_ID
    else
        error_exit "Não foi possível detectar o sistema operacional."
    fi
    increment_step

    # ---->>>> Verificação do sistema
    case $OS_NAME in
        ubuntu)
            case $VERSION in
                24.*|22.*|20.*|18.*)
                    show_progress "Sistema Ubuntu suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
                    ;;
            esac
            ;;
        debian)
            case $VERSION in
                12*|11*|10*|9*)
                    show_progress "Sistema Debian suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
                    ;;
            esac
            ;;
        almalinux|rocky)
            case $VERSION in
                9*|8*)
                    show_progress "Sistema $OS_NAME suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do $OS_NAME não suportada. Use 8 ou 9."
                    ;;
            esac
            ;;
        *)
            error_exit "Sistema não suportado. Use Ubuntu, Debian, AlmaLinux ou Rocky Linux."
            ;;
    esac
    increment_step

    # ---->>>> Instalação de pacotes requisitos e atualização do sistema
    show_progress "Atualizando o sistema..."
    case $OS_NAME in
        ubuntu|debian)
            apt-get upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
            apt-get install gnupg curl build-essential git cmake sysstat net-tools sqlite3 libsqlite3-dev -y > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes"
            ;;
        almalinux|rocky)
            dnf update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
            dnf install epel-release gnupg2 curl gcc g++ make git cmake sysstat net-tools sqlite sqlite-devel -y > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes"
            ;;
    esac
    increment_step

    # ---->>>> Criando o diretorio do script
    show_progress "Criando diretorio /opt/rustymanager..."
    mkdir /opt/ > /dev/null 2>&1
    mkdir /opt/rustymanager > /dev/null 2>&1
    increment_step

    # ---->>>> Criando as colunas no banco de dados
    show_progress "Configurando o banco de dados..."
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
        id INTEGER PRIMARY KEY
    );
    "
    for column in "proxy_ports" "stunnel_ports" "badvpn_ports" "checkuser_ports" "openvpn_port"; do
        column_exists=$(sqlite3 /opt/rustymanager/db "PRAGMA table_info(connections);" | grep -w "$column" | wc -l)
        if [ "$column_exists" -eq 0 ]; then
            sqlite3 /opt/rustymanager/db "ALTER TABLE connections ADD COLUMN $column TEXT;"
        fi
    done
    if [ $? -ne 0 ]; then
        error_exit "Falha ao configurar o banco de dados"
    fi
    increment_step

    # ---->>>> Instalar rust
    show_progress "Instalando Rust..."
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1 || error_exit "Falha ao instalar Rust"
        . "$HOME/.cargo/env"
    fi
    increment_step

    # ---->>>> Instalar o RustyManager
    show_progress "Compilando RustyManager, isso pode levar bastante tempo dependendo da maquina..."
    mkdir -p /opt/rustymanager
    mkdir -p /opt/rustymanager/ssl
    git clone --branch "$SCRIPT_VERSION" --recurse-submodules --single-branch https://github.com/UlekBR/RustyManager.git /root/RustyManager > /dev/null 2>&1 || error_exit "Falha ao clonar RustyManager"

    cd /root/RustyManager/
    cargo build --release --jobs $(nproc) > /dev/null 2>&1 || error_exit "Falha ao compilar RustyManager"
    mv -f ./target/release/SshScript /opt/rustymanager/manager
    mv -f ./target/release/CheckUser /opt/rustymanager/checkuser
    mv -f ./target/release/RustyProxy /opt/rustymanager/rustyproxy
    mv -f ./target/release/ConnectionsManager /opt/rustymanager/connectionsmanager
    increment_step

    # ---->>>> Compilar BadVPN
    show_progress "Compilando BadVPN..."
    mkdir -p /root/RustyManager/BadVpn/badvpn/badvpn-build
    cd /root/RustyManager/BadVpn/badvpn/badvpn-build
    cmake .. -DBUILD_NOTHING_BY_DEFAULT=1 -DBUILD_UDPGW=1 > /dev/null 2>&1 || error_exit "Falha ao configurar cmake para BadVPN"
    make > /dev/null 2>&1 || error_exit "Falha ao compilar BadVPN"
    mv -f udpgw/badvpn-udpgw /opt/rustymanager/badvpn
    increment_step

    # ---->>>> Configuração de permissões
    show_progress "Configurando permissões..."
    chmod +x /opt/rustymanager/{manager,rustyproxy,connectionsmanager,checkuser,badvpn}
    if [[ "$OS_NAME" == "almalinux" || "$OS_NAME" == "rockylinux" ]]; then
        sudo chcon -t bin_t /opt/rustymanager/{manager,rustyproxy,connectionsmanager,checkuser,badvpn}
    fi
    ln -sf /opt/rustymanager/manager /usr/local/bin/menu
    increment_step


    # ---->>>> Instalar speedtest
    show_progress "Instalando Speedtest..."

    case $OS_NAME in
        ubuntu|debian)
            curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | bash > /dev/null 2>&1 || error_exit "Falha ao baixar e instalar o script do speedtest"
            apt-get install speedtest -y > /dev/null 2>&1 || error_exit "Falha ao instalar o speedtest"
            ;;
        almalinux|rocky)
            curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.rpm.sh | bash > /dev/null 2>&1 || error_exit "Falha ao baixar e instalar o script do speedtest"
            dnf install speedtest -y > /dev/null 2>&1 || error_exit "Falha ao instalar o speedtest"
            ;;
    esac
    increment_step
    
    # ---->>>> Instalar Htop
    show_progress "Instalando monitor de recursos..."
    case $OS_NAME in
        ubuntu|debian)
            apt-get install htop -y > /dev/null 2>&1 || error_exit "Falha ao instalar o htop"
            ;;
        almalinux|rocky)
            dnf install htop -y > /dev/null 2>&1 || error_exit "Falha ao instalar o htop"
            ;;
    esac
    increment_step

    # ---->>>> Instalando STunnel
    show_progress "Instalando STunnel..."
    case $OS_NAME in
        ubuntu|debian)
            apt-get install stunnel4 -y > /dev/null 2>&1 || error_exit "Falha ao instalar o stunnel"
            ;;
        almalinux|rocky)
            dnf install stunnel -y > /dev/null 2>&1 || error_exit "Falha ao instalar o stunnel"
            ;;
    esac
    curl -sf -o /etc/stunnel/cert.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/cert.pem || error_exit "Falha ao baixar cert.pem"
    curl -sf -o /etc/stunnel/key.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/key.pem || error_exit "Falha ao baixar key.pem"
    curl -sf -o /etc/stunnel/stunnel.conf https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/conf || error_exit "Falha ao baixar config"
    systemctl stop stunnel4 > /dev/null 2>&1
    systemctl disable stunnel4 > /dev/null 2>&1
    increment_step

    # ---->>>> Instalando OpenVPN
    show_progress "Instalando OpenVPN..."
    case $OS_NAME in
        ubuntu|debian)
            apt-get install -y openvpn iptables openssl ca-certificates zip tar -y > /dev/null 2>&1 || error_exit "Falha ao instalar o openvpn"
            ;;
        almalinux|rocky)
            dnf install -y openvpn iptables openssl ca-certificates zip tar > /dev/null 2>&1 || error_exit "Falha ao instalar o openvpn"
            ;;
    esac
    if [ ! -d "/etc/openvpn/easy-rsa" ]; then
        curl -L -o /root/EasyRSA-3.2.1.tgz "https://github.com/OpenVPN/easy-rsa/releases/download/v3.2.1/EasyRSA-3.2.1.tgz" > /dev/null 2>&1 || error_exit "Falha ao baixar EasyRSA"
        tar xzf /root/EasyRSA-3.2.1.tgz -C /root/ > /dev/null 2>&1 || error_exit "Falha ao extrair o EasyRSA"
        mv -f /root/EasyRSA-3.2.1/ /etc/openvpn/easy-rsa/ > /dev/null 2>&1 || error_exit "Falha ao mover o EasyRSA"
        chown -R root:root /etc/openvpn/easy-rsa/ > /dev/null 2>&1 || error_exit "Falha ao configurar permissões do EasyRSA"
        rm -f /root/EasyRSA-3.2.1.tgz > /dev/null 2>&1 || error_exit "Falha ao remover o arquivo do EasyRSA"
        cd /etc/openvpn/easy-rsa/ > /dev/null 2>&1 || error_exit "Falha ao acessar pasta do EasyRSA"
        ./easyrsa --batch init-pki > /dev/null 2>&1 || error_exit "Falha ao iniciar pki"
        ./easyrsa --batch build-ca nopass > /dev/null 2>&1 || error_exit "Falha ao gerar certificado"
        ./easyrsa --batch gen-dh > /dev/null 2>&1 || error_exit "Falha ao gerar dh"
        ./easyrsa --batch build-server-full server nopass > /dev/null 2>&1 || error_exit "Falha ao gerar certificado do servidor"
        ./easyrsa --batch build-client-full client nopass > /dev/null 2>&1 || error_exit "Falha ao gerar certificado do cliente"
        cp pki/ca.crt pki/private/ca.key pki/dh.pem pki/issued/server.crt pki/private/server.key /etc/openvpn  > /dev/null 2>&1 || error_exit "Falha ao copiar arquivos do openvpn"
        openvpn --genkey --secret /etc/openvpn/ta.key  > /dev/null 2>&1 || error_exit "Falha ao gerar a chave"
    fi

    if [ ! -f "/etc/openvpn/server.conf" ]; then
        echo "port XXXX
    proto tcp
    dev tun
    sndbuf 0
    rcvbuf 0
    ca /etc/openvpn/ca.crt
    cert /etc/openvpn/server.crt
    key /etc/openvpn/server.key
    dh /etc/openvpn/dh.pem
    tls-auth /etc/openvpn/ta.key 0
    topology subnet
    server 10.8.0.0 255.255.255.0
    ifconfig-pool-persist ipp.txt
    verb 3
    push \"redirect-gateway def1 bypass-dhcp\"
    push \"dhcp-option DNS 8.8.8.8\"
    push \"dhcp-option DNS 8.8.4.4\"
    keepalive 10 120
    float
    cipher AES-256-CBC
    comp-lzo yes
    user nobody
    group nogroup
    persist-key
    persist-tun
    status openvpn-status.log
    management localhost 7505
    client-to-client
    client-cert-not-required
    username-as-common-name
    plugin \$(find /usr -type f -name 'openvpn-plugin-auth-pam.so') login
    duplicate-cn" > /etc/openvpn/server.conf  > /dev/null 2>&1 || error_exit "Falha ao criar openvpn server.conf"

        # Iniciar e habilitar o serviço OpenVPN
        echo 1 > /proc/sys/net/ipv4/ip_forward > /dev/null 2>&1 || error_exit "Falha ao habilitar ip forwarding"
        sed -i '/net.ipv4.ip_forward/s/^#//g' /etc/sysctl.conf > /dev/null 2>&1 || error_exit "Falha ao habilitar ip forwarding"
        sysctl -p > /dev/null 2>&1 || error_exit "Falha ao habilitar ip forwarding"
        iptables -t nat -F > /dev/null 2>&1 || error_exit "Falha ao limpar regras do iptables"
        iptables -F > /dev/null 2>&1 || error_exit "Falha ao limpar regras do iptables"
        iptables -P INPUT ACCEPT > /dev/null 2>&1 || error_exit "Falha ao adicionar regras do iptables"
        iptables -P FORWARD ACCEPT > /dev/null 2>&1 || error_exit "Falha ao adicionar regras do iptables"
        iptables -P OUTPUT ACCEPT > /dev/null 2>&1 || error_exit "Falha ao adicionar regras do iptables"
        iptables -t nat -A POSTROUTING -o $(ip route | grep default | awk '{print $5}') -j MASQUERADE > /dev/null 2>&1 || error_exit "Falha ao adicionar regra para permitir o trafego do openvpn no iptables"

    fi

    systemctl start openvpn@server > /dev/null 2>&1
    systemctl enable openvpn@server > /dev/null 2>&1

    increment_step


    # ---->>>> Limpeza
    show_progress "Limpando diretórios temporários..."
    cd /root/
    rm -rf /root/RustyManager/
    increment_step

    # ---->>>> Instalação finalizada :)
    echo "Instalação concluída com sucesso. digite 'menu' para acessar o menu."
fi
