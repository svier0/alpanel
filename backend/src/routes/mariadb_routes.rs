use axum::routing::{get, post};
use axum::Router;

use crate::handlers::mariadb_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/mariadb/status", get(mariadb_handler::get_status))
        .route("/api/mariadb/install", post(mariadb_handler::install))
}
