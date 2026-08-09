#!/bin/sh
set -eu

ENV_FILE="/www/server/panel/.env"
DB_FILE="/www/server/panel/data/db/alpanel.db"

ghproxy_val=$(grep '^GHPROXY=' "$ENV_FILE" 2>/dev/null | cut -d= -f2-)
GH_PROXY=""
[ -n "$ghproxy_val" ] && [ "$ghproxy_val" != "false" ] && GH_PROXY="$ghproxy_val"
GH_RAW="${GH_PROXY}https://raw.githubusercontent.com/svier0/alpanel-plugins/master"

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
    echo "  alp 51     查看已安装插件"
    echo "  alp 52     插件市场（获取远程插件列表）"
    echo "  alp 53     安装插件 (如 alp 53 nginx)"
    echo "  alp 54     卸载插件 (如 alp 54 nginx)"
    echo "  alp 61     强制修改 MySQL root 密码 (无需旧密码)"
    echo "  alp 99     卸载面板 (删除 /www 及所有服务, 不可恢复)"
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
    printf "%s: " "$1" >&2
    read -r input
    echo "$input"
}

read_password() {
    printf "%s" "$1" >&2
    stty -echo 2>/dev/null
    read -r input
    stty echo 2>/dev/null
    echo ""
    echo "$input"
}

start() {
    start-stop-daemon --start --make-pidfile --pidfile /var/run/alpanel.pid \
        --background --chdir /www/server/panel --exec /www/server/panel/alpanel -- serve
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

ensure_sqlite() {
    if ! command -v sqlite3 >/dev/null 2>&1; then
        echo "需要 sqlite3 来修改用户表, 正在安装..."
        apk add sqlite >/dev/null 2>&1 || { echo "错误: 安装 sqlite 失败" >&2; exit 1; }
    fi
}

user_id() {
    sqlite3 "$DB_FILE" "SELECT id FROM users ORDER BY id LIMIT 1;" 2>/dev/null | head -1
}

set_username() {
    val=$(prompt "请输入新登录账号")
    [ -n "$val" ] || { echo "账号不能为空" >&2; exit 1; }
    ensure_sqlite
    uid=$(user_id)
    [ -n "$uid" ] || { echo "错误: 未找到用户记录" >&2; exit 1; }
    sqlite3 "$DB_FILE" "UPDATE users SET username='$val' WHERE id=$uid;" 2>/dev/null \
        || { echo "错误: 修改账号失败" >&2; exit 1; }
    echo "登录账号已修改为: $val"
    echo "(注意: .env 中的初始账号仅作安装记录, 不再用于登录)"
}

set_password() {
    pw=$(read_password "请输入新登录密码:")
    [ -n "$pw" ] || { echo "密码不能为空" >&2; exit 1; }
    ensure_sqlite
    uid=$(user_id)
    [ -n "$uid" ] || { echo "错误: 未找到用户记录" >&2; exit 1; }
    # 密码存储格式: md5(md5(pw) + salt)
    pw_md5=$(printf '%s' "$pw" | md5sum | awk '{print $1}')
    salt=$(head -c 16 /dev/urandom | md5sum | awk '{print $1}')
    final=$(printf '%s%s' "$pw_md5" "$salt" | md5sum | awk '{print $1}')
    sqlite3 "$DB_FILE" "UPDATE users SET password='$final', salt='$salt' WHERE id=$uid;" 2>/dev/null \
        || { echo "错误: 修改密码失败" >&2; exit 1; }
    echo "登录密码已修改"
    echo "(注意: .env 中的初始密码仅作安装记录, 不再用于登录)"
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

force_mysql_pw() {
    command -v mariadbd >/dev/null 2>&1 || { echo "错误: MySQL 未安装"; exit 1; }
    echo "请输入新的 MySQL root 密码:"
    printf "> "
    read -r newpw
    [ -n "$newpw" ] || { echo "密码不能为空"; exit 1; }

    echo "正在停止 MySQL..."
    /etc/init.d/mysql stop 2>/dev/null || true
    sleep 1

    echo "正在以跳过权限表模式启动..."
    socket="/www/server/mysql/run/mysql.sock"
    pidfile="/www/server/mysql/run/mysql.pid"

    export LD_LIBRARY_PATH=/www/server/mysql/lib
    mariadbd --skip-grant-tables --skip-networking \
        --datadir=/www/server/data --basedir=/www/server/mysql \
        --socket="$socket" --pid-file="$pidfile" \
        --defaults-file=/www/server/mysql/conf/my.cnf \
        --user=root >/dev/null 2>&1 &
    mysql_tmp_pid=$!
    sleep 2

    echo "正在修改密码..."
    if mariadb -uroot -S "$socket" -e \
        "ALTER USER 'root'@'localhost' IDENTIFIED BY '${newpw}'; FLUSH PRIVILEGES;" 2>/dev/null; then
        ok=1
    else
        # 旧版或不支持 ALTER USER 的降级
        mariadb -uroot -S "$socket" -e \
            "UPDATE mysql.user SET plugin='', authentication_string='' WHERE User='root'; FLUSH PRIVILEGES;" 2>/dev/null || {
            echo "错误: 修改失败" >&2
            kill "$mysql_tmp_pid" 2>/dev/null || true
            exit 1
        }
        kill "$mysql_tmp_pid" 2>/dev/null || true
        sleep 1
        rm -f "$pidfile" "$socket"
        /etc/init.d/mysql start 2>/dev/null || true
        sleep 2
        export LD_LIBRARY_PATH=/www/server/mysql/lib
        mariadb -uroot -S "$socket" -e \
            "ALTER USER 'root'@'localhost' IDENTIFIED BY '${newpw}'; FLUSH PRIVILEGES;" 2>/dev/null || {
            echo "错误: 密码清空但 ALTER USER 失败, 请手动设置密码" >&2
            exit 1
        }
        ok=1
    fi

    kill "$mysql_tmp_pid" 2>/dev/null || true
    sleep 1
    rm -f "$pidfile" "$socket"

    /etc/init.d/mysql start 2>/dev/null || true
    echo "MySQL root 密码已修改为: $newpw"
}

list_plugins() {
    plugin_dir="/www/server/panel/plugin"
    printf "["
    first=1
    for dir in "$plugin_dir"/*/; do
        [ -d "$dir" ] || continue
        json="${dir}info.json"
        [ -f "$json" ] || continue
        name=$(basename "$dir")
        content=$(cat "$json")
        dir_name=$(echo "$content" | jq -r '.name // empty')
        [ "$dir_name" = "$name" ] || continue
        [ "$first" -eq 1 ] && first=0 || printf ","
        printf "%s" "$content"
    done
    printf "]"
    echo ""
}

list_market() {
    wget -q --timeout=10 -O - "$GH_RAW/index.json" 2>/dev/null || { echo "[]"; exit 1; }
}

uninstall_plugin() {
    name="${1:-}"
    case "$name" in
        ""|*[!a-zA-Z0-9._-]*) echo "错误: 非法插件名" >&2; exit 1 ;;
    esac
    plugin_dir="/www/server/panel/plugin/$name"
    sh_file="$plugin_dir/$name.sh"
    if [ ! -d "$plugin_dir" ]; then
        echo "错误: 插件 $name 未安装" >&2
        exit 1
    fi
    if [ -f "$sh_file" ]; then
        . "$sh_file" && uninstall 2>/dev/null || true
    fi
    rm -rf "$plugin_dir"
    echo "插件 $name 已卸载"
}

install_plugin() {
    name="${1:-}"
    case "$name" in
        ""|*[!a-zA-Z0-9._-]*) echo "错误: 非法插件名" >&2; exit 1 ;;
    esac
    plugin_dir="/www/server/panel/plugin/$name"
    mkdir -p "$plugin_dir"
    base_url="$GH_RAW/plugins/$name"
    files="info.json ${name}.sh icon.png index.js"
    dl_err=0
    for f in $files; do
        wget -q --timeout=10 "$base_url/$f" -O "$plugin_dir/$f" 2>/dev/null || { dl_err=1; break; }
    done
    if [ "$dl_err" -ne 0 ]; then
        rm -rf "$plugin_dir"
        echo "错误: 下载插件 $name 失败" >&2
        exit 1
    fi
    chmod +x "$plugin_dir/${name}.sh" 2>/dev/null || true
    . "$plugin_dir/${name}.sh" && install
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
    51)  list_plugins ;;
    52)  list_market ;;
    53)  install_plugin "${2:-}" ;;
    54)  uninstall_plugin "${2:-}" ;;
    61)  force_mysql_pw ;;
    99)  uninstall ;;
    *)
        echo "未知命令: alp $1" >&2
        help
        exit 1
        ;;
esac
