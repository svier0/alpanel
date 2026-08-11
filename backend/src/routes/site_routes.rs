use axum::routing::{get, put};
use axum::Router;

use crate::handlers::site_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/sites", get(site_handler::list_sites).post(site_handler::create_site))
        .route("/api/sites/{id}", get(site_handler::get_site).put(site_handler::update_site).delete(site_handler::delete_site))
        .route("/api/sites/{id}/files", get(site_handler::get_site_files))
        .route("/api/sites/{id}/logs", get(site_handler::get_site_log))
        .route("/api/sites/types", get(site_handler::get_project_types))
}