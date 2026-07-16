use axum::routing::{get, post};
use axum::Router;

use crate::handlers::mysql_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/mysql/status", get(mysql_handler::get_status))
        .route("/api/mysql/install", post(mysql_handler::install))
        .route("/api/mysql/start", post(mysql_handler::start))
        .route("/api/mysql/stop", post(mysql_handler::stop))
        .route("/api/mysql/restart", post(mysql_handler::restart))
        .route("/api/mysql/reload", post(mysql_handler::reload))
        .route("/api/mysql/change_root_pw", post(mysql_handler::change_root_pw))
        .route("/api/mysql/root_pw", get(mysql_handler::get_root_pw))
}
