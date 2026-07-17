#!/bin/bash

DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/master/scripts/install.sh"
VERSION="0.1.0"
ALP_DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/master/scripts/alp.sh"

if wget --spider --timeout=1 --tries=1 -q https://www.google.com > /dev/null 2>&1; then
    REPO_URL="/etc/apk/repositories"
    GH_PROXY=""
else
    echo "https://mirrors.aliyun.com/alpine/v3.21/main" > /etc/apk/repositories
    echo "https://mirrors.aliyun.com/alpine/v3.21/community" >> /etc/apk/repositories
    GH_PROXY="https://gh-proxy.com/"
fi
apk update
apk add sqlite

if [ "$(whoami)" != "root" ]; then
    echo "检查到当前非 root 权限进行面板安装"
    echo "请使用以下命令重新执行："
    echo "sudo wget -O install.sh ${GH_PROXY}$DOWNLOAD_URL && sudo bash install.sh"
    exit 1
fi

if [ -f /etc/os-release ]; then
    . /etc/os-release
fi
if [ "$ID" != "alpine" ]; then
    echo "Alpanel 仅支持 Alpine Linux"
    exit 1
fi

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

addgroup -S www 2>/dev/null || true
adduser -D -H -S -G www -s /sbin/nologin www 2>/dev/null || true
chown -R www:www ${setup_path}/wwwroot ${setup_path}/wwwlogs 2>/dev/null || true

PANEL_DOWNLOAD_URL="https://github.com/svier0/alpanel/releases/latest/download/alpanel-${VERSION}-${PKG_ARCH}.tar.gz"
wget -O /tmp/alpanel.tar.gz ${GH_PROXY}$PANEL_DOWNLOAD_URL
tar -xzf /tmp/alpanel.tar.gz -C /www/server/panel/
chmod +x /www/server/panel/alpanel
rm -f /tmp/alpanel.tar.gz

wget -O /usr/bin/alp ${GH_PROXY}$ALP_DOWNLOAD_URL
chmod +x /usr/bin/alp

ENV_FILE="/www/server/panel/.env"
PANEL_PORT=$(shuf -i 10000-65535 -n 1)
PANEL_USER="admin"
PANEL_PASSWORD=$(tr -dc A-Za-z0-9 < /dev/urandom | head -c 16)

cat > $ENV_FILE << EOF
PANEL_PORT=$PANEL_PORT
PANEL_USER=$PANEL_USER
PANEL_PASSWORD=$PANEL_PASSWORD
PANEL_TITLE=Alpanel
PANEL_THEME=auto
EOF

if ! command -v openrc > /dev/null 2>&1; then
    apk add openrc
fi

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

alp 11

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
