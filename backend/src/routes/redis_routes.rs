use axum::routing::{get, post};
use axum::Router;

use crate::handlers::redis_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/redis/status", get(redis_handler::get_status))
        .route("/api/redis/install", post(redis_handler::install))
}
