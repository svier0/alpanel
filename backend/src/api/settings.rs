use std::collections::HashMap;
use std::time::Duration as StdDuration;

use axum::{http::HeaderMap, routing, Json, Router};
use serde::{Deserialize, Serialize};

use crate::api::auth::check_auth;
use crate::config;

#[derive(Serialize, Clone)]
pub struct SettingsResponse {
    port: u16,
    user: String,
    title: String,
    theme: String,
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    title: Option<String>,
    theme: Option<String>,
}

fn current_port() -> u16 {
    std::env::var("PANEL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5555)
}

async fn get_settings(
    headers: HeaderMap,
) -> Result<Json<SettingsResponse>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    check_auth(&headers)?;
    let cfg = config::config().read().unwrap();
    Ok(Json(SettingsResponse {
        port: current_port(),
        user: cfg.panel_user.clone(),
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
    }))
}

async fn update_settings(
    headers: HeaderMap,
    Json(body): Json<SettingsUpdate>,
) -> Result<Json<SettingsResponse>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    check_auth(&headers)?;

    let mut changes = HashMap::new();
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

    if port_changed {
        let resp = Json(SettingsResponse {
            port: body.port.unwrap_or_else(current_port),
            user: cfg.panel_user.clone(),
            title: cfg.panel_title.clone(),
            theme: cfg.panel_theme.clone(),
        });
        tokio::spawn(async move {
            tokio::time::sleep(StdDuration::from_millis(500)).await;
            config::restart_panel();
        });
        return Ok(resp);
    }

    Ok(Json(SettingsResponse {
        port: current_port(),
        user: cfg.panel_user.clone(),
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
    }))
}

pub(super) fn routes() -> Router<()> {
    Router::new()
        .route("/api/settings", routing::get(get_settings).put(update_settings))
}
