use axum::{http::HeaderMap, Json};

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::services::redis_service;

pub async fn get_status(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let installed = redis_service::check_installed();
    let running = redis_service::check_running();
    let version = redis_service::get_version();
    Ok(Json(serde_json::json!({ "installed": installed, "running": running, "version": version })))
}

pub async fn install(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = redis_service::install()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn start(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = redis_service::start()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn stop(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = redis_service::stop()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn restart(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = redis_service::restart()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn reload(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = redis_service::reload()?;
    Ok(Json(serde_json::json!({ "message": message })))
}
