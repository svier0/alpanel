use std::collections::HashMap;

use crate::config;
use crate::dto::settings_dto::{SettingsResponse, SettingsUpdate};
use crate::errors::AppResult;
use crate::repositories::user_repository;

fn current_port() -> u16 {
    std::env::var("PANEL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5555)
}

fn current_ghproxy() -> String {
    let path = std::env::var("PANEL_ENV").unwrap_or_else(|_| ".env".to_string());
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| {
            s.lines().find_map(|l| {
                let l = l.trim();
                l.strip_prefix("GHPROXY=").map(|v| v.trim().to_string())
            })
        })
        .unwrap_or_default()
}

pub fn get_settings() -> AppResult<SettingsResponse> {
    let cfg = config::config().read().unwrap();
    let user = user_repository::get_user_by_username(&cfg.panel_user)
        .map(|u| u.username)
        .unwrap_or_else(|| cfg.panel_user.clone());
    Ok(SettingsResponse {
        port: current_port(),
        user,
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
        ghproxy: current_ghproxy(),
    })
}

pub fn update_settings(body: SettingsUpdate) -> (SettingsResponse, bool) {
    let mut changes: HashMap<&str, String> = HashMap::new();
    let mut cfg = config::config().write().unwrap();

    if let Some(port) = body.port {
        changes.insert("PANEL_PORT", port.to_string());
    }
    if let Some(ref title) = body.title {
        if !title.is_empty() {
            cfg.panel_title = title.clone();
            changes.insert("PANEL_TITLE", title.clone());
        }
    }
    if let Some(ref theme) = body.theme {
        if !theme.is_empty() {
            cfg.panel_theme = theme.clone();
            changes.insert("PANEL_THEME", theme.clone());
        }
    }

    let port_changed = changes.contains_key("PANEL_PORT");

    // 账号/密码一律写入 users 表，不再写入 .env
    // .env 仅保留安装时的初始账号密码供查看
    let current_user = user_repository::get_user_by_username(&cfg.panel_user);
    if let Some(user) = current_user {
        if let Some(id) = user.id {
            let new_user = body.user.clone().filter(|u| !u.is_empty());
            let new_pw = body.password.clone().filter(|p| !p.is_empty());
            if new_user.is_some() || new_pw.is_some() {
                user_repository::update_user(id, new_user.clone(), new_pw, None, None);
            }
            if let Some(u) = new_user {
                cfg.panel_user = u;
            }
        }
    }

    if !changes.is_empty() {
        config::write_env(&changes);
    }

    let resp = SettingsResponse {
        port: body.port.unwrap_or_else(current_port),
        user: cfg.panel_user.clone(),
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
        ghproxy: current_ghproxy(),
    };

    (resp, port_changed)
}
