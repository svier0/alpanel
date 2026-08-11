use crate::dto::site_dto::CreateDomainInline;
use crate::errors::{AppError, AppResult};

const VHOST_TEMPLATE_DIR: &str = "/www/server/panel/vhost/template/nginx";
const VHOST_NGINX_DIR: &str = "/www/server/panel/vhost/nginx";
const VHOST_REWRITE_DIR: &str = "/www/server/panel/vhost/rewrite";
const STOP_PATH: &str = "/www/server/stop";

fn build_listen_ports(domains: &[CreateDomainInline]) -> String {
    let mut ports: Vec<i64> = Vec::new();
    for d in domains {
        let port = d.port.unwrap_or(80);
        if !ports.contains(&port) {
            ports.push(port);
        }
    }
    if ports.is_empty() {
        ports.push(80);
    }
    ports
        .into_iter()
        .map(|p| format!("listen {};", p))
        .collect::<Vec<_>>()
        .join("\n    ")
}

fn build_domains(domains: &[CreateDomainInline]) -> String {
    domains
        .iter()
        .map(|d| d.name.trim())
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn php_version_tag(phpversion: Option<&str>) -> String {
    let v = phpversion.unwrap_or("").trim();
    if v.is_empty() {
        return "php-00".to_string();
    }
    format!("php-{}", v.replace('.', ""))
}

pub fn generate_site_vhost(
    site_name: &str,
    site_path: &str,
    site_run_path: &str,
    status: Option<&str>,
    domains: &[CreateDomainInline],
    phpversion: Option<&str>,
) -> AppResult<()> {
    let template_path = format!("{}/site.conf", VHOST_TEMPLATE_DIR);
    let template = std::fs::read_to_string(&template_path)
        .map_err(|e| AppError::Internal(format!("读取站点模板失败: {}", e)))?;

    let listen_ports = build_listen_ports(domains);
    let domains_line = build_domains(domains);
    let path = if status == Some("0") {
        STOP_PATH.to_string()
    } else {
        format!(
            "{}{}",
            site_path.trim(),
            site_run_path.trim()
        )
    };

    let content = template
        .replace("{$listen_ports}", &listen_ports)
        .replace("{$domains}", &domains_line)
        .replace("{$site_path}", &path)
        .replace("{$site_name}", site_name)
        .replace("{$php_version}", &php_version_tag(phpversion));

    let nginx_dir = std::path::Path::new(VHOST_NGINX_DIR);
    std::fs::create_dir_all(nginx_dir).map_err(|e| {
        AppError::Internal(format!("创建站点配置目录失败: {}", e))
    })?;
    std::fs::write(
        nginx_dir.join(format!("{}.conf", site_name)),
        content,
    )
    .map_err(|e| AppError::Internal(format!("写入站点配置失败: {}", e)))?;

    let rewrite_dir = std::path::Path::new(VHOST_REWRITE_DIR);
    std::fs::create_dir_all(rewrite_dir).map_err(|e| {
        AppError::Internal(format!("创建伪静态目录失败: {}", e))
    })?;
    let rewrite_path = rewrite_dir.join(format!("{}.conf", site_name));
    if !rewrite_path.exists() {
        std::fs::write(&rewrite_path, "").map_err(|e| {
            AppError::Internal(format!("创建伪静态文件失败: {}", e))
        })?;
    }

    Ok(())
}

fn vhost_conf_path(project_type: &str, site_name: &str) -> String {
    match project_type {
        "Proxy" => format!("{}/proxy_{}.conf", VHOST_NGINX_DIR, site_name),
        "Other" => format!("{}/other_{}.conf", VHOST_NGINX_DIR, site_name),
        _ => format!("{}/{}.conf", VHOST_NGINX_DIR, site_name),
    }
}

fn read_tail(path: &std::path::Path, max_bytes: u64) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return String::new(),
    };
    let meta = match f.metadata() {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    if meta.len() <= max_bytes {
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        return s;
    }
    let _ = f.seek(SeekFrom::End(-(max_bytes as i64)));
    let mut buf = vec![0u8; max_bytes as usize];
    let n = f.read(&mut buf).unwrap_or(0);
    let mut s = String::from_utf8_lossy(&buf[..n]).into_owned();
    if let Some(pos) = s.find('\n') {
        s = s[pos + 1..].to_string();
    }
    s
}

pub fn read_site_rewrite(site_name: &str) -> String {
    read_tail(
        &std::path::Path::new(VHOST_REWRITE_DIR).join(format!("{}.conf", site_name)),
        512 * 1024,
    )
}

pub fn read_site_config(site: &crate::models::site::Site) -> String {
    let project_type = site.project_type.as_deref().unwrap_or("PHP");
    read_tail(
        &std::path::Path::new(&vhost_conf_path(project_type, &site.name)),
        512 * 1024,
    )
}

pub fn read_site_log(site_name: &str, log_type: &str) -> String {
    let file_name = if log_type == "error" {
        format!("{}.error.log", site_name)
    } else {
        format!("{}.log", site_name)
    };
    read_tail(&std::path::Path::new("/www/wwwlogs").join(file_name), 512 * 1024)
}

pub fn remove_site_vhost(site: &crate::models::site::Site) -> AppResult<()> {
    let project_type = site.project_type.as_deref().unwrap_or("PHP");
    let conf_path = vhost_conf_path(project_type, &site.name);
    let rewrite_path = format!("{}/{}.conf", VHOST_REWRITE_DIR, site.name);

    for p in [&conf_path, &rewrite_path] {
        let path = std::path::Path::new(p);
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| AppError::Internal(format!("删除站点配置文件失败: {}", e)))?;
        }
    }
    Ok(())
}

fn set_php_version_in_conf(content: &str, new_tag: &str) -> String {
    content
        .lines()
        .map(|l| {
            let trimmed = l.trim();
            if trimmed.starts_with("include php-") && trimmed.ends_with(".conf;") {
                let indent = &l[..l.len() - trimmed.len()];
                format!("{}include {}.conf;", indent, new_tag)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn set_site_phpversion(site: &crate::models::site::Site, new_version: &str) -> AppResult<()> {
    let project_type = site.project_type.as_deref().unwrap_or("PHP");
    let conf_path = vhost_conf_path(project_type, &site.name);
    let content = std::fs::read_to_string(&conf_path)
        .map_err(|e| AppError::Internal(format!("读取站点配置文件失败: {}", e)))?;
    let tag = php_version_tag(Some(new_version));
    let new_content = set_php_version_in_conf(&content, &tag);
    if new_content != content {
        std::fs::write(&conf_path, new_content)
            .map_err(|e| AppError::Internal(format!("写入站点配置文件失败: {}", e)))?;
        reload_nginx()?;
    }
    Ok(())
}

pub fn reload_nginx() -> AppResult<()> {
    let pid_path = "/www/server/nginx/run/nginx.pid";
    if !std::path::Path::new(pid_path).exists() {
        return Ok(());
    }
    let pid = std::fs::read_to_string(pid_path)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok());
    if let Some(pid) = pid {
        let out = std::process::Command::new("kill")
            .args(["-HUP", &pid.to_string()])
            .output()
            .map_err(|e| AppError::Internal(format!("reload nginx 失败: {}", e)))?;
        if !out.status.success() {
            let msg = String::from_utf8_lossy(&out.stderr);
            return Err(AppError::Internal(format!(
                "reload nginx 失败: {}",
                msg.trim()
            )));
        }
    }
    Ok(())
}

pub fn set_site_status(site: &crate::models::site::Site) -> AppResult<()> {    let project_type = site.project_type.as_deref().unwrap_or("PHP");
    let conf_path = vhost_conf_path(project_type, &site.name);
    let content = std::fs::read_to_string(&conf_path)
        .map_err(|e| AppError::Internal(format!("读取站点配置文件失败: {}", e)))?;
    let target = if site.status.as_deref() == Some("0") {
        STOP_PATH.to_string()
    } else {
        format!(
            "{}{}",
            site.path.trim(),
            site.php_run_path.as_deref().unwrap_or("").trim()
        )
    };
    let new_content = content
        .lines()
        .map(|l| {
            let trimmed = l.trim_start();
            if trimmed.starts_with("root ") && trimmed.ends_with(';') {
                let indent = &l[..l.len() - trimmed.len()];
                format!("{}root {};", indent, target)
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&conf_path, new_content)
        .map_err(|e| AppError::Internal(format!("写入站点配置文件失败: {}", e)))?;
    reload_nginx()?;
    Ok(())
}