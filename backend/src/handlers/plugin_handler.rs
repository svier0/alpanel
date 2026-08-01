use std::collections::HashSet;
use std::path::Path;

use axum::{extract::Path as AxumPath, http::HeaderMap, Json};
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
    #[serde(default)]
    pub func: String,
}

pub async fn list_plugins(headers: HeaderMap) -> AppResult<Json<Vec<PluginInfo>>> {
    check_auth(&headers)?;

    let output = tokio::task::spawn_blocking(|| {
        std::process::Command::new("alp")
            .arg("51")
            .output()
    }).await.map_err(|_| crate::errors::AppError::Internal("无法执行 alp 命令".into()))?
    .map_err(|e| crate::errors::AppError::Internal(format!("alp 执行失败: {}", e)))?;

    if !output.status.success() {
        return Ok(Json(Vec::new()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let plugins: Vec<PluginInfo> = serde_json::from_str(stdout.trim())
        .unwrap_or_default();

    Ok(Json(plugins))
}

pub async fn remote_plugins() -> AppResult<Json<Vec<PluginInfo>>> {
    let output = tokio::task::spawn_blocking(|| {
        std::process::Command::new("alp")
            .arg("52")
            .output()
    }).await.map_err(|_| crate::errors::AppError::Internal("无法执行 alp 命令".into()))?
    .map_err(|e| crate::errors::AppError::Internal(format!("alp 执行失败: {}", e)))?;

    if !output.status.success() {
        return Err(crate::errors::AppError::Internal("获取远程插件列表失败".into()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let plugins: Vec<PluginInfo> = serde_json::from_str(stdout.trim())
        .map_err(|_| crate::errors::AppError::Internal("解析插件列表失败".into()))?;

    Ok(Json(plugins))
}

const FIXED_METHODS: [&str; 7] = ["install", "uninstall", "start", "stop", "restart", "reload", "status"];

fn valid_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

fn valid_method(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub async fn action(
    headers: HeaderMap,
    AxumPath((name, method)): AxumPath<(String, String)>,
    body: axum::body::Bytes,
) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;

    if !valid_name(&name) {
        return Err(crate::errors::AppError::BadRequest("非法插件名".into()));
    }
    if !valid_method(&method) {
        return Err(crate::errors::AppError::BadRequest("非法方法名".into()));
    }

    // whitelist: fixed methods + plugin-defined func
    let plugin_dir = Path::new("/www/server/panel/plugin").join(&name);
    let info_path = plugin_dir.join("info.json");
    let extra = std::fs::read_to_string(&info_path)
        .ok()
        .and_then(|s| serde_json::from_str::<PluginInfo>(&s).ok())
        .map(|p| p.func)
        .unwrap_or_default();
    let allowed: HashSet<&str> = FIXED_METHODS.iter().copied().collect();
    let extra_methods: HashSet<&str> = extra.split('|').filter(|s| !s.is_empty()).collect();
    if !allowed.contains(method.as_str()) && !extra_methods.contains(method.as_str()) {
        return Err(crate::errors::AppError::BadRequest(format!("不允许的方法: {}", method)));
    }

    let args = if body.is_empty() {
        String::new()
    } else {
        String::from_utf8_lossy(&body).to_string()
    };

    let sh_path = plugin_dir.join(format!("{}.sh", name));

    let output = match method.as_str() {
        "install" => tokio::task::spawn_blocking(move || {
            std::process::Command::new("alp")
                .args(["53", &name])
                .output()
        }).await.map_err(|_| crate::errors::AppError::Internal("执行失败".into()))?
        .map_err(|e| crate::errors::AppError::Internal(format!("alp 执行失败: {}", e)))?,
        "uninstall" => tokio::task::spawn_blocking(move || {
            std::process::Command::new("alp")
                .args(["54", &name])
                .output()
        }).await.map_err(|_| crate::errors::AppError::Internal("执行失败".into()))?
        .map_err(|e| crate::errors::AppError::Internal(format!("alp 执行失败: {}", e)))?,
        _ => {
            if !sh_path.exists() {
                return Err(crate::errors::AppError::NotFound("插件脚本不存在".into()));
            }
            let sh_path = sh_path.clone();
            let method = method.clone();
            let args = args.clone();
            tokio::task::spawn_blocking(move || {
                let mut cmd = std::process::Command::new("sh");
                cmd.args(["-c", &format!(". '{}' && {}", sh_path.display(), method)]);
                if !args.is_empty() {
                    cmd.env("PLUGIN_ARGS", &args);
                }
                cmd.output()
            }).await.map_err(|_| crate::errors::AppError::Internal("执行失败".into()))?
            .map_err(|_| crate::errors::AppError::Internal("无法执行脚本".into()))?
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(Json(serde_json::json!({
        "code": output.status.code(),
        "stdout": stdout,
        "stderr": stderr,
    })))
}
