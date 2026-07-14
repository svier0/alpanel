#!/bin/sh
set -eu

ENV_FILE="/www/server/panel/.env"

help() {
    echo "Alpanel 面板管理工具"
    echo ""
    echo "  alp        显示此帮助菜单"
    echo "  alp 11     启动面板服务"
    echo "  alp 12     停止面板服务"
    echo "  alp 13     重启面板服务"
    echo "  alp 21     修改登录账号"
    echo "  alp 22     修改登录密码"
    echo "  alp 31     修改面板端口"
    echo "  alp 51     安装 Nginx"
    echo "  alp 0      取消"
}

update_env() {
    key="$1" val="$2"
    tmp=$(mktemp) || exit 1
    found=0
    if [ -f "$ENV_FILE" ]; then
        while IFS='=' read -r k v; do
            case "$k" in
                ""|[#]*) continue ;;
                "$key") echo "$key=$val" >> "$tmp"; found=1 ;;
                *)       echo "$k=$v" >> "$tmp" ;;
            esac
        done < "$ENV_FILE"
    fi
    if [ "$found" -eq 0 ]; then
        echo "$key=$val" >> "$tmp"
    fi
    mv "$tmp" "$ENV_FILE"
    echo "已更新 $key=$val"
}

prompt() {
    printf "%s: " "$1"
    read -r input
    echo "$input"
}

read_password() {
    printf "%s" "$1"
    stty -echo 2>/dev/null
    read -r input
    stty echo 2>/dev/null
    echo ""
    echo "$input"
}

start() {
    rc-service alpanel start
    echo "面板服务已启动"
}

stop() {
    if rc-service alpanel stop 2>/dev/null; then
        echo "面板服务已停止"
        return
    fi
    pid=$(pgrep -x alpanel 2>/dev/null || true)
    if [ -n "$pid" ]; then
        kill "$pid" 2>/dev/null || true
        sleep 1
        kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
        echo "面板服务已停止"
    else
        echo "面板服务未运行"
    fi
}

restart() {
    rc-service alpanel restart
    echo "面板服务已重启"
}

set_username() {
    val=$(prompt "请输入新登录账号")
    [ -n "$val" ] || { echo "账号不能为空" >&2; exit 1; }
    update_env "PANEL_USER" "$val"
}

set_password() {
    pw=$(read_password "请输入新登录密码:")
    [ -n "$pw" ] || { echo "密码不能为空" >&2; exit 1; }
    confirm=$(read_password "请再次输入新登录密码:")
    if [ "$pw" != "$confirm" ]; then
        echo "两次输入的密码不一致" >&2
        exit 1
    fi
    update_env "PANEL_PASSWORD" "$pw"
}

set_port() {
    val=$(prompt "请输入新面板端口 (10000-65535)")
    case "$val" in
        ""|*[!0-9]*)
            echo "端口无效，请输入 10000-65535 之间的数字" >&2
            exit 1
            ;;
    esac
    if [ "$val" -lt 10000 ] || [ "$val" -gt 65535 ]; then
        echo "端口无效，请输入 10000-65535 之间的数字" >&2
        exit 1
    fi
    update_env "PANEL_PORT" "$val"
    restart
}

install_nginx() {
    echo "正在安装 Nginx..."

    command -v apk >/dev/null 2>&1 || { echo "错误: 仅支持 Alpine Linux" >&2; exit 1; }

    nginx_dir="/www/server/nginx"
    conf_dir="$nginx_dir/conf"
    run_dir="$nginx_dir/run"
    log_dir="/www/wwwlogs"
    vhost_dir="/www/server/panel/vhost/nginx"

    mkdir -p "$nginx_dir" "$conf_dir" "$run_dir" "$vhost_dir" "$log_dir"

    dl_dir=$(mktemp -d)
    ext_dir=$(mktemp -d)

    (
        cd "$dl_dir"
        apk fetch --recursive nginx
    )

    for apk_file in "$dl_dir"/*.apk; do
        [ -f "$apk_file" ] || continue
        tar -xzf "$apk_file" -C "$ext_dir"
    done

    if [ -f "$ext_dir/usr/sbin/nginx" ]; then
        mkdir -p "$nginx_dir/sbin"
        cp "$ext_dir/usr/sbin/nginx" "$nginx_dir/sbin/nginx"
        chmod +x "$nginx_dir/sbin/nginx"
        cat > /usr/bin/nginx << 'NGINXWRAP'
#!/bin/sh
export LD_LIBRARY_PATH=/www/server/nginx/lib
exec /www/server/nginx/sbin/nginx "$@"
NGINXWRAP
        chmod +x /usr/bin/nginx
    else
        echo "错误: 未找到 nginx 二进制" >&2
        rm -rf "$dl_dir" "$ext_dir"
        exit 1
    fi

    mkdir -p "$nginx_dir/lib"
    for d in "$ext_dir/lib" "$ext_dir/usr/lib"; do
        [ -d "$d" ] && cp -r "$d/." "$nginx_dir/lib/" 2>/dev/null || true
    done

    if [ -d "$ext_dir/etc/nginx" ]; then
        cp -r "$ext_dir/etc/nginx/." "$conf_dir/"
    fi

    cat > "$conf_dir/nginx.conf" << 'EOF'
user root;
worker_processes auto;
pid /www/server/nginx/run/nginx.pid;
error_log /www/wwwlogs/nginx_error.log warn;

events {
    worker_connections 1024;
}

http {
    include mime.types;
    default_type application/octet-stream;

    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /www/wwwlogs/nginx_access.log main;
    sendfile on;
    tcp_nopush on;
    keepalive_timeout 65;

    include /www/server/panel/vhost/nginx/*.conf;
}
EOF

    cat > /etc/init.d/nginx << 'NGINXINIT'
#!/bin/sh

NGINX_BIN="/www/server/nginx/sbin/nginx"
NGINX_CONF="/www/server/nginx/conf/nginx.conf"
PIDFILE="/www/server/nginx/run/nginx.pid"
ERRLOG="/www/wwwlogs/nginx_error.log"

start() {
    mkdir -p /www/server/nginx/run
    rm -f "$PIDFILE"
    export LD_LIBRARY_PATH=/www/server/nginx/lib
    start-stop-daemon --start --background --make-pidfile \
        --pidfile "$PIDFILE" \
        --env LD_LIBRARY_PATH=/www/server/nginx/lib \
        --exec "$NGINX_BIN" -- -e "$ERRLOG" -c "$NGINX_CONF"
}

stop() {
    if [ -f "$PIDFILE" ]; then
        start-stop-daemon --stop --pidfile "$PIDFILE" --retry QUIT/5
        rm -f "$PIDFILE"
    fi
}

reload() {
    if [ -f "$PIDFILE" ]; then
        read PID < "$PIDFILE"
        kill -HUP "$PID" 2>/dev/null
    fi
}

status() {
    if [ -f "$PIDFILE" ]; then
        read PID < "$PIDFILE"
        if kill -0 "$PID" 2>/dev/null; then
            echo "nginx 运行中 (pid $PID)"
            return 0
        fi
    fi
    echo "nginx 未运行"
    return 1
}

# 被 openrc-run source 时只定义函数，不执行
if [ -z "${RC_SVCNAME:-}" ]; then
    case "${1:-}" in
        start)   start ;;
        stop)    stop ;;
        restart) stop; sleep 1; start ;;
        reload)  reload ;;
        status)  status ;;
        *)       echo "用法: $0 {start|stop|restart|reload|status}" >&2; exit 1 ;;
    esac
fi
NGINXINIT
    chmod +x /etc/init.d/nginx

    rm -rf "$dl_dir" "$ext_dir"

    rc-update add nginx default 2>/dev/null || true

    echo "Nginx 安装完成"
    echo "  二进制: $nginx_dir/sbin/nginx"
    echo "  配置:   $conf_dir/nginx.conf"
    echo "  站点:   $vhost_dir/"
    echo "  日志:   $log_dir/"
    echo "启动: rc-service nginx start"
}

case "${1:-}" in
    "")  help ;;
    0)   echo "已取消"; exit 0 ;;
    11)  start ;;
    12)  stop ;;
    13)  restart ;;
    21)  set_username ;;
    22)  set_password ;;
    31)  set_port ;;
    51)  install_nginx ;;
    *)
        echo "未知命令: alp $1" >&2
        help
        exit 1
        ;;
esac
