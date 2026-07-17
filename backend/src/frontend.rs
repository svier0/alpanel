use std::path::PathBuf;

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};

fn dist_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    let bin_dir = exe.parent().unwrap_or_else(|| std::path::Path::new("."));
    bin_dir.join("dist")
}

async fn serve_index() -> Response {
    let index = dist_dir().join("index.html");
    match tokio::fs::read(&index).await {
        Ok(data) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(data))
            .unwrap()
            .into_response(),
        Err(_) => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("前端资源未找到，请确保 dist 目录位于二进制同目录下"))
            .unwrap()
            .into_response(),
    }
}

pub async fn serve_frontend(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return serve_index().await;
    }
    let file_path = dist_dir().join(path);
    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(data))
                .unwrap()
                .into_response()
        }
        Err(_) => serve_index().await,
    }
}
