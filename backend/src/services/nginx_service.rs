use crate::errors::{AppError, AppResult};

const PID_FILE: &str = "/www/server/nginx/run/nginx.pid";
const NGINX_BIN: &str = "/www/server/nginx/sbin/nginx";
const NGINX_CONF: &str = "/www/server/nginx/conf/nginx.conf";

fn ssd(args: &[&str]) -> std::process::Output {
    std::process::Command::new("start-stop-daemon")
        .args(args)
        .output()
        .expect("start-stop-daemon failed to run")
}

fn pid_alive(pid: i32) -> bool {
    std::process::Command::new("kill")
        .arg("-0").arg(pid.to_string())
        .output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn check_installed() -> bool {
    std::path::Path::new(NGINX_BIN).exists()
}

pub fn check_running() -> bool {
    let pid_path = std::path::Path::new(PID_FILE);
    if !pid_path.exists() {
        return false;
    }
    let s = match std::fs::read_to_string(pid_path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let pid: i32 = match s.trim().parse() {
        Ok(p) => p,
        Err(_) => return false,
    };
    pid_alive(pid)
}

fn last_error() -> String {
    std::fs::read_to_string("/www/wwwlogs/nginx_error.log")
        .ok()
        .and_then(|log| log.lines().filter(|l| !l.is_empty()).last().map(|s| s.to_string()))
        .unwrap_or_else(|| "未知错误，请查看日志".to_string())
}

pub fn start() -> AppResult<String> {
    let _ = std::fs::remove_file(PID_FILE);
    let out = ssd(&[
        "--start", "--background", "--make-pidfile",
        "--pidfile", PID_FILE,
        "--env", "LD_LIBRARY_PATH=/www/server/nginx/lib",
        "--exec", NGINX_BIN, "--",
        "-e", "/www/wwwlogs/nginx_error.log",
        "-c", NGINX_CONF,
    ]);
    if out.status.success() {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if check_running() {
            Ok("Nginx 已启动".to_string())
        } else {
            Err(AppError::Internal(format!("Nginx 启动失败: {}", last_error())))
        }
    } else {
        let msg = String::from_utf8_lossy(&out.stderr);
        Err(AppError::Internal(format!("Nginx 启动失败: {}", msg)))
    }
}

pub fn stop() -> AppResult<String> {
    let out = ssd(&["--stop", "--pidfile", PID_FILE, "--retry", "QUIT/5"]);
    std::thread::sleep(std::time::Duration::from_millis(500));
    if check_running() {
        Err(AppError::Internal("Nginx 停止失败".to_string()))
    } else {
        let _ = std::fs::remove_file(PID_FILE);
        Ok("Nginx 已停止".to_string())
    }
}

pub fn restart() -> AppResult<String> {
    stop().ok();
    start()
}

pub fn reload() -> AppResult<String> {
    let pid_path = std::path::Path::new(PID_FILE);
    if !pid_path.exists() {
        return Err(AppError::Internal("Nginx 未运行，无法重载".to_string()));
    }
    let s = std::fs::read_to_string(pid_path).map_err(|_| AppError::Internal("无法读取 pid 文件".to_string()))?;
    let pid: i32 = s.trim().parse().map_err(|_| AppError::Internal("pid 文件格式错误".to_string()))?;
    if !pid_alive(pid) {
        let _ = std::fs::remove_file(PID_FILE);
        return Err(AppError::Internal("Nginx 未运行，无法重载".to_string()));
    }
    let out = std::process::Command::new("kill")
        .args(["-HUP", &pid.to_string()])
        .output()
        .map_err(|e| AppError::Internal(format!("重载失败: {}", e)))?;
    if out.status.success() {
        Ok("Nginx 已重载".to_string())
    } else {
        Err(AppError::Internal(format!("重载失败: {}", String::from_utf8_lossy(&out.stderr))))
    }
}

pub fn install() -> AppResult<String> {
    let output = std::process::Command::new("alp")
        .arg("51")
        .output()
        .map_err(|e| AppError::Internal(format!("无法执行 alp 命令: {}", e)))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(AppError::Internal(format!("nginx 安装失败: {}", String::from_utf8_lossy(&output.stderr))))
    }
}
