#!/bin/sh
set -e

DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/master/scripts/install.sh"
VERSION="0.1.0"
ALP_DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/master/scripts/alp.sh"

PANEL_PORT_ARG=""
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
            PANEL_PORT_ARG="$1"
            shift
            ;;
        *)
            echo "错误: 未知参数 $1"
            echo "用法: sh install.sh [--port 端口号]"
            exit 1
            ;;
    esac
done

if wget --spider --timeout=1 --tries=1 -q https://www.google.com > /dev/null 2>&1; then
    REPO_URL="/etc/apk/repositories"
    GH_PROXY=""
else
    echo "https://mirrors.aliyun.com/alpine/v3.21/main" > /etc/apk/repositories
    echo "https://mirrors.aliyun.com/alpine/v3.21/community" >> /etc/apk/repositories
    GH_PROXY="https://gh-proxy.com/"
fi
if ! command -v curl >/dev/null 2>&1; then
    echo "未检测到 curl，先安装..."
    apk add curl 2>/dev/null || { apk update && apk add curl; }
fi
if [ "$(whoami)" != "root" ]; then
    echo "检查到当前非 root 权限进行面板安装"
    echo "请使用以下命令重新执行："
    echo "sudo curl -fsSL -o install.sh ${GH_PROXY}$DOWNLOAD_URL && sudo sh install.sh"
    exit 1
fi

if [ -f /etc/os-release ]; then
    . /etc/os-release
fi
if [ "$ID" != "alpine" ]; then
    echo "Alpanel 仅支持 Alpine Linux"
    exit 1
fi

download_file() {
    url="$1" dest="$2"
    for proxy in "https://gh-proxy.com/" "https://ghfast.top/" ""; do
        echo "下载: ${proxy}${url}"
        if curl -fsSL --connect-timeout 10 -o "$dest" "${proxy}${url}" >/dev/null 2>&1; then
            echo "下载成功"
            return 0
        fi
        echo "下载失败，尝试下一个镜像..."
    done
    echo "错误: 下载失败: $url"
    echo "      请检查网络或服务器配置，可手动下载到 $dest"
    return 1
}

apk update
apk add sqlite jq vnstat openrc curl
rc-update add vnstatd default
rc-service vnstatd start 2>/dev/null || true

ARCH=$(uname -m)
case "$ARCH" in
    x86_64|amd64)  PKG_ARCH="x86_64-unknown-linux-musl" ;;
    aarch64|arm64) PKG_ARCH="aarch64-unknown-linux-musl" ;;
    *)
        echo "暂不支持 $ARCH 架构，请自行编译 https://github.com/svier0/alpanel"
        exit 1
        ;;
esac

setup_path=/www
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

addgroup -S www 2>/dev/null || true
adduser -D -H -S -G www -s /sbin/nologin www 2>/dev/null || true
chown -R www:www ${setup_path}/wwwroot ${setup_path}/wwwlogs 2>/dev/null || true

PANEL_DOWNLOAD_URL="https://github.com/svier0/alpanel/releases/download/${VERSION}/alpanel-${VERSION}-${PKG_ARCH}.tar.gz"
download_file "$PANEL_DOWNLOAD_URL" /tmp/alpanel.tar.gz || exit 1
if [ ! -s /tmp/alpanel.tar.gz ]; then
    echo "错误: 安装包下载失败或为空"
    exit 1
fi
if ! gzip -t /tmp/alpanel.tar.gz >/dev/null 2>&1; then
    echo "错误: 安装包校验失败，文件不完整或已损坏"
    exit 1
fi
if ! tar -xzf /tmp/alpanel.tar.gz -C /www/server/panel/; then
    echo "错误: 安装包解压失败"
    exit 1
fi
if [ ! -f /www/server/panel/alpanel ]; then
    echo "错误: 解压后未找到面板程序 /www/server/panel/alpanel"
    exit 1
fi
chmod +x /www/server/panel/alpanel
if [ ! -x /www/server/panel/alpanel ]; then
    echo "错误: 面板程序 /www/server/panel/alpanel 不可执行"
    exit 1
fi
rm -f /tmp/alpanel.tar.gz

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

ENV_FILE="/www/server/panel/.env"
if [ -n "$PANEL_PORT_ARG" ]; then
    PANEL_PORT="$PANEL_PORT_ARG"
else
    PANEL_PORT=$(shuf -i 10000-65535 -n 1)
fi
PANEL_USER="admin"
PANEL_PASSWORD=$(tr -dc A-Za-z0-9 < /dev/urandom | head -c 16)

cat > $ENV_FILE << EOF
PANEL_PORT=$PANEL_PORT
PANEL_USER=$PANEL_USER
PANEL_PASSWORD=$PANEL_PASSWORD
PANEL_TITLE=Alpanel
PANEL_THEME=auto
GHPROXY=${GH_PROXY:-false}
EOF

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

echo "================================"
echo " Alpanel 安装完成"
echo "================================"
LAN_IPV4=$(ip -4 addr show scope global | grep 'inet ' | head -1 | awk '{print $2}' | cut -d/ -f1)
SRV=$(nslookup resolver1.opendns.com 2>/dev/null | grep -oE '([0-9]+\.){3}[0-9]+' | tail -1)
PUBLIC_IPV4=$(nslookup myip.opendns.com $SRV 2>/dev/null | grep -oE '([0-9]+\.){3}[0-9]+' | tail -1)
if [ -n "$PUBLIC_IPV4" ]; then
    echo "公网面板地址: http://$PUBLIC_IPV4:$PANEL_PORT"
fi
echo "局域网面板地址: http://$LAN_IPV4:$PANEL_PORT"
echo "账号: $PANEL_USER"
echo "密码: $PANEL_PASSWORD"
echo "================================"
echo "面板配置目录: /www/server/panel/"
echo "使用 alp 命令管理面板"
echo "================================"
