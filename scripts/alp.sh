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
    echo "  alp 52     安装 PHP (可多版本, 如 alp 52 74)"
    echo "  alp 53     安装 MariaDB"
    echo "  alp 54     安装 Redis"
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
    start-stop-daemon --start --make-pidfile --pidfile /var/run/alpanel.pid \
        --background --exec /www/server/panel/alpanel -- serve
    echo "面板服务已启动"
}

stop() {
    if start-stop-daemon --stop --pidfile /var/run/alpanel.pid 2>/dev/null; then
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
    stop
    sleep 1
    start
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

php_versions_from_apk() {
    apk update >/dev/null 2>&1 || return 1
    apk list --available 'php*' 2>/dev/null \
        | sed -n 's/^php\([0-9][0-9]\)-[0-9].*/\1/p' | sort -rnu | tr '\n' ' '
}

install_php() {
    ver="${1:-}"
    versions=$(php_versions_from_apk)
    if [ -z "$versions" ]; then
        echo "错误: 无法从 apk 源获取 PHP 版本列表，请检查 apk 源是否可用" >&2
        exit 1
    fi
    if [ -z "$ver" ]; then
        set -- $versions
        echo "请指定要安装的 PHP 版本:"
        echo ""
        echo "  支持版本: $versions"
        echo ""
        [ -n "${1:-}" ] && echo "  用法: alp 52 $1"
        exit 0
    fi
    case " $versions " in
        *" $ver "*) ;;
        *)
            echo "错误: 不支持的 PHP 版本: $ver (支持的: $versions)" >&2
            echo "用法: alp 52 <版本>" >&2
            exit 1
            ;;
    esac

    echo "正在安装 PHP $ver..."

    command -v apk >/dev/null 2>&1 || { echo "错误: 仅支持 Alpine Linux" >&2; exit 1; }

    php_dir="/www/server/php/$ver"
    bin_dir="$php_dir/bin"
    lib_dir="$php_dir/lib"
    conf_dir="$php_dir/conf"
    run_dir="$php_dir/run"
    log_dir="/www/wwwlogs"

    mkdir -p "$bin_dir" "$lib_dir" "$conf_dir" "$run_dir" "$log_dir"

    dl_dir=$(mktemp -d)
    ext_dir=$(mktemp -d)

    (
        cd "$dl_dir"
        apk fetch --recursive "php$ver" "php$ver-fpm" "php$ver-mysqli" "php$ver-pdo_mysql" \
            "php$ver-gd" "php$ver-curl" "php$ver-mbstring" "php$ver-opcache" "php$ver-zip"
    ) || {
        echo "错误: 未找到 php$ver 相关软件包 (Alpine 可能不含该版本)" >&2
        rm -rf "$dl_dir" "$ext_dir"
        exit 1
    }

    for apk_file in "$dl_dir"/*.apk; do
        [ -f "$apk_file" ] || continue
        tar -xzf "$apk_file" -C "$ext_dir"
    done

    if [ -f "$ext_dir/usr/bin/php$ver" ]; then
        cp -r "$ext_dir/usr/bin/." "$bin_dir/" 2>/dev/null || true
        cp -r "$ext_dir/usr/sbin/." "$bin_dir/" 2>/dev/null || true
        chmod +x "$bin_dir/"* 2>/dev/null || true
        cat > "/usr/bin/php$ver" << PHWRAP
#!/bin/sh
export LD_LIBRARY_PATH=/www/server/php/$ver/lib
exec /www/server/php/$ver/bin/php$ver "\$@"
PHWRAP
        chmod +x "/usr/bin/php$ver"
    else
        echo "错误: 未找到 php$ver 二进制" >&2
        rm -rf "$dl_dir" "$ext_dir"
        exit 1
    fi

    for d in "$ext_dir/lib" "$ext_dir/usr/lib"; do
        [ -d "$d" ] && cp -r "$d/." "$lib_dir/" 2>/dev/null || true
    done

    if [ -d "$ext_dir/etc/php$ver" ]; then
        cp -r "$ext_dir/etc/php$ver/." "$conf_dir/"
    fi

    cat > "$conf_dir/php-fpm.conf" << 'EOF'
[global]
pid = /www/server/php/VERRUN/php-fpm.pid
error_log = /www/wwwlogs/php-fpmVER.log
include=/www/server/php/VER/conf/php-fpm.d/*.conf
EOF
    sed -i "s|VERRUN|$run_dir|g; s|VER|$ver|g" "$conf_dir/php-fpm.conf"

    mkdir -p "$conf_dir/php-fpm.d"
    cat > "$conf_dir/php-fpm.d/www.conf" << 'EOF'
[www]
user = root
group = root
listen = /www/server/php/VERRUN/php-fpmVER.sock
listen.owner = root
listen.group = root
pm = dynamic
pm.max_children = 5
pm.start_servers = 2
pm.min_spare_servers = 1
pm.max_spare_servers = 3
EOF
    sed -i "s|VER|$ver|g" "$conf_dir/php-fpm.d/www.conf"

    cat > "/etc/init.d/php$ver" << 'PHINIT'
#!/bin/sh

PHP_FPM_BIN="/www/server/php/__VER__/bin/php-fpm__VER__"
PHP_FPM_CONF="/www/server/php/__VER__/conf/php-fpm.conf"
PIDFILE="/www/server/php/__VER__/run/php-fpm.pid"

start() {
    mkdir -p /www/server/php/__VER__/run
    export LD_LIBRARY_PATH=/www/server/php/__VER__/lib
    start-stop-daemon --start --background --make-pidfile \
        --pidfile "$PIDFILE" \
        --env LD_LIBRARY_PATH=/www/server/php/__VER__/lib \
        --exec "$PHP_FPM_BIN" -- --fpm-config "$PHP_FPM_CONF"
}

stop() {
    if [ -f "$PIDFILE" ]; then
        start-stop-daemon --stop --pidfile "$PIDFILE" --retry QUIT/5
        rm -f "$PIDFILE"
    fi
}

status() {
    if [ -f "$PIDFILE" ]; then
        read PID < "$PIDFILE"
        if kill -0 "$PID" 2>/dev/null; then
            echo "php__VER__-fpm 运行中 (pid $PID)"
            return 0
        fi
    fi
    echo "php__VER__-fpm 未运行"
    return 1
}

if [ -z "${RC_SVCNAME:-}" ]; then
    case "${1:-}" in
        start)   start ;;
        stop)    stop ;;
        restart) stop; sleep 1; start ;;
        status)  status ;;
        *)       echo "用法: $0 {start|stop|restart|status}" >&2; exit 1 ;;
    esac
fi
PHINIT
    sed -i "s|__VER__|$ver|g" "/etc/init.d/php$ver"
    chmod +x "/etc/init.d/php$ver"

    rm -rf "$dl_dir" "$ext_dir"

    rc-update add "php$ver" default 2>/dev/null || true

    echo "PHP $ver 安装完成"
    echo "  二进制: $bin_dir/php$ver"
    echo "  配置:   $conf_dir/"
    echo "  运行:   rc-service php$ver start"
}

install_mariadb() {
    echo "正在安装 MariaDB..."

    command -v apk >/dev/null 2>&1 || { echo "错误: 仅支持 Alpine Linux" >&2; exit 1; }

    mariadb_dir="/www/server/mysql"
    bin_dir="$mariadb_dir/bin"
    lib_dir="$mariadb_dir/lib"
    conf_dir="$mariadb_dir/conf"
    run_dir="$mariadb_dir/run"
    data_dir="/www/server/data"
    log_dir="/www/wwwlogs"

    mkdir -p "$bin_dir" "$lib_dir" "$conf_dir" "$run_dir" "$data_dir" "$log_dir"

    dl_dir=$(mktemp -d)
    ext_dir=$(mktemp -d)

    (
        cd "$dl_dir"
        apk fetch --recursive mariadb mariadb-client
    )

    for apk_file in "$dl_dir"/*.apk; do
        [ -f "$apk_file" ] || continue
        tar -xzf "$apk_file" -C "$ext_dir"
    done

    if [ -f "$ext_dir/usr/bin/mariadbd" ]; then
        cp -r "$ext_dir/usr/bin/." "$bin_dir/" 2>/dev/null || true
        chmod +x "$bin_dir/"* 2>/dev/null || true
        cat > /usr/bin/mariadbd << 'MARIADBWRAP'
#!/bin/sh
export LD_LIBRARY_PATH=/www/server/mysql/lib
exec /www/server/mysql/bin/mariadbd "$@"
MARIADBWRAP
        chmod +x /usr/bin/mariadbd
    else
        echo "错误: 未找到 mariadbd 二进制" >&2
        rm -rf "$dl_dir" "$ext_dir"
        exit 1
    fi

    for d in "$ext_dir/lib" "$ext_dir/usr/lib"; do
        [ -d "$d" ] && cp -r "$d/." "$lib_dir/" 2>/dev/null || true
    done

    if [ -d "$ext_dir/usr/share/mariadb" ]; then
        mkdir -p "$mariadb_dir/share"
        cp -r "$ext_dir/usr/share/mariadb/." "$mariadb_dir/share/mariadb/" 2>/dev/null || true
    fi

    if [ -d "$ext_dir/etc/mysql" ]; then
        cp -r "$ext_dir/etc/mysql/." "$conf_dir/"
    fi

    cat > "$conf_dir/my.cnf" << 'EOF'
[mysqld]
user=root
basedir=/www/server/mysql
datadir=/www/server/data
pid-file=/www/server/mysql/run/mariadb.pid
socket=/www/server/mysql/run/mariadb.sock
log-error=/www/wwwlogs/mariadb_error.log
character-set-server=utf8mb4
collation-server=utf8mb4_unicode_ci

[client]
socket=/www/server/mysql/run/mariadb.sock
EOF

    if [ ! -d "$data_dir/mysql" ]; then
        echo "正在初始化数据库..."
        export LD_LIBRARY_PATH="$lib_dir"
        "$bin_dir/mariadb-install-db" --defaults-file="$conf_dir/my.cnf" \
            --user=root --datadir="$data_dir" --basedir="$mariadb_dir" >/dev/null 2>&1 || {
            echo "错误: 数据库初始化失败" >&2
            rm -rf "$dl_dir" "$ext_dir"
            exit 1
        }
    fi

    cat > /etc/init.d/mariadb << 'MARIADBINIT'
#!/bin/sh

MARIADBD_BIN="/www/server/mysql/bin/mariadbd"
MY_CNF="/www/server/mysql/conf/my.cnf"
PIDFILE="/www/server/mysql/run/mariadb.pid"

start() {
    mkdir -p /www/server/mysql/run
    export LD_LIBRARY_PATH=/www/server/mysql/lib
    start-stop-daemon --start --background --make-pidfile \
        --pidfile "$PIDFILE" \
        --env LD_LIBRARY_PATH=/www/server/mysql/lib \
        --exec "$MARIADBD_BIN" -- --defaults-file="$MY_CNF"
}

stop() {
    if [ -f "$PIDFILE" ]; then
        start-stop-daemon --stop --pidfile "$PIDFILE" --retry QUIT/5
        rm -f "$PIDFILE"
    fi
}

status() {
    if [ -f "$PIDFILE" ]; then
        read PID < "$PIDFILE"
        if kill -0 "$PID" 2>/dev/null; then
            echo "mariadb 运行中 (pid $PID)"
            return 0
        fi
    fi
    echo "mariadb 未运行"
    return 1
}

if [ -z "${RC_SVCNAME:-}" ]; then
    case "${1:-}" in
        start)   start ;;
        stop)    stop ;;
        restart) stop; sleep 1; start ;;
        status)  status ;;
        *)       echo "用法: $0 {start|stop|restart|status}" >&2; exit 1 ;;
    esac
fi
MARIADBINIT
    chmod +x /etc/init.d/mariadb

    rm -rf "$dl_dir" "$ext_dir"

    rc-update add mariadb default 2>/dev/null || true

    echo "MariaDB 安装完成"
    echo "  二进制: $bin_dir/mariadbd"
    echo "  配置:   $conf_dir/my.cnf"
    echo "  数据:   $data_dir/"
    echo "  日志:   $log_dir/mariadb_error.log"
    echo "启动: rc-service mariadb start"
}

install_redis() {
    echo "正在安装 Redis..."

    command -v apk >/dev/null 2>&1 || { echo "错误: 仅支持 Alpine Linux" >&2; exit 1; }

    redis_dir="/www/server/redis"
    bin_dir="$redis_dir/bin"
    lib_dir="$redis_dir/lib"
    conf_dir="$redis_dir/conf"
    run_dir="$redis_dir/run"
    data_dir="$redis_dir/data"
    log_dir="/www/wwwlogs"

    mkdir -p "$bin_dir" "$lib_dir" "$conf_dir" "$run_dir" "$data_dir" "$log_dir"

    dl_dir=$(mktemp -d)
    ext_dir=$(mktemp -d)

    (
        cd "$dl_dir"
        apk fetch --recursive redis
    )

    for apk_file in "$dl_dir"/*.apk; do
        [ -f "$apk_file" ] || continue
        tar -xzf "$apk_file" -C "$ext_dir"
    done

    if [ -f "$ext_dir/usr/bin/redis-server" ]; then
        cp -r "$ext_dir/usr/bin/." "$bin_dir/" 2>/dev/null || true
        chmod +x "$bin_dir/"* 2>/dev/null || true
        cat > /usr/bin/redis-server << 'REDISWRAP'
#!/bin/sh
export LD_LIBRARY_PATH=/www/server/redis/lib
exec /www/server/redis/bin/redis-server "$@"
REDISWRAP
        chmod +x /usr/bin/redis-server
    else
        echo "错误: 未找到 redis-server 二进制" >&2
        rm -rf "$dl_dir" "$ext_dir"
        exit 1
    fi

    for d in "$ext_dir/lib" "$ext_dir/usr/lib"; do
        [ -d "$d" ] && cp -r "$d/." "$lib_dir/" 2>/dev/null || true
    done

    cat > "$conf_dir/redis.conf" << 'EOF'
bind 0.0.0.0
port 6379
daemonize no
dir /www/server/redis/data
pidfile /www/server/redis/run/redis.pid
logfile /www/wwwlogs/redis.log
EOF

    cat > /etc/init.d/redis << 'REDISINIT'
#!/bin/sh

REDIS_BIN="/www/server/redis/bin/redis-server"
REDIS_CONF="/www/server/redis/conf/redis.conf"
PIDFILE="/www/server/redis/run/redis.pid"

start() {
    mkdir -p /www/server/redis/run
    export LD_LIBRARY_PATH=/www/server/redis/lib
    start-stop-daemon --start --background --make-pidfile \
        --pidfile "$PIDFILE" \
        --env LD_LIBRARY_PATH=/www/server/redis/lib \
        --exec "$REDIS_BIN" -- "$REDIS_CONF"
}

stop() {
    if [ -f "$PIDFILE" ]; then
        start-stop-daemon --stop --pidfile "$PIDFILE" --retry QUIT/5
        rm -f "$PIDFILE"
    fi
}

status() {
    if [ -f "$PIDFILE" ]; then
        read PID < "$PIDFILE"
        if kill -0 "$PID" 2>/dev/null; then
            echo "redis 运行中 (pid $PID)"
            return 0
        fi
    fi
    echo "redis 未运行"
    return 1
}

if [ -z "${RC_SVCNAME:-}" ]; then
    case "${1:-}" in
        start)   start ;;
        stop)    stop ;;
        restart) stop; sleep 1; start ;;
        status)  status ;;
        *)       echo "用法: $0 {start|stop|restart|status}" >&2; exit 1 ;;
    esac
fi
REDISINIT
    chmod +x /etc/init.d/redis

    rm -rf "$dl_dir" "$ext_dir"

    rc-update add redis default 2>/dev/null || true

    echo "Redis 安装完成"
    echo "  二进制: $bin_dir/redis-server"
    echo "  配置:   $conf_dir/redis.conf"
    echo "  数据:   $data_dir/"
    echo "  日志:   $log_dir/redis.log"
    echo "启动: rc-service redis start"
}

case "${1:-}" in
    "")  [ -n "${RC_SVCNAME:-}" ] || help ;;
    0)   echo "已取消"; exit 0 ;;
    start)   start ;;
    stop)    stop ;;
    restart) restart ;;
    status)
        if [ -f /var/run/alpanel.pid ] && kill -0 "$(cat /var/run/alpanel.pid)" 2>/dev/null; then
            echo "alpanel 运行中"
        else
            echo "alpanel 未运行"
        fi
        ;;
    11)  start ;;
    12)  stop ;;
    13)  restart ;;
    21)  set_username ;;
    22)  set_password ;;
    31)  set_port ;;
    51)  install_nginx ;;
    52)  install_php "${2:-}" ;;
    53)  install_mariadb ;;
    54)  install_redis ;;
    *)
        echo "未知命令: alp $1" >&2
        help
        exit 1
        ;;
esac
