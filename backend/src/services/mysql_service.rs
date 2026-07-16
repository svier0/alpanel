use crate::errors::{AppError, AppResult};

const PID_FILE: &str = "/www/server/mysql/run/mysql.pid";
const MYSQL_BIN: &str = "/www/server/mysql/bin/mariadbd";
const MYSQL_ADMIN: &str = "/www/server/mysql/bin/mariadb-admin";
const SOCK_FILE: &str = "/www/server/mysql/run/mysql.sock";
const INIT_SCRIPT: &str = "/etc/init.d/mysql";

fn init_d(action: &str) -> std::process::Output {
    std::process::Command::new(INIT_SCRIPT)
        .arg(action)
        .output()
        .expect("/etc/init.d/mysql failed to run")
}

fn pid_alive(pid: i32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_installed() -> bool {
    std::path::Path::new(MYSQL_BIN).exists()
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
    std::fs::read_to_string("/www/wwwlogs/mysql_error.log")
        .ok()
        .and_then(|log| log.lines().filter(|l| !l.is_empty()).last().map(|s| s.to_string()))
        .unwrap_or_else(|| "未知错误，请查看日志".to_string())
}

pub fn start() -> AppResult<String> {
    let out = init_d("start");
    std::thread::sleep(std::time::Duration::from_millis(500));
    if check_running() {
        Ok("MySQL 已启动".to_string())
    } else {
        let msg = String::from_utf8_lossy(&out.stderr);
        let detail = if msg.trim().is_empty() { last_error() } else { msg.trim().to_string() };
        Err(AppError::Internal(format!("MySQL 启动失败: {}", detail)))
    }
}

pub fn stop() -> AppResult<String> {
    let _ = init_d("stop");
    std::thread::sleep(std::time::Duration::from_millis(500));
    if check_running() {
        Err(AppError::Internal("MySQL 停止失败".to_string()))
    } else {
        Ok("MySQL 已停止".to_string())
    }
}

pub fn restart() -> AppResult<String> {
    stop().ok();
    start()
}

pub fn reload() -> AppResult<String> {
    if !check_running() {
        return Err(AppError::Internal("MySQL 未运行，无法重载".to_string()));
    }
    let out = init_d("reload");
    if out.status.success() {
        Ok("MySQL 已重载".to_string())
    } else {
        Err(AppError::Internal(format!("重载失败: {}", String::from_utf8_lossy(&out.stderr))))
    }
}

pub fn install() -> AppResult<String> {
    let output = std::process::Command::new("alp")
        .arg("53")
        .output()
        .map_err(|e| AppError::Internal(format!("无法执行 alp 命令: {}", e)))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(AppError::Internal(format!(
            "mysql 安装失败: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub fn change_root_pw(new_pw: &str) -> AppResult<String> {
    if !check_installed() {
        return Err(AppError::Internal("MySQL 未安装".to_string()));
    }
    if !check_running() {
        return Err(AppError::Internal("MySQL 未运行，无法修改密码".to_string()));
    }
    if new_pw.is_empty() {
        return Err(AppError::BadRequest("密码不能为空".to_string()));
    }
    if new_pw.contains('\'') {
        return Err(AppError::BadRequest("密码包含非法字符".to_string()));
    }
    let output = std::process::Command::new(MYSQL_ADMIN)
        .arg("-uroot")
        .arg("-S")
        .arg(SOCK_FILE)
        .arg("password")
        .arg(new_pw)
        .output()
        .map_err(|e| AppError::Internal(format!("无法执行 mariadb-admin 命令: {}", e)))?;
    if output.status.success() {
        Ok("root 密码已修改".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Access denied") {
            Err(AppError::Internal(
                "当前 root 需要密码验证，请先通过其他方式修改".to_string(),
            ))
        } else {
            Err(AppError::Internal(format!("修改失败: {}", stderr.trim())))
        }
    }
}
