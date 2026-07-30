use std::path::Path;

use axum::{http::HeaderMap, Json};
use serde::{Deserialize, Serialize};

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginInfo {
    pub title: String,
    pub name: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub versions: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub home: String,
}

pub async fn list_plugins(headers: HeaderMap) -> AppResult<Json<Vec<PluginInfo>>> {
    check_auth(&headers)?;

    let plugin_dir = Path::new("/www/server/panel/plugin");
    let mut plugins = Vec::new();

    let entries = match std::fs::read_dir(plugin_dir) {
        Ok(e) => e,
        Err(_) => return Ok(Json(plugins)),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let json_path = path.join("info.json");
        if !json_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&json_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let info: PluginInfo = match serde_json::from_str(&content) {
            Ok(i) => i,
            Err(_) => continue,
        };

        if info.name == dir_name {
            plugins.push(info);
        }
    }

    Ok(Json(plugins))
}

pub async fn remote_plugins() -> AppResult<Json<Vec<PluginInfo>>> {
    let tmp = std::env::temp_dir().join("alpanel_plugins.json");
    let url = "https://raw.githubusercontent.com/svier0/alpanel-plugins/master/index.json";

    let status = std::process::Command::new("wget")
        .args(["-q", "-O", tmp.to_str().unwrap_or("/tmp/alpanel_plugins.json"), url])
        .status()
        .map_err(|_| crate::errors::AppError::Internal("无法执行 wget".into()))?;

    if !status.success() {
        return Err(crate::errors::AppError::Internal("无法获取远程插件列表".into()));
    }

    let content = std::fs::read_to_string(&tmp)
        .map_err(|_| crate::errors::AppError::Internal("读取插件列表失败".into()))?;

    std::fs::remove_file(&tmp).ok();

    let plugins: Vec<PluginInfo> = serde_json::from_str(&content)
        .map_err(|_| crate::errors::AppError::Internal("解析插件列表失败".into()))?;

    Ok(Json(plugins))
}
