use jwt_simple::prelude::*;

use crate::config;
use crate::dto::auth_dto::LoginRequest;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::JwtClaims;

pub fn login(req: LoginRequest) -> AppResult<String> {
    let cfg = config::config().read().unwrap();
    if req.username != cfg.panel_user || req.password != cfg.panel_password {
        return Err(AppError::Unauthorized("invalid credentials".to_string()));
    }

    let claims =
        Claims::with_custom_claims(JwtClaims { username: req.username }, Duration::from_hours(24));
    let token = cfg
        .jwt_key
        .authenticate(claims)
        .map_err(|_| AppError::Internal("token creation failed".to_string()))?;

    Ok(token)
}
