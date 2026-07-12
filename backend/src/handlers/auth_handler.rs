use axum::{http::HeaderMap, Json};

use crate::dto::auth_dto::{LoginRequest, LoginResponse};
use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::services::auth_service;

pub async fn login(
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let token = auth_service::login(req)?;
    Ok(Json(LoginResponse { token }))
}

pub async fn verify(
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
