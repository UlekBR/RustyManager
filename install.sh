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
    mv -f ./target/release/SshScript /opt/rustymanager/manager
    mv -f ./target/release/CheckUser /opt/rustymanager/checkuser
    mv -f ./target/release/RustyProxy /opt/rustymanager/rustyproxy
    mv -f ./target/release/ConnectionsManager /opt/rustymanager/connectionsmanager
    increment_step

    # ---->>>> Baixando arquivos para o ssl
    show_progress "Baixando arquivos para ssl..."
    wget -O /opt/rustymanager/ssl/cert.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/ssl/cert.pem > /dev/null 2>&1 || error_exit "Falha ao baixar cert.pem"
    wget -O /opt/rustymanager/ssl/key.pem https://raw.githubusercontent.com/UlekBR/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/ssl/key.pem > /dev/null 2>&1 || error_exit "Falha ao baixar key.pem"
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
    ln -sf /opt/rustymanager/manager /usr/local/bin/menu
    increment_step


    # ---->>>> Instalar speedtest
    show_progress "Instalando Speedtest..."
    curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | bash > /dev/null 2>&1 || error_exit "Falha ao baixar e instalar o script do speedtest"
    case $OS_NAME in
        ubuntu|debian)
            apt-get install speedtest -y > /dev/null 2>&1 || error_exit "Falha ao instalar o speedtest"
            ;;
        almalinux|rocky)
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

    # ---->>>> Limpeza
    show_progress "Limpando diretórios temporários..."
    cd /root/
    rm -rf /root/RustyManager/
    increment_step

    # ---->>>> Instalação finalizada :)
    echo "Instalação concluída com sucesso. digite 'menu' para acessar o menu."
fi
