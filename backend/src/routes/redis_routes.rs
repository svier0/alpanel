use axum::routing::{get, post};
use axum::Router;

use crate::handlers::redis_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/redis/status", get(redis_handler::get_status))
        .route("/api/redis/install", post(redis_handler::install))
        .route("/api/redis/start", post(redis_handler::start))
        .route("/api/redis/stop", post(redis_handler::stop))
        .route("/api/redis/restart", post(redis_handler::restart))
        .route("/api/redis/reload", post(redis_handler::reload))
}
