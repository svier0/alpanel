use axum::routing::get;
use axum::Router;

use crate::handlers::plugin_handler;

pub fn routes() -> Router<()> {
    Router::new().route("/api/plugins", get(plugin_handler::list_plugins))
}
