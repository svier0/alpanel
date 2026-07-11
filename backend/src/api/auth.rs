use axum::{
    http::{header, HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct JwtClaims {
    username: String,
}

pub fn check_auth(headers: &HeaderMap) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "unauthorized"})),
            )
        })?;

    let cfg = config::config().read().unwrap();
    cfg.jwt_key
        .verify_token::<JwtClaims>(token, None)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "unauthorized"})),
            )
        })?;

    Ok(())
}

async fn login(
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let cfg = config::config().read().unwrap();
    if req.username != cfg.panel_user || req.password != cfg.panel_password {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid credentials"})),
        ));
    }
    let claims =
        Claims::with_custom_claims(JwtClaims { username: req.username }, Duration::from_hours(24));
    let token = cfg.jwt_key.authenticate(claims).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "token creation failed"})),
        )
    })?;
    Ok(Json(LoginResponse { token }))
}

async fn verify(
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    check_auth(&headers)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub(super) fn routes() -> Router<()> {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/verify", get(verify))
}
