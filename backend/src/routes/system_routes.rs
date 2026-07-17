use axum::routing::get;
use axum::Router;

use crate::handlers::system_handler;

pub fn routes() -> Router<()> {
    Router::new().route("/api/system/users", get(system_handler::list_users))
}