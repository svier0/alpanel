use axum::{http::HeaderMap, Json};

use crate::dto::auth_dto::{LoginRequest, LoginResponse};
use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::services::auth_service;

pub async fn login(
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
        });
    let token = auth_service::login(req, client_ip)?;
    Ok(Json(LoginResponse { token }))
}

pub async fn verify(
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
