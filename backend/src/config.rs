use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use jwt_simple::prelude::HS256Key;

pub struct AppConfig {
    pub panel_user: String,
    pub panel_password: String,
    pub panel_title: String,
    pub panel_theme: String,
    pub jwt_secret: String,
    pub jwt_key: HS256Key,
}

static GLOBAL_CONFIG: OnceLock<Arc<RwLock<AppConfig>>> = OnceLock::new();

pub fn config() -> &'static Arc<RwLock<AppConfig>> {
    GLOBAL_CONFIG.get().expect("AppConfig not initialized")
}

pub fn init_config(cfg: AppConfig) {
    GLOBAL_CONFIG.set(Arc::new(RwLock::new(cfg))).ok();
}

fn env_path() -> String {
    std::env::var("PANEL_ENV").unwrap_or_else(|_| ".env".to_string())
}

pub fn read_env() -> HashMap<String, String> {
    let content = std::fs::read_to_string(env_path()).unwrap_or_default();
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

pub fn write_env(changes: &HashMap<&str, String>) {
    let path = env_path();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    for (&key, value) in changes {
        let prefix = format!("{}=", key);
        let mut found = false;
        for line in &mut lines {
            let trimmed = line.trim();
            if trimmed.starts_with(&prefix) || trimmed.starts_with(&prefix.to_lowercase()) {
                *line = format!("{}={}", key, value);
                found = true;
                break;
            }
        }
        if !found {
            lines.push(format!("{}={}", key, value));
        }
    }
    std::fs::write(&path, lines.join("\n") + "\n").ok();
}

pub fn restart_panel() {
    let result = std::process::Command::new("alp").arg("13").output();
    if let Ok(output) = result {
        if output.status.success() {
            return;
        }
    }
    let result = std::process::Command::new("rc-service")
        .args(["alpanel", "restart"])
        .output();
    if let Ok(output) = result {
        if output.status.success() {
            return;
        }
    }
    tracing::warn!("failed to restart panel: no alp or rc-service found");
}
