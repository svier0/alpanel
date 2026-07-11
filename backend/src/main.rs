use std::net::SocketAddr;
use axum::{Router, response::{IntoResponse, Response}, http::{StatusCode, header, Uri}};
use rust_embed::Embed;
use tracing::info;

#[derive(Embed)]
#[folder = "../frontend/dist"]
struct Frontend;

async fn serve_frontend(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return serve_index().await;
    }
    if let Some(file) = Frontend::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(axum::body::Body::from(file.data))
            .unwrap()
            .into_response();
    }
    serve_index().await
}

async fn serve_index() -> Response {
    let file = Frontend::get("index.html").unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(axum::body::Body::from(file.data))
        .unwrap()
        .into_response()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let port: u16 = std::env::var("PANEL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5555);

    let app = Router::new().fallback(serve_frontend);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Alpanel listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
