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

    if os_pretty.is_empty() {
        os_pretty = "Unknown".to_string();
    }

    Ok(Json(SystemInfo {
        os_id,
        os_name,
        os_version,
        os_pretty,
        os_arch,
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
