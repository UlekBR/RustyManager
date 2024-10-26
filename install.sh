#!/bin/bash
# RustyManager Installer

TOTAL_STEPS=14
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
    apt-get update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar os repositorios"
    SCRIPT_VERSION="beta"
    increment_step

    # ---->>>> Verificação do sistema
    show_progress "Verificando o sistema..."
    if ! command -v lsb_release &> /dev/null; then
        apt-get install lsb-release -y > /dev/null 2>&1 || error_exit "Falha ao instalar lsb-release"
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
    apt-get upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema"
    apt-get install gnupg curl build-essential git cmake sysstat net-tools sqlite3 libsqlite3-dev -y > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes"
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
        id INTEGER PRIMARY KEY,
        proxy_ports TEXT,
        stunnel_ports TEXT,
        badvpn_ports TEXT,
        checkuser_ports TEXT
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
    mkdir -p /opt/rustymanager/ssl
    git clone --branch "$SCRIPT_VERSION" --recurse-submodules --single-branch https://github.com/UlekBR/RustyManager.git /root/RustyManager > /dev/null 2>&1 || error_exit "Falha ao clonar RustyManager"

    cd /root/RustyManager/
    cargo build --release --jobs $(nproc) > /dev/null 2>&1 || error_exit "Falha ao compilar RustyManager"
    mv ./target/release/SshScript /opt/rustymanager/manager
    mv ./target/release/CheckUser /opt/rustymanager/checkuser
    mv ./target/release/RustyProxy /opt/rustymanager/rustyproxy
    mv ./target/release/ConnectionsManager /opt/rustymanager/connectionsmanager
    increment_step

    # ---->>>> Baixando arquivos para o ssl
    show_progress "Baixando arquivos para ssl..."
    apt-get install -y stunnel4 > /dev/null 2>&1 || error_exit "Falha ao instalar STunnel"
    wget -O /opt/rustymanager/ssl/cert.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/main/Utils/ssl/cert.pem > /dev/null 2>&1 || error_exit "Falha ao baixar cert.pem"
    wget -O /opt/rustymanager/ssl/key.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/main/Utils/ssl/key.pem > /dev/null 2>&1 || error_exit "Falha ao baixar key.pem"
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
    chmod +x /opt/rustymanager/{manager,proxy,connectionsmanager,checkuser,badvpn}
    ln -sf /opt/rustymanager/manager /usr/local/bin/menu
    increment_step


    # ---->>>> Instalar speedtest
    show_progress "Instalando Speedtest..."
    curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | bash > /dev/null 2>&1 || error_exit "Falha ao baixar e instalar o script do speedtest"
    apt-get install -y speedtest > /dev/null 2>&1 || error_exit "Falha ao instalar o speedtest"
    increment_step
    
    # ---->>>> Instalar Htop
    show_progress "Instalando monitor de recursos..."
    apt-get install -y htop > /dev/null 2>&1 || error_exit "Falha ao instalar o speedtest"
    increment_step


    # ---->>>> Substituindo arquivo sshdconfig
    show_progress "Otimizando ssh..."
    wget -O /etc/ssh/sshd_config https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/sshd/config > /dev/null 2>&1 || error_exit "Falha ao baixar sshd_config"
    systemctl restart ssh > /dev/null 2>&1
    systemctl restart sshd > /dev/null 2>&1
    increment_step

    # ---->>>> Limpeza
    show_progress "Limpando diretórios temporários..."
    cd /root/
    rm -rf /root/RustyManager/
    increment_step

    # ---->>>> Instalação finalizada :)
    echo "Instalação concluída com sucesso. digite 'menu' para acessar o menu."
fi
