use axum::{http::HeaderMap, Json};

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::services::mysql_service;

pub async fn get_status(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let installed = mysql_service::check_installed();
    let running = mysql_service::check_running();
    Ok(Json(serde_json::json!({ "installed": installed, "running": running })))
}

pub async fn install(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = mysql_service::install()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn start(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = mysql_service::start()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn stop(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = mysql_service::stop()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn restart(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = mysql_service::restart()?;
    Ok(Json(serde_json::json!({ "message": message })))
}

pub async fn reload(headers: HeaderMap) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let message = mysql_service::reload()?;
    Ok(Json(serde_json::json!({ "message": message })))
}
