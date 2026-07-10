#!/bin/bash

DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/refs/heads/master/scripts/install.sh"
PANEL_DOWNLOAD_URL="https://github.com/svier0/alpanel/releases/latest/download/alpanel-linux-amd64"
ALP_DOWNLOAD_URL="https://raw.githubusercontent.com/svier0/alpanel/refs/heads/master/scripts/alp.sh"

if wget --spider --timeout=1 --tries=1 -q https://www.google.com > /dev/null 2>&1; then
    REPO_URL="/etc/apk/repositories"
    GH_PROXY=""
else
    echo "https://mirrors.aliyun.com/alpine/v3.21/main" > /etc/apk/repositories
    echo "https://mirrors.aliyun.com/alpine/v3.21/community" >> /etc/apk/repositories
    GH_PROXY="https://gh-proxy.com/"
fi
apk update

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

mkdir -p /www
mkdir -p /www/wwwlogs
mkdir -p /www/wwwroot
mkdir -p /www/server/panel

wget -O /www/server/panel/alpanel ${GH_PROXY}$PANEL_DOWNLOAD_URL
chmod +x /www/server/panel/alpanel

wget -O /usr/local/bin/alp ${GH_PROXY}$ALP_DOWNLOAD_URL
chmod +x /usr/local/bin/alp

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

description="Alpanel Service"

command="/www/server/panel/alpanel"
command_args="serve"
pidfile="/var/run/alpanel.pid"

depend() {
    need net
}

start() {
    ebegin "Starting Alpanel"
    start-stop-daemon --start --make-pidfile --pidfile $pidfile \
        --background --exec $command -- $command_args
    eend $?
}

stop() {
    ebegin "Stopping Alpanel"
    start-stop-daemon --stop --pidfile $pidfile
    eend $?
}
EOF

chmod +x /etc/init.d/alpanel
rc-update add alpanel default

rc-service alpanel start

echo "================================"
echo " Alpanel 安装完成"
echo "================================"
echo "面板地址: http://$(wget -q -O- ifconfig.me/ip):$PANEL_PORT"
echo "账号: $PANEL_USER"
echo "密码: $PANEL_PASSWORD"
echo "================================"
echo "面板配置目录: /www/server/panel/"
echo "使用 alp 命令管理面板"
echo "================================"
