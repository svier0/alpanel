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

#[derive(Deserialize)]
pub struct PluginAction {
    pub name: String,
}

const GH_RAW: &str = "https://raw.githubusercontent.com/svier0/alpanel-plugins/master";
const ICON_DIR: &str = "/www/server/panel/dist/static/img/plugins/icon";

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

fn wget(url: &str, dest: &Path) -> Result<(), crate::errors::AppError> {
    let status = std::process::Command::new("wget")
        .args(["-q", "-O", dest.to_str().unwrap_or(""), url])
        .status()
        .map_err(|_| crate::errors::AppError::Internal("无法执行 wget".into()))?;
    if !status.success() {
        return Err(crate::errors::AppError::Internal("下载失败".into()));
    }
    Ok(())
}

pub async fn install_plugin(headers: HeaderMap, Json(body): Json<PluginAction>) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;

    let plugin_dir = Path::new("/www/server/panel/plugin").join(&body.name);
    if plugin_dir.exists() {
        return Err(crate::errors::AppError::Internal("插件已安装".into()));
    }
    std::fs::create_dir_all(&plugin_dir)
        .map_err(|_| crate::errors::AppError::Internal("创建插件目录失败".into()))?;

    let info_url = format!("{}/plugins/{}/info.json", GH_RAW, body.name);
    let info_path = plugin_dir.join("info.json");
    wget(&info_url, &info_path)?;

    let icon_url = format!("{}/plugins/{}/icon.png", GH_RAW, body.name);
    let icon_dir = Path::new(ICON_DIR);
    std::fs::create_dir_all(icon_dir).ok();
    let icon_path = icon_dir.join(format!("{}.png", body.name));
    if wget(&icon_url, &icon_path).is_err() {
        // icon optional
    }

    Ok(Json(serde_json::json!({ "message": "安装成功" })))
}

pub async fn uninstall_plugin(headers: HeaderMap, Json(body): Json<PluginAction>) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;

    let plugin_dir = Path::new("/www/server/panel/plugin").join(&body.name);
    if !plugin_dir.exists() {
        return Err(crate::errors::AppError::Internal("插件未安装".into()));
    }
    std::fs::remove_dir_all(&plugin_dir)
        .map_err(|_| crate::errors::AppError::Internal("卸载失败".into()))?;

    let icon_path = Path::new(ICON_DIR).join(format!("{}.png", body.name));
    std::fs::remove_file(icon_path).ok();

    Ok(Json(serde_json::json!({ "message": "卸载成功" })))
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
