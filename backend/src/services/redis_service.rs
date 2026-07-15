use crate::errors::{AppError, AppResult};

const REDIS_BIN: &str = "/www/server/redis/bin/redis-server";
const PID_FILE: &str = "/www/server/redis/run/redis.pid";

fn pid_alive(pid: i32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_installed() -> bool {
    std::path::Path::new(REDIS_BIN).exists()
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

pub fn install() -> AppResult<String> {
    let output = std::process::Command::new("alp")
        .arg("54")
        .output()
        .map_err(|e| AppError::Internal(format!("无法执行 alp 命令: {}", e)))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(AppError::Internal(format!(
            "redis 安装失败: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}
