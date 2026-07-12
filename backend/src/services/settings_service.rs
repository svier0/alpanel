use std::collections::HashMap;

use crate::config;
use crate::dto::settings_dto::{SettingsResponse, SettingsUpdate};
use crate::errors::AppResult;

fn current_port() -> u16 {
    std::env::var("PANEL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5555)
}

pub fn get_settings() -> AppResult<SettingsResponse> {
    let cfg = config::config().read().unwrap();
    Ok(SettingsResponse {
        port: current_port(),
        user: cfg.panel_user.clone(),
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
    })
}

pub fn update_settings(body: SettingsUpdate) -> (SettingsResponse, bool) {
    let mut changes: HashMap<&str, String> = HashMap::new();
    let mut cfg = config::config().write().unwrap();

    if let Some(port) = body.port {
        changes.insert("PANEL_PORT", port.to_string());
    }
    if let Some(ref user) = body.user {
        if !user.is_empty() {
            cfg.panel_user = user.clone();
            changes.insert("PANEL_USER", user.clone());
        }
    }
    if let Some(ref password) = body.password {
        if !password.is_empty() {
            cfg.panel_password = password.clone();
            cfg.jwt_key = jwt_simple::prelude::HS256Key::from_bytes(cfg.jwt_secret.as_bytes());
            changes.insert("PANEL_PASSWORD", password.clone());
        }
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

    if !changes.is_empty() {
        config::write_env(&changes);
    }

    let resp = SettingsResponse {
        port: body.port.unwrap_or_else(current_port),
        user: cfg.panel_user.clone(),
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
    };

    (resp, port_changed)
}
