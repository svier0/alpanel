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
        site_path.trim().to_string()
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

pub fn set_site_status(site: &crate::models::site::Site) -> AppResult<()> {
    let project_type = site.project_type.as_deref().unwrap_or("PHP");
    let conf_path = vhost_conf_path(project_type, &site.name);
    let content = std::fs::read_to_string(&conf_path)
        .map_err(|e| AppError::Internal(format!("读取站点配置文件失败: {}", e)))?;
    let target = if site.status.as_deref() == Some("0") {
        STOP_PATH.to_string()
    } else {
        site.path.trim().to_string()
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
    Ok(())
}