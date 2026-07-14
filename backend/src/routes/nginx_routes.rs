use axum::routing::{get, post};
use axum::Router;

use crate::handlers::nginx_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/nginx/status", get(nginx_handler::get_status))
        .route("/api/nginx/install", post(nginx_handler::install))
        .route("/api/nginx/start", post(nginx_handler::start))
        .route("/api/nginx/stop", post(nginx_handler::stop))
        .route("/api/nginx/restart", post(nginx_handler::restart))
        .route("/api/nginx/reload", post(nginx_handler::reload))
}
