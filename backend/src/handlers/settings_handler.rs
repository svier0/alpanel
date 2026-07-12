use std::time::Duration;

use axum::{http::HeaderMap, Json};

use crate::config;
use crate::dto::settings_dto::{SettingsResponse, SettingsUpdate};
use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::services::settings_service;

pub async fn get_settings(
    headers: HeaderMap,
) -> AppResult<Json<SettingsResponse>> {
    check_auth(&headers)?;
    let settings = settings_service::get_settings()?;
    Ok(Json(settings))
}

pub async fn update_settings(
    headers: HeaderMap,
    Json(body): Json<SettingsUpdate>,
) -> AppResult<Json<SettingsResponse>> {
    check_auth(&headers)?;
    let (settings, port_changed) = settings_service::update_settings(body);

    if port_changed {
        let resp = Json(settings);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            config::restart_panel();
        });
        return Ok(resp);
    }

    Ok(Json(settings))
}
