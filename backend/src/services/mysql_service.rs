use crate::db::pool;
use crate::errors::{AppError, AppResult};

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn random_root_pw(len: usize) -> String {
    let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut bytes = [0u8; 64];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = std::io::Read::read_exact(&mut f, &mut bytes);
    } else {
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (i as u8).wrapping_add(7);
        }
    }
    let mut s = String::with_capacity(len);
    for i in 0..len {
        s.push(chars[(bytes[i] as usize) % chars.len()] as char);
    }
    s
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let mut acc: u32 = 0;
    let mut n = 0u32;
    for c in input.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => continue,
            _ => return None,
        };
        acc = (acc << 6) | v as u32;
        n += 1;
        if n == 4 {
            buf.push((acc >> 16) as u8);
            buf.push((acc >> 8) as u8);
            buf.push(acc as u8);
            acc = 0;
            n = 0;
        }
    }
    if n == 3 {
        buf.push((acc >> 10) as u8);
        buf.push((acc >> 2) as u8);
    } else if n == 2 {
        buf.push((acc >> 4) as u8);
    }
    Some(buf)
}

pub fn get_root_pw() -> String {
    let encoded = pool::get_config("mysql_root").unwrap_or_default();
    if encoded.is_empty() {
        return String::new();
    }
    match base64_decode(&encoded) {
        Some(b) => String::from_utf8_lossy(&b).to_string(),
        None => String::new(),
    }
}

const PID_FILE: &str = "/www/server/mysql/run/mysql.pid";
const MYSQL_BIN: &str = "/www/server/mysql/bin/mariadbd";
const MYSQL_CLIENT: &str = "/www/server/mysql/bin/mariadb";
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

pub fn get_version() -> Option<String> {
    if !check_installed() {
        return None;
    }
    let out = std::process::Command::new(MYSQL_BIN)
        .arg("--version")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    // "mariadbd  Ver 11.4.5-MariaDB"
    let ver = s.split("Ver ").nth(1)?.trim();
    let (v, engine) = match ver.split_once('-') {
        Some((v, e)) => (v, e.trim()),
        None => (ver, "MariaDB"),
    };
    let parts: Vec<&str> = v.splitn(3, '.').collect();
    if parts.len() >= 2 {
        Some(format!("{} {}.{}", engine, parts[0], parts[1]))
    } else {
        Some(format!("{} {}", engine, v))
    }
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
    if !output.status.success() {
        return Err(AppError::Internal(format!(
            "mysql 安装失败: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let mut msg = String::from_utf8_lossy(&output.stdout).to_string();
    if check_installed() {
        let _ = start();
        std::thread::sleep(std::time::Duration::from_millis(3000));
        if check_running() {
            match set_random_root_pw() {
                Ok(pw) => {
                    pool::set_config("mysql_root", &base64_encode(pw.as_bytes()));
                    msg.push_str("\n已生成随机 root 密码");
                }
                Err(e) => msg.push_str(&format!("\n初始 root 密码设置失败: {}", e)),
            }
        }
    }
    Ok(msg)
}

fn set_random_root_pw() -> Result<String, AppError> {
    let pw = random_root_pw(16);
    let sql = format!(
        "ALTER USER 'root'@'localhost' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;",
        pw.replace('\'', "''")
    );
    let out = std::process::Command::new(MYSQL_CLIENT)
        .arg("-uroot")
        .arg("-S")
        .arg(SOCK_FILE)
        .arg("-e")
        .arg(sql)
        .output()
        .map_err(|e| AppError::Internal(format!("无法执行 mariadb 命令: {}", e)))?;
    if out.status.success() {
        Ok(pw)
    } else {
        Err(AppError::Internal(format!(
            "修改失败: {}",
            String::from_utf8_lossy(&out.stderr).trim()
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
    let current_pw = get_root_pw();
    let sql = format!(
        "ALTER USER 'root'@'localhost' IDENTIFIED BY '{}'; FLUSH PRIVILEGES;",
        new_pw.replace('\'', "''")
    );
    let mut cmd = std::process::Command::new(MYSQL_CLIENT);
    cmd.arg("-uroot").arg("-S").arg(SOCK_FILE);
    if !current_pw.is_empty() {
        cmd.arg(format!("-p{}", current_pw));
    }
    cmd.arg("-e").arg(sql);
    let output = cmd.output()
        .map_err(|e| AppError::Internal(format!("无法执行 mariadb 命令: {}", e)))?;
    if output.status.success() {
        pool::set_config("mysql_root", &base64_encode(new_pw.as_bytes()));
        Ok("root 密码已修改".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::Internal(format!("修改失败: {}", stderr.trim())))
    }
}
