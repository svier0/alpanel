#!/bin/sh
set -e

VERSION="0.1.0"

# 安装脚本
DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/master/scripts/install.sh"
# 控制脚本
ALP_DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/master/scripts/alp.sh"

# 变量申明
ARCH=$(uname -m)
REPO_URL="/etc/apk/repositories"
GH_PROXY=""
PKG_ARCH=""
setup_path=/www
PANEL_PORT=8888
PANEL_USER="admin"
PANEL_PASS=""
ARG_PANEL_PORT=""
ARG_PANEL_USER=""
ARG_PANEL_PASS=""
LAN_IPV4="0.0.0.0"
PUBLIC_IPV4="0.0.0.0"

# 解析安装参数
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --port)
                shift
                case "$1" in
                    ''|*[!0-9]*)
                        echo "错误: --port 需要 1-65535 之间的数字"
                        exit 1
                        ;;
                esac
                if [ "$1" -lt 1 ] || [ "$1" -gt 65535 ]; then
                    echo "错误: --port 需要 1-65535 之间的数字"
                    exit 1
                fi
                ARG_PANEL_PORT="$1"
                shift
                ;;
            --user)
                shift
                ARG_PANEL_USER="$1"
                shift
                ;;
            --pass)
                shift
                ARG_PANEL_PASS="$1"
                shift
                ;;
            *)
                echo "错误: 未知参数 $1"
                echo "用法: sh install.sh [--port 端口号]"
                exit 1
                ;;
        esac
    done
}

# 判断系统版本
check_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
    fi
    if [ "$ID" != "alpine" ]; then
        echo "Alpanel 仅支持 Alpine Linux"
        exit 1
    fi
}

# 判断架构
check_arch() {
    case "$ARCH" in
        x86_64|amd64)  PKG_ARCH="x86_64-unknown-linux-musl" ;;
        aarch64|arm64) PKG_ARCH="aarch64-unknown-linux-musl" ;;
        *)
            echo "暂不支持 $ARCH 架构，请自行编译 https://github.com/svier0/alpanel"
            exit 1
            ;;
    esac
}

# 判断网络环境
check_ghproxy() {
    if ! wget --spider --timeout=1 --tries=1 -q https://www.google.com > /dev/null 2>&1; then
        GH_PROXY="https://gh-proxy.com/"
    fi
}

# 判断安装权限
check_root() {
    if [ "$(whoami)" != "root" ]; then
        echo "检查到当前非 root 权限进行面板安装"
        echo "请使用以下命令重新执行："
        echo "sudo curl -fsSL -o install.sh ${GH_PROXY}$DOWNLOAD_URL && sudo sh install.sh"
        exit 1
    fi
}

# 辅助函数：下载文件
download_file() {
    url="$1" dest="$2"
    echo "下载: ${GH_PROXY}${url}"
    if curl -fsSL --connect-timeout 10 -o "$dest" "${GH_PROXY}${url}" >/dev/null 2>&1; then
        echo "下载成功"
        return 0
    fi
    echo "错误: 下载失败: $url"
    echo "      请检查网络或服务器配置，可手动下载到 $dest"
    return 1
}

# 安装依赖
install_dependency() {
    if [ -n "$GH_PROXY" ]; then
        echo "https://mirrors.aliyun.com/alpine/v3.21/main" > /etc/apk/repositories
        echo "https://mirrors.aliyun.com/alpine/v3.21/community" >> /etc/apk/repositories
    fi
    apk update
    apk add curl sqlite openrc jq vnstat
    rc-update add vnstatd default
    rc-service vnstatd start 2>/dev/null || true
}

# 创建目录
mkpaneldir() {
    mkdir -p ${setup_path}
    mkdir -p ${setup_path}/wwwlogs
    mkdir -p ${setup_path}/wwwroot
    mkdir -p ${setup_path}/server
    mkdir -p ${setup_path}/server/cron
    mkdir -p ${setup_path}/server/data
    mkdir -p ${setup_path}/server/stop
    mkdir -p ${setup_path}/server/panel
    mkdir -p ${setup_path}/server/panel/vhost
    mkdir -p ${setup_path}/server/panel/vhost/nginx
    mkdir -p ${setup_path}/server/panel/vhost/rewrite
    mkdir -p ${setup_path}/server/panel/vhost/ssl
    mkdir -p ${setup_path}/server/panel/vhost/template
    mkdir -p ${setup_path}/server/panel/vhost/template/nginx
    mkdir -p ${setup_path}/server/panel/data
    mkdir -p ${setup_path}/server/panel/data/db
    mkdir -p ${setup_path}/server/panel/data/files_ps
    mkdir -p ${setup_path}/server/panel/plugin
}

# 添加www用户
add_user() {
    addgroup -S www 2>/dev/null || true
    adduser -D -H -S -G www -s /sbin/nologin www 2>/dev/null || true
    chown -R www:www ${setup_path}/wwwroot ${setup_path}/wwwlogs 2>/dev/null || true
}

# 下载面板
dl_panel() {
    PANEL_DOWNLOAD_URL="https://github.com/svier0/alpanel/releases/download/${VERSION}/alpanel-${VERSION}-${PKG_ARCH}.tar.gz"
    download_file "$PANEL_DOWNLOAD_URL" /tmp/alpanel.tar.gz || exit 1
}

# 解压面板
untar_panel() {
    if [ ! -s /tmp/alpanel.tar.gz ]; then
        echo "错误: 安装包下载失败或为空"
        exit 1
    fi
    if ! gzip -t /tmp/alpanel.tar.gz >/dev/null 2>&1; then
        echo "错误: 安装包校验失败，文件不完整或已损坏"
        exit 1
    fi
    if ! tar -xzf /tmp/alpanel.tar.gz -C ${setup_path}/server/panel/; then
        echo "错误: 安装包解压失败"
        exit 1
    fi
    if [ ! -f ${setup_path}/server/panel/alpanel ]; then
        echo "错误: 解压后未找到面板程序 ${setup_path}/server/panel/alpanel"
        exit 1
    fi
    chmod +x ${setup_path}/server/panel/alpanel
    if [ ! -x ${setup_path}/server/panel/alpanel ]; then
        echo "错误: 面板程序 ${setup_path}/server/panel/alpanel 不可执行"
        exit 1
    fi
    rm -f /tmp/alpanel.tar.gz
}

# 下载控制脚本
dl_ctl_script() {
    download_file "$ALP_DOWNLOAD_URL" /usr/bin/alp || exit 1
    if [ ! -s /usr/bin/alp ]; then
        echo "错误: alp 脚本下载失败或为空"
        exit 1
    fi
    chmod +x /usr/bin/alp
    if [ ! -x /usr/bin/alp ]; then
        echo "错误: alp 脚本不可执行"
        exit 1
    fi
}

# 生成初始配置
init_env() {
    ENV_FILE="${setup_path}/server/panel/.env"

    if [ -n "$ARG_PANEL_PORT" ]; then
        PANEL_PORT="$ARG_PANEL_PORT"
    else
        PANEL_PORT=$(shuf -i 10000-65535 -n 1)
    fi
    if [ -n "$ARG_PANEL_USER" ]; then
        PANEL_USER="$ARG_PANEL_USER"
    fi
    if [ -n "$ARG_PANEL_PASS" ]; then
        PANEL_PASS="$ARG_PANEL_PASS"
    else
        PANEL_PASS=$(tr -dc A-Za-z0-9 < /dev/urandom | head -c 16)
    fi

    cat > $ENV_FILE << EOF
PANEL_PORT=$PANEL_PORT
PANEL_USER=$PANEL_USER
PANEL_PASSWORD=$PANEL_PASS
PANEL_TITLE=Alpanel
PANEL_THEME=auto
GHPROXY=${GH_PROXY:-false}
EOF
}

# 配置面板开机自启
set_autostart() {
    cat > /etc/init.d/alpanel << 'EOF'
#!/sbin/openrc-run

name="Alpanel"
description="Alpanel server management panel"

start() {
    ebegin "Starting ${name}"
    /usr/bin/alp 11
    eend $?
}

stop() {
    ebegin "Stopping ${name}"
    /usr/bin/alp 12
    eend $?
}
EOF
    chmod +x /etc/init.d/alpanel
    rc-update add alpanel default 2>/dev/null || true
}

# 启动面板
serve_start() {
    echo "正在启动面板服务..."
    if ! alp 11; then
        echo "错误: 面板启动命令执行失败"
        exit 1
    fi
    i=0
    while [ $i -lt 20 ]; do
        if curl -sf -o /dev/null "http://127.0.0.1:$PANEL_PORT/" >/dev/null 2>&1; then
            echo "面板启动成功"
            break
        fi
        i=$((i + 1))
        sleep 1
    done
    if [ $i -ge 20 ]; then
        echo "错误: 面板启动失败，请手动查看日志"
        exit 1
    fi
}

# 回显
echoinfo() {
    echo "================================"
    echo " Alpanel 安装完成"
    echo "================================"
    LAN_IPV4=$(ip -4 addr show scope global | grep 'inet ' | head -1 | awk '{print $2}' | cut -d/ -f1)
    if [ -n "$GH_PROXY" ]; then
        SRV=$(nslookup resolver1.opendns.com 2>/dev/null | grep -oE '([0-9]+\.){3}[0-9]+' | tail -1)
        PUBLIC_IPV4=$(nslookup myip.opendns.com $SRV 2>/dev/null | grep -oE '([0-9]+\.){3}[0-9]+' | tail -1)
    else
        PUBLIC_IPV4=$(curl --connect-timeout 10 https://ifconfig.me)
    fi
    if [ -n "$PUBLIC_IPV4" ]; then
        echo "公网面板地址: http://$PUBLIC_IPV4:$PANEL_PORT"
    fi
    echo "局域网面板地址: http://$LAN_IPV4:$PANEL_PORT"
    echo "账号: $PANEL_USER"
    echo "密码: $PANEL_PASS"
    echo "================================"
    echo "面板配置目录: ${setup_path}/server/panel/"
    echo "使用 alp 命令管理面板"
    echo "================================"
}

# 入口
main() {
    # 解析安装参数
    parse_args "$@"
    # 判断系统版本
    check_os
    # 判断架构
    check_arch
    # 判断网络环境
    check_ghproxy
    # 判断安装权限
    check_root
    # 安装依赖
    install_dependency
    # 创建目录
    mkpaneldir
    # 添加www用户
    add_user
    # 下载面板
    dl_panel
    # 解压面板
    untar_panel
    # 下载控制脚本
    dl_ctl_script
    # 生成初始配置
    init_env
    # 配置面板开机自启
    set_autostart
    # 启动面板
    serve_start
    # 回显
    echoinfo
}
main "$@"