use std::net::SocketAddr;
use std::sync::{Arc, RwLock, OnceLock};
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use axum::{
    Router, response::{IntoResponse, Response}, http::{StatusCode, header, Uri, HeaderMap},
    routing::{get, post}, Json,
};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use jwt_simple::prelude::*;
use tracing::info;

#[derive(Embed)]
#[folder = "../frontend/dist"]
struct Frontend;

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct JwtClaims {
    username: String,
}

#[derive(Serialize, Clone)]
struct SettingsResponse {
    port: u16,
    user: String,
    title: String,
    theme: String,
}

#[derive(Deserialize)]
struct SettingsUpdate {
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    title: Option<String>,
    theme: Option<String>,
}

struct AppConfig {
    panel_user: String,
    panel_password: String,
    panel_title: String,
    panel_theme: String,
    jwt_secret: String,
    jwt_key: HS256Key,
}

static GLOBAL_CONFIG: OnceLock<Arc<RwLock<AppConfig>>> = OnceLock::new();

fn config() -> &'static Arc<RwLock<AppConfig>> {
    GLOBAL_CONFIG.get().expect("AppConfig not initialized")
}

fn env_path() -> String {
    std::env::var("PANEL_ENV").unwrap_or_else(|_| ".env".to_string())
}

fn read_env() -> HashMap<String, String> {
    let content = std::fs::read_to_string(env_path()).unwrap_or_default();
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

fn write_env(changes: &HashMap<&str, String>) {
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

fn check_auth(headers: &HeaderMap) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))))?;

    let cfg = config().read().unwrap();
    cfg.jwt_key.verify_token::<JwtClaims>(token, None)
        .map_err(|_| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "unauthorized"}))))?;

    Ok(())
}

async fn serve_frontend(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return serve_index().await;
    }
    if let Some(file) = Frontend::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(axum::body::Body::from(file.data))
            .unwrap()
            .into_response();
    }
    serve_index().await
}

async fn serve_index() -> Response {
    let file = Frontend::get("index.html").unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(axum::body::Body::from(file.data))
        .unwrap()
        .into_response()
}

async fn login(
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let cfg = config().read().unwrap();
    if req.username != cfg.panel_user || req.password != cfg.panel_password {
        return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "invalid credentials"}))));
    }
    let claims = Claims::with_custom_claims(JwtClaims { username: req.username }, Duration::from_hours(24));
    let token = cfg.jwt_key.authenticate(claims).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "token creation failed"})))
    })?;
    Ok(Json(LoginResponse { token }))
}

async fn verify(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&headers)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn get_settings(
    headers: HeaderMap,
) -> Result<Json<SettingsResponse>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&headers)?;
    let cfg = config().read().unwrap();
    Ok(Json(SettingsResponse {
        port: std::env::var("PANEL_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(5555),
        user: cfg.panel_user.clone(),
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
    }))
}

async fn update_settings(
    headers: HeaderMap,
    Json(body): Json<SettingsUpdate>,
) -> Result<Json<SettingsResponse>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&headers)?;

    let mut changes = HashMap::new();
    let mut cfg = config().write().unwrap();

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
            cfg.jwt_key = HS256Key::from_bytes(cfg.jwt_secret.as_bytes());
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
        write_env(&changes);
    }

    if port_changed {
        let resp = Json(SettingsResponse {
            port: body.port.unwrap_or_else(|| std::env::var("PANEL_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(5555)),
            user: cfg.panel_user.clone(),
            title: cfg.panel_title.clone(),
            theme: cfg.panel_theme.clone(),
        });
        tokio::spawn(async move {
            tokio::time::sleep(StdDuration::from_millis(500)).await;
            restart_panel();
        });
        return Ok(resp);
    }

    Ok(Json(SettingsResponse {
        port: std::env::var("PANEL_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(5555),
        user: cfg.panel_user.clone(),
        title: cfg.panel_title.clone(),
        theme: cfg.panel_theme.clone(),
    }))
}

fn restart_panel() {
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let port: u16 = std::env::var("PANEL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5555);

    let env = read_env();

    let panel_user = env.get("PANEL_USER").cloned().unwrap_or_else(|| "admin".to_string());
    let panel_password = env.get("PANEL_PASSWORD").cloned().unwrap_or_else(|| "admin123".to_string());
    let panel_title = env.get("PANEL_TITLE").cloned().unwrap_or_else(|| "Alpanel".to_string());
    let panel_theme = env.get("PANEL_THEME").cloned().unwrap_or_else(|| "auto".to_string());

    let jwt_secret = env.get("JWT_SECRET").cloned().unwrap_or_else(|| "alpanel_hs256_secret_2026_32bytes!".to_string());
    let jwt_key = HS256Key::from_bytes(jwt_secret.as_bytes());

    GLOBAL_CONFIG.set(Arc::new(RwLock::new(AppConfig {
        panel_user,
        panel_password,
        panel_title,
        panel_theme,
        jwt_secret,
        jwt_key,
    }))).ok();

    let app = Router::new()
        .route("/api/login", post(login))
        .route("/api/verify", get(verify))
        .route("/api/settings", get(get_settings).put(update_settings))
        .fallback(serve_frontend);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Alpanel listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
