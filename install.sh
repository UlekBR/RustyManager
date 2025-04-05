#!/bin/bash
# RustyManager Installer

TOTAL_STEPS=13
CURRENT_STEP=0
INSTALL_DIR=/opt/rustymanager
LOG_FILE="/dev/null"

show_progress() {
    PERCENT=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo "Progresso: [${PERCENT}%] - $1"
}

error_exit() {
    echo -e "\nErro: $1"
    exit
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
    SCRIPT_VERSION="main"
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
        24.* | 22.* | 20.* | 18.*)
            show_progress "Sistema Ubuntu suportado, continuando..."
            ;;
        *)
            error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
            ;;
        esac
        ;;
    debian)
        case $VERSION in
        12* | 11* | 10* | 9*)
            show_progress "Sistema Debian suportado, continuando..."
            ;;
        *)
            error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
            ;;
        esac
        ;;
    almalinux | rocky)
        case $VERSION in
        9* | 8*)
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
    ubuntu | debian)
        apt-get update -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao atualizar o sistema"
        apt-get install gnupg curl build-essential git cmake clang sysstat net-tools sqlite3 libsqlite3-dev zip tar iptables ca-certificates -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar pacotes"
        ;;
    almalinux | rocky)
        dnf update -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao atualizar o sistema"
        dnf install epel-release gnupg2 curl gcc g++ make git cmake clang sysstat net-tools sqlite sqlite-devel zip tar iptables ca-certificates -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar pacotes"
        ;;
    esac
    increment_step

    # ---->>>> Criando o diretorio do script
    show_progress "Criando diretorio $INSTALL_DIR..."
    # Deletando caso já exista
    if test -d "$INSTALL_DIR"; then
        rm -rf "$INSTALL_DIR" >"$LOG_FILE" 2>&1
    fi
    mkdir -p "$INSTALL_DIR" >"$LOG_FILE" 2>&1
    increment_step

    # ---->>>> Criando as colunas no banco de dados
    show_progress "Configurando o banco de dados..."
    sqlite3 "$INSTALL_DIR"/db "
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
    for column in "proxy_ports" "sslproxy_ports" "badvpn_ports" "checkuser_ports" "openvpn_port"; do
        column_exists=$(sqlite3 "$INSTALL_DIR"/db "PRAGMA table_info(connections);" | grep -w "$column" | wc -l)
        if [ "$column_exists" -eq 0 ]; then
            sqlite3 "$INSTALL_DIR"/db "ALTER TABLE connections ADD COLUMN $column TEXT;"
        fi
    done
    if [ $? -ne 0 ]; then
        error_exit "Falha ao configurar o banco de dados"
    fi
    increment_step

    # ---->>>> Instalar rust
    show_progress "Instalando Rust..."
    if ! command -v rustc &>"$LOG_FILE"; then
        if ! test -d "$HOME/.cargo"; then
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar Rust"
        fi
        . "$HOME/.cargo/env"
    fi
    increment_step

    # ---->>>> Instalar o RustyManager
    show_progress "Compilando RustyManager, isso pode levar bastante tempo dependendo da maquina..."
    mkdir -p "$INSTALL_DIR"/ssl
    REPO_DIR="${TMPDIR:-/tmp}/rustymanager"
    trap "rm -rf '$REPO_DIR'" EXIT

    SCRIPT_PATH=$(dirname "$(realpath -s "$0")")
    # Se o script não estiver rodando dentro do repositório clonado,
    # Um git clone é feito
    if ! test -d "$SCRIPT_PATH/.git"; then
        git clone \
            --branch "$SCRIPT_VERSION" \
            --recurse-submodules \
            --single-branch \
            --filter=tree:0 \
            https://github.com/UlekBR/RustyManager.git \
            "$REPO_DIR" >"$LOG_FILE" 2>&1 || error_exit "Falha ao clonar RustyManager"
    else
        cp -r "$SCRIPT_PATH" "$REPO_DIR"
    fi
    cd "$REPO_DIR" || error_exit "Falha ao abrir a pasta do RustyManager"
    cp -f ./Utils/ssl/cert.pem "$INSTALL_DIR"/ssl/cert.pem >"$LOG_FILE" 2>&1
    cp -f ./Utils/ssl/key.pem "$INSTALL_DIR"/ssl/key.pem >"$LOG_FILE" 2>&1

    ## definindo compilador
    export CC=clang
    cargo build --release --jobs "$(nproc)" >"$LOG_FILE" 2>&1 || error_exit "Falha ao compilar RustyManager"
    mv -f ./target/release/SshScript "$INSTALL_DIR"/manager >"$LOG_FILE" 2>&1
    mv -f ./target/release/CheckUser "$INSTALL_DIR"/checkuser >"$LOG_FILE" 2>&1
    mv -f ./target/release/RustyProxy "$INSTALL_DIR"/rustyproxy >"$LOG_FILE" 2>&1
    mv -f ./target/release/RustyProxySSL "$INSTALL_DIR"/rustyproxyssl >"$LOG_FILE" 2>&1
    mv -f ./target/release/ConnectionsManager "$INSTALL_DIR"/connectionsmanager >"$LOG_FILE" 2>&1

    increment_step

    # ---->>>> Compilar BadVPN
    show_progress "Compilando BadVPN..."
    mkdir -p "$REPO_DIR"/BadVpn/badvpn-build
    cd "$REPO_DIR"/BadVpn/badvpn-build
    cmake .. -DBUILD_NOTHING_BY_DEFAULT=1 -DBUILD_UDPGW=1 >"$LOG_FILE" 2>&1 || error_exit "Falha ao configurar cmake para BadVPN"
    make >"$LOG_FILE" 2>&1 || error_exit "Falha ao compilar BadVPN"
    cp -f udpgw/badvpn-udpgw "$INSTALL_DIR"/badvpn
    increment_step

    # ---->>>> Configuração de permissões
    show_progress "Configurando permissões..."
    chmod +x "$INSTALL_DIR"/{manager,rustyproxy,rustyproxyssl,connectionsmanager,checkuser,badvpn}
    if [[ "$OS_NAME" == "almalinux" || "$OS_NAME" == "rockylinux" ]]; then
        sudo chcon -t bin_t "$INSTALL_DIR"/{manager,rustyproxy,rustyproxyssl,connectionsmanager,checkuser,badvpn}
    fi
    ln -sf "$INSTALL_DIR"/manager /usr/local/bin/menu
    increment_step

    # ---->>>> Instalar speedtest
    show_progress "Instalando Speedtest..."
    case $OS_NAME in
    ubuntu | debian)
        curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | bash >"$LOG_FILE" 2>&1 || error_exit "Falha ao baixar e instalar o script do speedtest"
        apt-get install speedtest -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar o speedtest"
        ;;
    almalinux | rocky)
        curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.rpm.sh | bash >"$LOG_FILE" 2>&1 || error_exit "Falha ao baixar e instalar o script do speedtest"
        dnf install speedtest -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar o speedtest"
        ;;
    esac
    increment_step

    # ---->>>> Instalar Htop
    show_progress "Instalando monitor de recursos..."
    case $OS_NAME in
    ubuntu | debian)
        apt-get install htop -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar o htop"
        ;;
    almalinux | rocky)
        dnf install htop -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar o htop"
        ;;
    esac
    increment_step

    # ---->>>> Instalando OpenVPN
    show_progress "Instalando OpenVPN..."
    case $OS_NAME in
    ubuntu | debian)
        apt-get install -y openvpn openssl -y >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar o openvpn"
        ;;
    almalinux | rocky)
        dnf install -y openvpn openssl >"$LOG_FILE" 2>&1 || error_exit "Falha ao instalar o openvpn"
        ;;
    esac
    if [ ! -d "/etc/openvpn/easy-rsa" ]; then
        easytmp=$(mktemp -d)
        version="3.2.1"
        curl -L -o "$easytmp/EasyRSA.tgz" "https://github.com/OpenVPN/easy-rsa/releases/download/v$version/EasyRSA-$version.tgz" >"$LOG_FILE" 2>&1 || error_exit "Falha ao baixar EasyRSA"
        tar xzf "$easytmp/EasyRSA.tgz" -C "$easytmp" >"$LOG_FILE" 2>&1 || error_exit "Falha ao extrair o EasyRSA"
        mv -f "$easytmp/EasyRSA-$version/" /etc/openvpn/easy-rsa/ >"$LOG_FILE" 2>&1 || error_exit "Falha ao mover o EasyRSA"
        rm -rf "$easytmp" >"$LOG_FILE" 2>&1 || error_exit "Falha ao remover o arquivo do EasyRSA"
        chown -R root:root /etc/openvpn/easy-rsa/ >"$LOG_FILE" 2>&1 || error_exit "Falha ao configurar permissões do EasyRSA"
        cd /etc/openvpn/easy-rsa/ >"$LOG_FILE" 2>&1 || error_exit "Falha ao acessar pasta do EasyRSA"
        ./easyrsa --batch init-pki >"$LOG_FILE" 2>&1 || error_exit "Falha ao iniciar pki"
        ./easyrsa --batch build-ca nopass >"$LOG_FILE" 2>&1 || error_exit "Falha ao gerar certificado"
        ./easyrsa --batch gen-dh >"$LOG_FILE" 2>&1 || error_exit "Falha ao gerar dh"
        ./easyrsa --batch build-server-full server nopass >"$LOG_FILE" 2>&1 || error_exit "Falha ao gerar certificado do servidor"
        ./easyrsa --batch build-client-full client nopass >"$LOG_FILE" 2>&1 || error_exit "Falha ao gerar certificado do cliente"
        cp pki/ca.crt pki/private/ca.key pki/dh.pem pki/issued/server.crt pki/private/server.key /etc/openvpn >"$LOG_FILE" 2>&1 || error_exit "Falha ao copiar arquivos do openvpn"
        openvpn --genkey --secret /etc/openvpn/ta.key >"$LOG_FILE" 2>&1 || error_exit "Falha ao gerar a chave"
    fi

    plugin_path=$(find /usr -type f -name 'openvpn-plugin-auth-pam.so')
    echo "port none
proto none
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
client-to-client
client-cert-not-required
username-as-common-name
plugin $plugin_path login
duplicate-cn" >/etc/openvpn/server.conf || error_exit "Falha ao criar openvpn server.conf"

    echo 1 >/proc/sys/net/ipv4/ip_forward >"$LOG_FILE" 2>&1 || error_exit "Falha ao habilitar ip forwarding"
    sed -i '/net.ipv4.ip_forward/s/^#//g' /etc/sysctl.conf >"$LOG_FILE" 2>&1 || error_exit "Falha ao habilitar ip forwarding"
    sysctl -p >"$LOG_FILE" 2>&1 || error_exit "Falha ao habilitar ip forwarding"
    iptables -t nat -F >"$LOG_FILE" 2>&1 || error_exit "Falha ao limpar regras do iptables"
    iptables -F >"$LOG_FILE" 2>&1 || error_exit "Falha ao limpar regras do iptables"
    iptables -P INPUT ACCEPT >"$LOG_FILE" 2>&1 || error_exit "Falha ao adicionar regras do iptables"
    iptables -P FORWARD ACCEPT >"$LOG_FILE" 2>&1 || error_exit "Falha ao adicionar regras do iptables"
    iptables -P OUTPUT ACCEPT >"$LOG_FILE" 2>&1 || error_exit "Falha ao adicionar regras do iptables"
    iptables -t nat -A POSTROUTING -o $(ip route | grep default | awk '{print $5}') -j MASQUERADE >"$LOG_FILE" 2>&1 || error_exit "Falha ao adicionar regra para permitir o trafego do openvpn no iptables"

    systemctl stop openvpn@server >"$LOG_FILE" 2>&1
    systemctl disable openvpn@server >"$LOG_FILE" 2>&1
    increment_step

    # ---->>>> Limpeza
    show_progress "Limpando diretórios temporários..."
    increment_step

    # ---->>>> Instalação finalizada :)
    echo "Instalação concluída com sucesso. digite 'menu' para acessar o menu."
fi
