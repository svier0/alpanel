use axum::{Json, Router};
use axum::routing::get;
use serde::Serialize;

#[derive(Serialize)]
struct PluginInfo {
    version: &'static str,
}

pub fn routes() -> Router<()> {
    Router::new().route("/api/plugins", get(list_plugins))
}

async fn list_plugins() -> Json<Vec<PluginInfo>> {
    Json(vec![])
}
