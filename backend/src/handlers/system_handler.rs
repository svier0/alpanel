use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use std::sync::LazyLock;
use axum::{Json, http::HeaderMap, extract::Path};
use serde::Serialize;

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::db::pool::db_conn;

#[derive(Serialize)]
pub struct SystemInfo {
    pub os_id: String,
    pub os_name: String,
    pub os_version: String,
    pub os_pretty: String,
    pub os_arch: String,
    pub os_uptime: String,
    pub hostname: String,
    pub kernel: String,
    pub ip: String,
    pub boot_time: String,
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
            let secs = secs % 60;
            format!("{}天 {}小时 {}分钟 {}秒", days, hours, mins, secs)
        })
        .unwrap_or_else(|| "未知".to_string());

    let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .unwrap_or_default()
        .trim()
        .to_string();

    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .unwrap_or_default()
        .trim()
        .to_string();

    let ip = std::process::Command::new("ip")
        .args(["-4", "addr", "show", "scope", "global"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines()
                .find(|l| l.trim().starts_with("inet"))
                .and_then(|l| l.trim().split_whitespace().nth(1))
                .map(|a| a.split('/').next().unwrap_or(a).to_string())
        })
        .unwrap_or_default();

    let boot_time = std::fs::read_to_string("/proc/stat")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("btime"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<i64>().ok())
                .map(|ts| {
                    let dt = chrono::DateTime::from_timestamp(ts, 0)
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default();
                    dt
                })
        })
        .unwrap_or_default();

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
        hostname,
        kernel,
        ip,
        boot_time,
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
    pub loadavg: LoadAvg,
    pub cpu: CpuStat,
    pub cpu_detail: CpuDetail,
    pub mem: MemStat,
    pub mem_detail: MemDetail,
    pub disks: Vec<DiskStat>,
    pub disk_detail: Vec<DiskDetail>,
    pub net: Vec<NetStat>,
    pub disk_io: DiskIo,
    pub overview: Overview,
}

#[derive(Serialize)]
pub struct Overview {
    pub sites: i64,
    pub databases: i64,
    pub apps: i64,
}

#[derive(Serialize)]
pub struct NetStat {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Serialize)]
pub struct DiskIo {
    pub name: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
}

#[derive(Serialize)]
pub struct LoadAvg {
    pub load1: f64,
    pub load5: f64,
    pub load15: f64,
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
pub struct CpuDetail {
    pub freq: u32,
    pub per_core: Vec<f64>,
    pub breakdown: CpuBreakdown,
    pub top_procs: Vec<ProcStat>,
}

#[derive(Serialize)]
pub struct CpuBreakdown {
    pub user: f64,
    pub nice: f64,
    pub system: f64,
    pub idle: f64,
    pub iowait: f64,
    pub irq: f64,
    pub softirq: f64,
    pub steal: f64,
}

#[derive(Serialize)]
pub struct ProcStat {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
}

#[derive(Serialize)]
pub struct MemStat {
    pub total: u64,
    pub used: u64,
    pub percent: f64,
}

#[derive(Serialize)]
pub struct MemDetail {
    pub total: u64,
    pub used: u64,
    pub avail: u64,
    pub free: u64,
    pub cached: u64,
    pub shared: u64,
    pub percent: f64,
    pub top_procs: Vec<MemProc>,
}

#[derive(Serialize)]
pub struct MemProc {
    pub pid: u32,
    pub name: String,
    pub mem_bytes: u64,
    pub percent: f64,
}

#[derive(Serialize)]
pub struct DiskStat {
    pub mount: String,
    pub total: u64,
    pub used: u64,
    pub percent: f64,
}

#[derive(Serialize)]
pub struct DiskDetail {
    pub mount: String,
    pub device: String,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub avail: u64,
    pub percent: f64,
    pub inode_total: u64,
    pub inode_used: u64,
    pub inode_percent: f64,
}

struct CpuRaw {
    total: u64,
    idle: u64,
}

struct CoreRaw {
    total: u64,
    idle: u64,
    user: u64,
    nice: u64,
    system: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

static PREV_CPU: LazyLock<Mutex<Option<CpuRaw>>> = LazyLock::new(|| Mutex::new(None));
static PREV_CORES: LazyLock<Mutex<Vec<CoreRaw>>> = LazyLock::new(|| Mutex::new(Vec::new()));

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

    // ── CPU Detail ──
    // frequency
    let freq = cpuinfo.lines()
        .find(|l| l.contains("cpu MHz"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|v| v.trim().split('.').next())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    // per-core usage & breakdown from /proc/stat
    let stat_content = std::fs::read_to_string("/proc/stat").unwrap_or_default();
    let core_lines: Vec<&str> = stat_content.lines().filter(|l| l.starts_with("cpu") && l.as_bytes().get(3).map_or(false, |c| c.is_ascii_digit())).collect();
    let mut cores_now = Vec::new();
    for line in &core_lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 { continue; }
        let vals: Vec<u64> = parts[1..].iter().filter_map(|s| s.parse().ok()).collect();
        if vals.len() < 8 { continue; }
        cores_now.push(CoreRaw {
            total: vals.iter().sum(),
            idle: vals[3],
            user: vals[0], nice: vals[1], system: vals[2],
            iowait: vals[4], irq: vals[5], softirq: vals[6], steal: vals[7],
        });
    }
    let mut per_core = Vec::new();
    let mut detail = CoreRaw { total: 0, idle: 0, user: 0, nice: 0, system: 0, iowait: 0, irq: 0, softirq: 0, steal: 0 };
    {
        let mut prev = PREV_CORES.lock().unwrap();
        for (i, now) in cores_now.iter().enumerate() {
            if let Some(p) = prev.get(i) {
                let dt = now.total - p.total;
                if dt > 0 {
                    let pct = ((dt - (now.idle - p.idle)) as f64 / dt as f64) * 100.0;
                    per_core.push(pct);
                    detail.user += now.user - p.user;
                    detail.nice += now.nice - p.nice;
                    detail.system += now.system - p.system;
                    detail.idle += now.idle - p.idle;
                    detail.iowait += now.iowait - p.iowait;
                    detail.irq += now.irq - p.irq;
                    detail.softirq += now.softirq - p.softirq;
                    detail.steal += now.steal - p.steal;
                }
            }
        }
        *prev = cores_now;
    }
    let d_total = detail.total as f64;
    let breakdown = if d_total > 0.0 {
        CpuBreakdown {
            user: detail.user as f64 / d_total * 100.0,
            nice: detail.nice as f64 / d_total * 100.0,
            system: detail.system as f64 / d_total * 100.0,
            idle: detail.idle as f64 / d_total * 100.0,
            iowait: detail.iowait as f64 / d_total * 100.0,
            irq: detail.irq as f64 / d_total * 100.0,
            softirq: detail.softirq as f64 / d_total * 100.0,
            steal: detail.steal as f64 / d_total * 100.0,
        }
    } else {
        CpuBreakdown { user: 0.0, nice: 0.0, system: 0.0, idle: 0.0, iowait: 0.0, irq: 0.0, softirq: 0.0, steal: 0.0 }
    };

    // Top 5 processes by CPU
    let top_procs = parse_top_procs();

    let cpu_detail = CpuDetail {
        freq,
        per_core,
        breakdown,
        top_procs,
    };

    // Memory from /proc/meminfo
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_total = 0u64;
    let mut mem_avail = 0u64;
    let mut mem_free = 0u64;
    let mut mem_cached = 0u64;
    let mut mem_buffers = 0u64;
    let mut mem_shmem = 0u64;
    let mut mem_sreclaimable = 0u64;
    for line in meminfo.lines() {
        if let Some(kv) = line.strip_suffix(" kB") {
            if let Some(val) = kv.strip_prefix("MemTotal:") {
                mem_total = val.trim().parse().unwrap_or(0) * 1024;
            } else if let Some(val) = kv.strip_prefix("MemAvailable:") {
                mem_avail = val.trim().parse().unwrap_or(0) * 1024;
            } else if let Some(val) = kv.strip_prefix("MemFree:") {
                mem_free = val.trim().parse().unwrap_or(0) * 1024;
            } else if let Some(val) = kv.strip_prefix("Cached:") {
                mem_cached = val.trim().parse().unwrap_or(0) * 1024;
            } else if let Some(val) = kv.strip_prefix("Buffers:") {
                mem_buffers = val.trim().parse().unwrap_or(0) * 1024;
            } else if let Some(val) = kv.strip_prefix("Shmem:") {
                mem_shmem = val.trim().parse().unwrap_or(0) * 1024;
            } else if let Some(val) = kv.strip_prefix("SReclaimable:") {
                mem_sreclaimable = val.trim().parse().unwrap_or(0) * 1024;
            }
        }
    }
    let mem_used = mem_total.saturating_sub(mem_avail);
    let mem_percent = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else { 0.0 };
    let mem_cache = mem_cached + mem_sreclaimable + mem_buffers;
    let mem_top = parse_top_mem_procs(mem_total);

    let mem_detail = MemDetail {
        total: mem_total,
        used: mem_used,
        avail: mem_avail,
        free: mem_free,
        cached: mem_cache,
        shared: mem_shmem,
        percent: mem_percent,
        top_procs: mem_top,
    };

    // Disk from df -B1
    let disks = parse_df();
    let disk_detail = parse_disk_detail();

    // Load average from /proc/loadavg
    let loadavg = std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default();
    let load_parts: Vec<f64> = loadavg
        .split_whitespace()
        .take(3)
        .filter_map(|s| s.parse().ok())
        .collect();
    let loadavg = LoadAvg {
        load1: load_parts.first().copied().unwrap_or(0.0),
        load5: load_parts.get(1).copied().unwrap_or(0.0),
        load15: load_parts.get(2).copied().unwrap_or(0.0),
    };

    // Network from /proc/net/dev
    let net = parse_net();

    // Disk I/O from /proc/diskstats
    let disk_io = parse_disk_io();

    // Overview from database
    let overview = {
        let sites = db_conn()
            .map(|c| c.query_row("SELECT COUNT(*) FROM sites", [], |r| r.get(0)).unwrap_or(0))
            .unwrap_or(0);
        let databases = 0i64;
        let mut apps = 0i64;
        if std::path::Path::new("/www/server/nginx/sbin/nginx").exists() { apps += 1; }
        if std::path::Path::new("/www/server/mysql/bin/mariadbd").exists() { apps += 1; }
        if std::path::Path::new("/www/server/redis/bin/redis-server").exists() { apps += 1; }
        Overview { sites, databases, apps }
    };

    Ok(Json(SystemStat {
        loadavg,
        cpu: CpuStat {
            name: cpu_name,
            physical_count,
            core_count,
            logical_count: logical,
            usage_percent,
        },
        cpu_detail,
        mem: MemStat {
            total: mem_total,
            used: mem_used,
            percent: mem_percent,
        },
        mem_detail,
        disks,
        disk_detail,
        net,
        disk_io,
        overview,
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

fn parse_disk_detail() -> Vec<DiskDetail> {
    // read /proc/mounts for device and fs_type
    let mounts = std::fs::read_to_string("/proc/mounts").unwrap_or_default();
    let mut mount_info: Vec<(String, String, String)> = Vec::new();
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 { continue; }
        let device = parts[0].to_string();
        let mount = parts[1].to_string();
        let fs_type = parts[2].to_string();
        if mount == "/" || (mount.starts_with("/mnt/") && !mount.starts_with("/mnt/wsl")) {
            mount_info.push((mount, device, fs_type));
        }
    }

    // run df -B1 for usage
    let mut usage: std::collections::HashMap<String, (u64, u64, u64)> = std::collections::HashMap::new();
    if let Ok(output) = std::process::Command::new("df").arg("-B1").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 { continue; }
            let mount = parts[5].to_string();
            let total: u64 = parts[1].parse().unwrap_or(0);
            let used: u64 = parts[2].parse().unwrap_or(0);
            let avail: u64 = parts[3].parse().unwrap_or(0);
            usage.insert(mount, (total, used, avail));
        }
    }

    // run df -i for inode
    let mut inode_info: std::collections::HashMap<String, (u64, u64, u64)> = std::collections::HashMap::new();
    if let Ok(output) = std::process::Command::new("df").arg("-i").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 { continue; }
            let mount = parts[5].to_string();
            let total: u64 = parts[1].parse().unwrap_or(0);
            let used: u64 = parts[2].parse().unwrap_or(0);
            let avail: u64 = parts[3].parse().unwrap_or(0);
            inode_info.insert(mount, (total, used, avail));
        }
    }

    let mut result = Vec::new();
    for (mount, device, fs_type) in &mount_info {
        let (total, used, avail) = usage.get(mount).copied().unwrap_or((0, 0, 0));
        let percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
        let (inode_total, inode_used, _inode_avail) = inode_info.get(mount).copied().unwrap_or((0, 0, 0));
        let inode_percent = if inode_total > 0 { (inode_used as f64 / inode_total as f64) * 100.0 } else { 0.0 };
        result.push(DiskDetail {
            mount: mount.clone(),
            device: device.clone(),
            fs_type: fs_type.clone(),
            total,
            used,
            avail,
            percent,
            inode_total,
            inode_used,
            inode_percent,
        });
    }
    result
}

fn parse_net() -> Vec<NetStat> {
    let content = match std::fs::read_to_string("/proc/net/dev") {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut list = Vec::new();
    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 { continue; }
        let name = parts[0].trim_end_matches(':');
        // skip loopback
        if name == "lo" { continue; }
        let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
        let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
        list.push(NetStat { name: name.to_string(), rx_bytes, tx_bytes });
    }
    list
}

fn parse_disk_io() -> DiskIo {
    let content = match std::fs::read_to_string("/proc/diskstats") {
        Ok(c) => c,
        Err(_) => return DiskIo { name: String::new(), read_bytes: 0, write_bytes: 0 },
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 14 { continue; }
        let name = parts[2];
        // skip partitions (sda1, sda2) and loop, ram, nbd
        if name.chars().any(|c| c.is_ascii_digit()) { continue; }
        if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("nbd") { continue; }
        let rd_sectors: u64 = parts[5].parse().unwrap_or(0);
        let wr_sectors: u64 = parts[9].parse().unwrap_or(0);
        return DiskIo {
            name: name.to_string(),
            read_bytes: rd_sectors * 512,
            write_bytes: wr_sectors * 512,
        };
    }
    DiskIo { name: String::new(), read_bytes: 0, write_bytes: 0 }
}

fn parse_top_procs() -> Vec<ProcStat> {
    let uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split('.').next()?.parse::<f64>().ok())
        .unwrap_or(1.0);
    let clk_tck = 100.0;
    let mut procs = Vec::new();
    let dir = match std::fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return procs,
    };
    for entry in dir.flatten() {
        let pid_str = match entry.file_name().to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let stat_path = entry.path().join("stat");
        let content = match std::fs::read_to_string(&stat_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // find comm end: first ')' after '('
        let comm_end = match content.rfind(')') {
            Some(i) => i,
            None => continue,
        };
        let after = &content[comm_end + 2..];
        let fields: Vec<&str> = after.split_whitespace().collect();
        if fields.len() < 17 { continue; }
        let utime: f64 = fields[11].parse().unwrap_or(0.0);
        let stime: f64 = fields[12].parse().unwrap_or(0.0);
        let starttime: f64 = fields[19].parse().unwrap_or(0.0);
        let total_jiffies = uptime * clk_tck - starttime;
        let cpu_pct = if total_jiffies > 0.0 {
            (utime + stime) / total_jiffies * 100.0
        } else {
            0.0
        };
        let comm = &content[content.find('(').unwrap_or(0) + 1..comm_end];
        procs.push(ProcStat {
            pid,
            name: comm.to_string(),
            cpu_percent: cpu_pct,
        });
    }
    procs.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(5);
    procs
}

fn parse_top_mem_procs(mem_total: u64) -> Vec<MemProc> {
    let mut procs = Vec::new();
    let dir = match std::fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return procs,
    };
    for entry in dir.flatten() {
        let pid_str = match entry.file_name().to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let status_path = entry.path().join("status");
        let content = match std::fs::read_to_string(&status_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut name = String::new();
        let mut vmrss = 0u64;
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("Name:") {
                name = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("VmRSS:") {
                vmrss = val.trim().trim_end_matches(" kB").parse().unwrap_or(0) * 1024;
            }
        }
        if vmrss == 0 { continue; }
        let pct = if mem_total > 0 { (vmrss as f64 / mem_total as f64) * 100.0 } else { 0.0 };
        procs.push(MemProc { pid, name, mem_bytes: vmrss, percent: pct });
    }
    procs.sort_by(|a, b| b.mem_bytes.cmp(&a.mem_bytes));
    procs.truncate(5);
    procs
}

pub async fn kill_process(
    headers: HeaderMap,
    Path(pid): Path<u32>,
) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let _ = std::process::Command::new("kill")
        .arg(pid.to_string())
        .output();
    Ok(Json(serde_json::json!({"ok": true})))
}
