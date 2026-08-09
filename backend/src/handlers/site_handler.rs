use axum::{Json, http::HeaderMap};

use crate::dto::site_dto::{CreateSiteRequest, SiteResponse, UpdateSiteRequest};
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::check_auth;
use crate::repositories::site_repository;
use crate::services::file_service::sanitize_path_pub;

fn ensure_site_dir(path: &str) -> AppResult<()> {
    if path.trim().is_empty() {
        return Ok(());
    }
    let p = sanitize_path_pub(path)?;
    if !p.exists() {
        std::fs::create_dir_all(&p)
            .map_err(|e| AppError::BadRequest(format!("创建站点根目录失败: {}", e)))?;
    }
    let out = std::process::Command::new("chown")
        .args(["www:www", &p.to_string_lossy()])
        .output()
        .map_err(|e| AppError::BadRequest(format!("设置站点目录属主失败: {}", e)))?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr);
        return Err(AppError::BadRequest(format!(
            "设置站点目录属主失败: {}",
            msg.trim()
        )));
    }
    Ok(())
}

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

pub async fn get_project_types(
    headers: HeaderMap,
) -> AppResult<Json<Vec<crate::dto::site_dto::ProjectTypeInfo>>> {
    check_auth(&headers)?;
    Ok(Json(crate::dto::site_dto::project_type_list()))
}

pub async fn create_site(
    headers: HeaderMap,
    Json(body): Json<CreateSiteRequest>,
) -> AppResult<Json<SiteResponse>> {
    check_auth(&headers)?;
    if body.project_type.as_deref().unwrap_or("PHP") == "PHP" {
        ensure_site_dir(&body.path)?;
    }
    let id = site_repository::create_site(&body)?;
    let site = site_repository::get_site(id).ok_or_else(|| {
        crate::errors::AppError::Internal("创建后无法读取站点".into())
    })?;
    if site.project_type.as_deref().unwrap_or("PHP") == "PHP" {
        crate::services::site_service::generate_site_vhost(
            &site.name,
            &site.path,
            site.status.as_deref(),
            &body.domains,
        )?;
    }
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
    if site.project_type.as_deref().unwrap_or("PHP") == "PHP" {
        let domains: Vec<crate::dto::site_dto::CreateDomainInline> = crate::repositories::domain_repository::list_domains(id)
            .into_iter()
            .map(|d| crate::dto::site_dto::CreateDomainInline {
                name: d.name,
                port: d.port,
            })
            .collect();
        crate::services::site_service::generate_site_vhost(
            &site.name,
            &site.path,
            site.status.as_deref(),
            &domains,
        )?;
    }
    Ok(Json(site_repository::to_response(&site)))
}

pub async fn delete_site(
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> AppResult<Json<serde_json::Value>> {
    check_auth(&headers)?;
    let site = site_repository::get_site(id).ok_or_else(|| {
        crate::errors::AppError::Internal("无法读取站点".into())
    })?;
    site_repository::delete_site(id)?;
    crate::services::site_service::remove_site_vhost(&site)?;
    Ok(Json(serde_json::json!({"ok": true})))
}
