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

    dotenvy::dotenv().ok();

    let port: u16 = std::env::var("PANEL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5555);

    let env = config::read_env();

    let panel_user = env.get("PANEL_USER").cloned().unwrap_or_else(|| "admin".to_string());
    let panel_password = env.get("PANEL_PASSWORD").cloned().unwrap_or_else(|| "admin123".to_string());
    let panel_title = env.get("PANEL_TITLE").cloned().unwrap_or_else(|| "Alpanel".to_string());
    let panel_theme = env.get("PANEL_THEME").cloned().unwrap_or_else(|| "auto".to_string());

    let jwt_secret = env.get("JWT_SECRET").cloned().unwrap_or_else(|| "alpanel_hs256_secret_2026_32bytes!".to_string());
    let jwt_key = jwt_simple::prelude::HS256Key::from_bytes(jwt_secret.as_bytes());

    config::init_config(config::AppConfig {
        panel_user,
        panel_password,
        panel_title,
        panel_theme,
        jwt_secret,
        jwt_key,
    });

    db::pool::init_db();

    let app = routes::routes().fallback(frontend::serve_frontend);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Alpanel listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
