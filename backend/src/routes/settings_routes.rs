use axum::routing::get;
use axum::Router;

use crate::handlers::settings_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/settings", get(settings_handler::get_settings).put(settings_handler::update_settings))
}
