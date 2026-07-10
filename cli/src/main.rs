use std::fs;
use std::io::{self, Write};
use std::process::Command;

const PANEL_BIN: &str = "/www/server/panel/alpanel";
const PID_FILE: &str = "/var/run/alpanel.pid";
const ENV_FILE: &str = "/www/server/panel/.env";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => help(),
        2 => match args[1].as_str() {
            "11" => start(),
            "12" => stop(),
            "13" => restart(),
            "21" => set_username(),
            "22" => set_password(),
            "31" => set_port(),
            _ => {
                eprintln!("未知命令: alp {}", args[1]);
                help();
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!("用法: alp [命令]");
            help();
            std::process::exit(1);
        }
    }
}

fn help() {
    println!("Alpanel 面板管理工具");
    println!();
    println!("  alp        显示此帮助菜单");
    println!("  alp 11     启动面板服务");
    println!("  alp 12     停止面板服务");
    println!("  alp 13     重启面板服务");
    println!("  alp 21     修改登录账号");
    println!("  alp 22     修改登录密码");
    println!("  alp 31     修改面板端口");
}

fn read_env() -> Vec<(String, String)> {
    let content = fs::read_to_string(ENV_FILE).unwrap_or_else(|e| {
        eprintln!("读取配置文件失败: {} ({})", ENV_FILE, e);
        std::process::exit(1);
    });

    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.splitn(2, '=');
            let key = parts.next()?.trim().to_string();
            let val = parts.next().unwrap_or("").trim().to_string();
            Some((key, val))
        })
        .collect()
}

fn write_env(kvs: &[(String, String)]) {
    let content = kvs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(ENV_FILE, content).unwrap_or_else(|e| {
        eprintln!("写入配置文件失败: {} ({})", ENV_FILE, e);
        std::process::exit(1);
    });
}

fn update_env(key: &str, value: &str) {
    let mut kvs = read_env();
    let mut found = false;
    for (k, v) in &mut kvs {
        if k == key {
            *v = value.to_string();
            found = true;
            break;
        }
    }
    if !found {
        kvs.push((key.to_string(), value.to_string()));
    }
    write_env(&kvs);
    println!("已更新 {}={}", key, value);
}

fn prompt(prompt: &str) -> String {
    print!("{}: ", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// --- commands ---

fn stop() {
    let pid_str = match fs::read_to_string(PID_FILE) {
        Ok(s) => s.trim().to_string(),
        Err(_) => {
            eprintln!("面板服务未运行");
            std::process::exit(1);
        }
    };

    let pid: u32 = match pid_str.parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("PID 文件格式错误");
            std::process::exit(1);
        }
    };

    let status = Command::new("kill").arg(pid.to_string()).status();
    match status {
        Ok(s) if s.success() => {
            let _ = fs::remove_file(PID_FILE);
            println!("面板服务已停止");
        }
        _ => {
            eprintln!("停止面板服务失败");
            std::process::exit(1);
        }
    }
}

fn start() {
    if let Ok(pid_str) = fs::read_to_string(PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let alive = Command::new("kill")
                .args(["-0", &pid.to_string()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if alive {
                eprintln!("面板服务已在运行中 (PID: {})", pid);
                std::process::exit(1);
            }
        }
        let _ = fs::remove_file(PID_FILE);
    }

    let child = Command::new(PANEL_BIN)
        .arg("serve")
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("启动面板服务失败: {} (路径: {})", e, PANEL_BIN);
            std::process::exit(1);
        });

    fs::write(PID_FILE, child.id().to_string()).unwrap_or_else(|e| {
        eprintln!("写入 PID 文件失败: {}", e);
        std::process::exit(1);
    });

    println!("面板服务已启动 (PID: {})", child.id());
}

fn restart() {
    let pid_str = fs::read_to_string(PID_FILE).ok();
    if let Some(pid_str) = pid_str {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let alive = Command::new("kill")
                .args(["-0", &pid.to_string()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if alive {
                let _ = Command::new("kill").arg(pid.to_string()).status();
                let _ = fs::remove_file(PID_FILE);
                println!("面板服务已停止");
            }
        }
    }

    let child = Command::new(PANEL_BIN)
        .arg("serve")
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("启动面板服务失败: {} (路径: {})", e, PANEL_BIN);
            std::process::exit(1);
        });

    fs::write(PID_FILE, child.id().to_string()).unwrap_or_else(|e| {
        eprintln!("写入 PID 文件失败: {}", e);
        std::process::exit(1);
    });

    println!("面板服务已重启 (PID: {})", child.id());
}

fn set_username() {
    let val = prompt("请输入新登录账号");
    if val.is_empty() {
        eprintln!("账号不能为空");
        std::process::exit(1);
    }
    update_env("PANEL_USER", &val);
}

fn read_password(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let _ = Command::new("stty").args(["-echo"]).status();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let _ = Command::new("stty").args(["echo"]).status();
    println!();
    input.trim().to_string()
}

fn set_password() {
    let pw = read_password("请输入新登录密码:");
    if pw.is_empty() {
        eprintln!("密码不能为空");
        std::process::exit(1);
    }
    let confirm = read_password("请再次输入新登录密码:");
    if pw != confirm {
        eprintln!("两次输入的密码不一致");
        std::process::exit(1);
    }
    update_env("PANEL_PASSWORD", &pw);
}

fn set_port() {
    let val = prompt("请输入新面板端口 (10000-65535)");
    let port: u16 = match val.parse() {
        Ok(p) if p >= 10000 => p,
        _ => {
            eprintln!("端口无效，请输入 10000-65535 之间的数字");
            std::process::exit(1);
        }
    };
    update_env("PANEL_PORT", &port.to_string());
}
