use axum::routing::{get, post};
use axum::Router;

use crate::handlers::file_handler;

pub fn routes() -> Router<()> {
    Router::new()
        .route("/api/files/list", get(file_handler::list))
        .route("/api/files/read", get(file_handler::read))
        .route("/api/files/write", post(file_handler::write))
        .route("/api/files/create", post(file_handler::create))
        .route("/api/files/delete", post(file_handler::delete))
        .route("/api/files/rename", post(file_handler::rename))
        .route("/api/files/dirsize", get(file_handler::dir_size))
        .route("/api/files/copy", post(file_handler::copy))
        .route("/api/files/download", post(file_handler::download))
        .route("/api/files/stream", get(file_handler::stream_download))
}
