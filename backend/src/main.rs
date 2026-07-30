use std::net::SocketAddr;

use tracing::info;

mod config;
mod db;
mod dto;
mod errors;
mod frontend;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(_) => {}
    }

    let port_raw = std::env::var("PANEL_PORT").unwrap_or_else(|_| {
        "8888".to_string()
    });
    let port: u16 = port_raw.parse().unwrap_or(8888);

    let panel_user = std::env::var("PANEL_USER").unwrap_or_else(|_| "admin".to_string());
    let panel_password = std::env::var("PANEL_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
    let panel_title = std::env::var("PANEL_TITLE").unwrap_or_else(|_| "Alpanel".to_string());
    let panel_theme = std::env::var("PANEL_THEME").unwrap_or_else(|_| "auto".to_string());

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "alpanel_hs256_secret_2026_32bytes!".to_string());
    let jwt_key = jwt_simple::prelude::HS256Key::from_bytes(jwt_secret.as_bytes());

    config::init_config(config::AppConfig {
        panel_user: panel_user.clone(),
        panel_title,
        panel_theme,
        jwt_secret,
        jwt_key,
    });

    db::pool::init_db();

    // 首次初始化：用 .env 中的初始账号密码作为 users 表的初始记录（仅当表为空时）
    // .env 仅作为安装后查看初始账号密码的用途，用户改密后 .env 不再生效
    crate::repositories::user_repository::init_user(&panel_user, &panel_password);

    let app = routes::routes().fallback(frontend::serve_frontend);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
