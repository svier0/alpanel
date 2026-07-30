use axum::routing::{get, post};
use axum::Router;

use crate::handlers::plugin_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/plugins", get(plugin_handler::list_plugins))
        .route("/api/plugins/remote", get(plugin_handler::remote_plugins))
        .route("/api/plugins/install", post(plugin_handler::install_plugin))
        .route("/api/plugins/uninstall", post(plugin_handler::uninstall_plugin))
}
