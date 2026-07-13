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

case "${1:-}" in
    "")  help ;;
    0)   echo "已取消"; exit 0 ;;
    11)  start ;;
    12)  stop ;;
    13)  restart ;;
    21)  set_username ;;
    22)  set_password ;;
    31)  set_port ;;
    *)
        echo "未知命令: alp $1" >&2
        help
        exit 1
        ;;
esac
