use axum::routing::{get, put};
use axum::Router;

use crate::handlers::site_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/sites", get(site_handler::list_sites).post(site_handler::create_site))
        .route("/api/sites/{id}", put(site_handler::update_site).delete(site_handler::delete_site))
        .route("/api/sites/types", get(site_handler::get_project_types))
}