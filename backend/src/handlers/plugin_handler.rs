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
            .arg("55")
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
    let tmp = std::env::temp_dir().join("alpanel_plugins.json");
    let url = "https://raw.githubusercontent.com/svier0/alpanel-plugins/master/index.json";

    let tmp_str = tmp.to_str().unwrap_or("/tmp/alpanel_plugins.json").to_string();

    let result = tokio::task::spawn_blocking(move || {
        let out = std::process::Command::new("wget")
            .args(["-q", "--timeout=10", "-O", &tmp_str, &url])
            .output()?;
        Ok::<_, std::io::Error>(out.status.success())
    }).await.map_err(|_| crate::errors::AppError::Internal("任务执行失败".into()))?;

    match result {
        Ok(true) => {}
        _ => return Err(crate::errors::AppError::Internal("无法获取远程插件列表".into())),
    }

    let content = std::fs::read_to_string(&tmp)
        .map_err(|_| crate::errors::AppError::Internal("读取插件列表失败".into()))?;

    std::fs::remove_file(&tmp).ok();

    let plugins: Vec<PluginInfo> = serde_json::from_str(&content)
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

    let sh_path = plugin_dir.join(format!("{}.sh", name));
    if !sh_path.exists() {
        return Err(crate::errors::AppError::NotFound("插件脚本不存在".into()));
    }

    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new("sh")
            .args(["-c", &format!(". '{}' && {}", sh_path.display(), method)])
            .output()
    }).await.map_err(|_| crate::errors::AppError::Internal("执行失败".into()))?
    .map_err(|_| crate::errors::AppError::Internal("无法执行脚本".into()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(Json(serde_json::json!({
        "code": output.status.code(),
        "stdout": stdout,
        "stderr": stderr,
    })))
}
