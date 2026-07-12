use axum::http::{header, HeaderMap};
use jwt_simple::algorithms::MACLike;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::errors::AppError;

#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
    pub username: String,
}

pub fn check_auth(headers: &HeaderMap) -> Result<(), AppError> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("unauthorized".to_string()))?;

    let cfg = config::config().read().unwrap();
    cfg.jwt_key
        .verify_token::<JwtClaims>(token, None)
        .map_err(|_| AppError::Unauthorized("unauthorized".to_string()))?;

    Ok(())
}
