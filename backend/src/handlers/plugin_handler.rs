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
