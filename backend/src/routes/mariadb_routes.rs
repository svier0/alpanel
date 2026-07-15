use axum::routing::{get, post};
use axum::Router;

use crate::handlers::mariadb_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/mariadb/status", get(mariadb_handler::get_status))
        .route("/api/mariadb/install", post(mariadb_handler::install))
        .route("/api/mariadb/start", post(mariadb_handler::start))
        .route("/api/mariadb/stop", post(mariadb_handler::stop))
        .route("/api/mariadb/restart", post(mariadb_handler::restart))
        .route("/api/mariadb/reload", post(mariadb_handler::reload))
}
