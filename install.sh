#!/bin/bash
# RustyManager Installer

TOTAL_STEPS=12
CURRENT_STEP=0

show_progress() {
    PERCENT=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo "Progresso: [${PERCENT}%] - $1"
}

error_exit() {
    echo -e "\nErro: $1"
    exit 1
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
    apt update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar os repositorios"
    SCRIPT_VERSION="main"
    increment_step

    # ---->>>> Verificação do sistema
    show_progress "Verificando o sistema..."
    if ! command -v lsb_release &> /dev/null; then
        apt install lsb-release -y > /dev/null 2>&1 || error_exit "Falha ao instalar lsb-release"
    fi
    increment_step

    # ---->>>> Verificação do sistema
    OS_NAME=$(lsb_release -is)
    VERSION=$(lsb_release -rs)

    case $OS_NAME in
        Ubuntu)
            case $VERSION in
                24.*|22.*|20.*|18.*)
                    show_progress "Sistema Ubuntu suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
                    ;;
            esac
            ;;
        Debian)
            case $VERSION in
                12*|11*|10*|9*)
                    show_progress "Sistema Debian suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
                    ;;
            esac
            ;;
        *)
            error_exit "Sistema não suportado. Use Ubuntu ou Debian."
            ;;
    esac
    increment_step

    # ---->>>> Instalação de pacotes requisitos e atualização do sistema
    show_progress "Atualizando o sistema..."
    apt upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
    apt-get install gnupg curl build-essential git cmake sqlite3 -y > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes"
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
        id INTEGER PRIMARY KEY,
        http_proxy_enabled BOOLEAN,
        http_proxy_port INTEGER,
        stunnel_enabled BOOLEAN,
        stunnel_port INTEGER,
        badvpn_ports TEXT
    );
    " || error_exit "Falha ao configurar o banco de dados"
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
    git clone --branch "$SCRIPT_VERSION" --recurse-submodules --single-branch https://github.com/UlekBR/RustyManager.git /root/RustyManager > /dev/null 2>&1 || error_exit "Falha ao clonar RustyManager"

    cd /root/RustyManager/
    cargo build --release --jobs $(nproc) > /dev/null 2>&1 || error_exit "Falha ao compilar RustyManager"
    mv ./target/release/SshScript /opt/rustymanager/manager
    mv ./target/release/HttpProxy /opt/rustymanager/proxy
    mv ./target/release/BadVpnManager /opt/rustymanager/badmanager
    increment_step

    # ---->>>> Compilar BadVPN
    show_progress "Compilando BadVPN..."
    mkdir -p /root/RustyManager/BadVpn/badvpn/badvpn-build
    cd /root/RustyManager/BadVpn/badvpn/badvpn-build
    cmake .. -DBUILD_NOTHING_BY_DEFAULT=1 -DBUILD_UDPGW=1 > /dev/null 2>&1 || error_exit "Falha ao configurar cmake para BadVPN"
    make > /dev/null 2>&1 || error_exit "Falha ao compilar BadVPN"
    mv udpgw/badvpn-udpgw /opt/rustymanager/badvpn
    increment_step

    # ---->>>> Configuração de permissões
    show_progress "Configurando permissões..."
    chmod +x /opt/rustymanager/{manager,proxy,badmanager,badvpn}
    ln -sf /opt/rustymanager/manager /usr/local/bin/menu
    increment_step

    # ---->>>> Criar o serviço do proxy
    show_progress "Criando o serviço do proxy..."
    SERVICE_FILE="/etc/systemd/system/proxy.service"
    echo "[Unit]
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
    WantedBy=multi-user.target" > "$SERVICE_FILE" || error_exit "Falha ao criar o serviço do proxy"
    systemctl daemon-reload > /dev/null 2>&1 || error_exit "Falha ao recarregar serviços"
    increment_step

    # ---->>>> Instalando STunnel
    show_progress "Instalando STunnel..."
    apt install -y stunnel4 > /dev/null 2>&1 || error_exit "Falha ao instalar STunnel"
    wget -O /etc/stunnel/cert.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/cert.pem > /dev/null 2>&1 || error_exit "Falha ao baixar cert.pem"
    wget -O /etc/stunnel/key.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/key.pem > /dev/null 2>&1 || error_exit "Falha ao baixar key.pem"
    sed -i 's/ENABLED=0/ENABLED=1/g' /etc/default/stunnel4 || error_exit "Falha ao configurar STunnel"
    systemctl stop stunnel4
    systemctl disable stunnel4
    increment_step

    # ---->>>> Limpeza
    show_progress "Limpando diretórios temporários..."
    rm -rf /root/RustyManager/
    increment_step

    # ---->>>> Instalação finalizada :)
    clear
    echo "Instalação concluída com sucesso. Digite 'menu' para acessar o menu."
fi
