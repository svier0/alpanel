use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::LazyLock;
use axum::{Json, http::HeaderMap};
use serde::Serialize;

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;

#[derive(Serialize)]
pub struct SystemInfo {
    pub os_id: String,
    pub os_name: String,
    pub os_version: String,
    pub os_pretty: String,
    pub os_arch: String,
    pub os_uptime: String,
}

pub async fn system_info(
    headers: HeaderMap,
) -> AppResult<Json<SystemInfo>> {
    check_auth(&headers)?;

    let mut os_id = String::new();
    let mut os_name = String::new();
    let mut os_version = String::new();
    let mut os_pretty = String::new();

    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("ID=") {
                os_id = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("NAME=") {
                os_name = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("VERSION_ID=") {
                os_version = val.trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                os_pretty = val.trim_matches('"').to_string();
            }
        }
    }

    let os_arch = std::env::consts::ARCH.to_string();

    let os_uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split('.').next()?.parse::<u64>().ok())
        .map(|secs| {
            let days = secs / 86400;
            let hours = (secs % 86400) / 3600;
            let mins = (secs % 3600) / 60;
            if days > 0 {
                format!("{}天 {}小时 {}分钟", days, hours, mins)
            } else if hours > 0 {
                format!("{}小时 {}分钟", hours, mins)
            } else {
                format!("{}分钟", mins)
            }
        })
        .unwrap_or_else(|| "未知".to_string());

    if os_pretty.is_empty() {
        os_pretty = "Unknown".to_string();
    }

    Ok(Json(SystemInfo {
        os_id,
        os_name,
        os_version,
        os_pretty,
        os_arch,
        os_uptime,
    }))
}

pub async fn list_users(
    headers: HeaderMap,
) -> AppResult<Json<Vec<String>>> {
    check_auth(&headers)?;
    let mut users: Vec<String> = vec!["www".to_string()];
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() < 3 {
                continue;
            }
            if let Ok(uid) = parts[2].parse::<u32>() {
                if uid >= 1000 && uid < 65534 {
                    users.push(parts[0].to_string());
                }
            }
        }
    }
    Ok(Json(users))
}

// ── System Stat ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SystemStat {
    pub cpu: CpuStat,
    pub mem: MemStat,
    pub disks: Vec<DiskStat>,
}

#[derive(Serialize)]
pub struct CpuStat {
    pub name: String,
    pub physical_count: u32,
    pub core_count: u32,
    pub logical_count: u32,
    pub usage_percent: f64,
}

#[derive(Serialize)]
pub struct MemStat {
    pub total: u64,
    pub used: u64,
    pub percent: f64,
}

#[derive(Serialize)]
pub struct DiskStat {
    pub mount: String,
    pub total: u64,
    pub used: u64,
    pub percent: f64,
}

struct CpuRaw {
    total: u64,
    idle: u64,
}

static PREV_CPU: LazyLock<Mutex<Option<CpuRaw>>> = LazyLock::new(|| Mutex::new(None));

pub async fn system_stat(
    headers: HeaderMap,
) -> AppResult<Json<SystemStat>> {
    check_auth(&headers)?;

    // CPU info from /proc/cpuinfo
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut cpu_name = String::new();
    let mut cores = 0u32;
    let mut logical = 0u32;
    let mut in_phys = HashSet::new();

    for line in cpuinfo.lines() {
        if let Some(val) = line.strip_prefix("model name") {
            let v = parse_val(val);
            if cpu_name.is_empty() { cpu_name = v; }
        } else if let Some(val) = line.strip_prefix("physical id") {
            let pid = parse_val(val);
            in_phys.insert(pid);
        } else if let Some(val) = line.strip_prefix("cpu cores") {
            cores = parse_val(val).parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("processor") {
            logical += 1;
        }
    }
    let physical_count = in_phys.len() as u32;
    let core_count = physical_count * cores;

    // CPU usage from /proc/stat
    let raw = parse_proc_stat();
    let usage_percent = {
        let mut prev = PREV_CPU.lock().unwrap();
        match (&*prev, &raw) {
            (Some(p), Some(r)) if r.total > p.total => {
                let d_total = r.total - p.total;
                let d_idle = r.idle - p.idle;
                if d_total > 0 {
                    ((d_total - d_idle) as f64 / d_total as f64) * 100.0
                } else { 0.0 }
            }
            _ => 0.0,
        }
    };
    *PREV_CPU.lock().unwrap() = raw;

    // Memory from /proc/meminfo
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_total = 0u64;
    let mut mem_avail = 0u64;
    for line in meminfo.lines() {
        if let Some(kv) = line.strip_suffix(" kB") {
            if let Some(val) = kv.strip_prefix("MemTotal:") {
                mem_total = val.trim().parse().unwrap_or(0) * 1024;
            } else if let Some(val) = kv.strip_prefix("MemAvailable:") {
                mem_avail = val.trim().parse().unwrap_or(0) * 1024;
            }
        }
    }
    let mem_used = mem_total.saturating_sub(mem_avail);
    let mem_percent = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else { 0.0 };

    // Disk from df -B1
    let disks = parse_df();

    Ok(Json(SystemStat {
        cpu: CpuStat {
            name: cpu_name,
            physical_count,
            core_count,
            logical_count: logical,
            usage_percent,
        },
        mem: MemStat {
            total: mem_total,
            used: mem_used,
            percent: mem_percent,
        },
        disks,
    }))
}

fn parse_val(s: &str) -> String {
    s.split(':').nth(1).map(|v| v.trim().to_string()).unwrap_or_default()
}

fn parse_proc_stat() -> Option<CpuRaw> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 { return None; }
    let vals: Vec<u64> = parts[1..].iter().filter_map(|s| s.parse().ok()).collect();
    let total: u64 = vals.iter().sum();
    let idle = vals.get(3).copied().unwrap_or(0);
    Some(CpuRaw { total, idle })
}

fn parse_df() -> Vec<DiskStat> {
    let output = match std::process::Command::new("df")
        .arg("-B1")
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 { continue; }
        let device = parts[0];
        // real disks: /dev/* or Windows drive (C:\)
        let is_real = device.starts_with("/dev/") ||
            (device.len() == 3 && device.as_bytes().get(1) == Some(&b':') && device.as_bytes().get(2) == Some(&b'\\'));
        if !is_real { continue; }
        let total: u64 = parts[1].parse().unwrap_or(0);
        let used: u64 = parts[2].parse().unwrap_or(0);
        let mount = parts[5].to_string();
        // skip WSL virtual mounts
        if mount.starts_with("/mnt/wsl") { continue; }
        if mount == "/usr/lib/wsl" { continue; }
        let percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
        disks.push(DiskStat { mount, total, used, percent });
    }
    disks
}
