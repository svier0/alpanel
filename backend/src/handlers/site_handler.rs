use axum::{Json, http::HeaderMap};

use crate::dto::site_dto::{CreateSiteRequest, SiteResponse, UpdateSiteRequest};
use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::repositories::site_repository;

pub async fn list_sites(
    headers: HeaderMap,
) -> AppResult<Json<Vec<SiteResponse>>> {
    check_auth(&headers)?;
    let sites = site_repository::list_sites(None)
        .into_iter()
        .map(|s| site_repository::to_response(&s))
        .collect();
    Ok(Json(sites))
}

pub async fn create_site(
    headers: HeaderMap,
    Json(body): Json<CreateSiteRequest>,
) -> AppResult<Json<SiteResponse>> {
    check_auth(&headers)?;
    let id = site_repository::create_site(&body)?;
    let site = site_repository::get_site(id).ok_or_else(|| {
        crate::errors::AppError::Internal("创建后无法读取站点".into())
    })?;
    Ok(Json(site_repository::to_response(&site)))
}

pub async fn update_site(
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(body): Json<UpdateSiteRequest>,
) -> AppResult<Json<SiteResponse>> {
    check_auth(&headers)?;
    site_repository::update_site(id, &body)?;
    let site = site_repository::get_site(id).ok_or_else(|| {
        crate::errors::AppError::Internal("无法读取站点".into())
    })?;
    Ok(Json(site_repository::to_response(&site)))
}

pub async fn delete_site(
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    site_repository::delete_site(id)?;
    Ok(Json(serde_json::json!({"ok": true})))
}
