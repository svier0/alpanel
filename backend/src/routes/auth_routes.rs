use axum::routing::{get, post};
use axum::Router;

use crate::handlers::auth_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/login", post(auth_handler::login))
        .route("/api/verify", get(auth_handler::verify))
}
