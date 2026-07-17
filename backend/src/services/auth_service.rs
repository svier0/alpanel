use jwt_simple::prelude::*;

use crate::config;
use crate::dto::auth_dto::LoginRequest;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::JwtClaims;
use crate::repositories::user_repository;

pub fn login(req: LoginRequest, client_ip: Option<String>) -> AppResult<String> {
    let user = user_repository::get_user_by_username(&req.username)
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".to_string()))?;

    if !user_repository::verify_password(&user, &req.password) {
        return Err(AppError::Unauthorized("invalid credentials".to_string()));
    }

    if let Some(id) = user.id {
        user_repository::update_login(id, client_ip);
    }

    let cfg = config::config().read().unwrap();
    let claims =
        Claims::with_custom_claims(JwtClaims { username: req.username }, Duration::from_hours(24));
    let token = cfg
        .jwt_key
        .authenticate(claims)
        .map_err(|_| AppError::Internal("token creation failed".to_string()))?;

    Ok(token)
}
