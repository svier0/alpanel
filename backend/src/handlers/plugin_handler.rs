use std::collections::HashMap;
use std::path::Path;

use axum::{http::HeaderMap, Json};
use serde::Serialize;

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;

#[derive(Serialize)]
pub struct PluginEntry {
    #[serde(flatten)]
    info: HashMap<String, String>,
}

fn parse_info_ini(path: &Path) -> Option<HashMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Some(map)
}

pub async fn list_plugins(headers: HeaderMap) -> AppResult<Json<Vec<PluginEntry>>> {
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

        let ini_path = path.join("info.ini");
        if !ini_path.exists() {
            continue;
        }

        let Some(info) = parse_info_ini(&ini_path) else { continue };

        if let Some(name) = info.get("name") {
            if name == &dir_name {
                plugins.push(PluginEntry { info });
            }
        }
    }

    Ok(Json(plugins))
}
