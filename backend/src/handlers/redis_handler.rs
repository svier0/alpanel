use axum::{http::HeaderMap, Json};

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::services::redis_service;

pub async fn get_status(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let installed = redis_service::check_installed();
    let running = redis_service::check_running();
    Ok(Json(serde_json::json!({ "installed": installed, "running": running })))
}

pub async fn install(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = redis_service::install()?;
    Ok(Json(serde_json::json!({ "message": message })))
}
