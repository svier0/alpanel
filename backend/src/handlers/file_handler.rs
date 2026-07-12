use axum::extract::Query;
use axum::{http::HeaderMap, Json};

use crate::dto::file_dto::{
    DirSizeQuery, DirSizeResponse, FileActionResponse, FileCopyRequest, FileCreateRequest,
    FileDeleteRequest, FileListQuery, FileListResponse, FileReadQuery, FileReadResponse,
    FileRenameRequest, FileWriteRequest,
};
use crate::errors::AppResult;
use crate::middleware::auth::check_auth;
use crate::services::file_service;

pub async fn list(
    headers: HeaderMap,
    Query(query): Query<FileListQuery>,
) -> AppResult<Json<FileListResponse>> {
    check_auth(&headers)?;
    let res = file_service::list_dir(&query.path, query.show_hidden.unwrap_or(false))?;
    Ok(Json(res))
}

pub async fn read(
    headers: HeaderMap,
    Query(query): Query<FileReadQuery>,
) -> AppResult<Json<FileReadResponse>> {
    check_auth(&headers)?;
    let res = file_service::read_file(&query.path)?;
    Ok(Json(res))
}

pub async fn write(
    headers: HeaderMap,
    Json(body): Json<FileWriteRequest>,
) -> AppResult<Json<FileActionResponse>> {
    check_auth(&headers)?;
    let res = file_service::write_file(&body.path, &body.content)?;
    Ok(Json(res))
}

pub async fn create(
    headers: HeaderMap,
    Json(body): Json<FileCreateRequest>,
) -> AppResult<Json<FileActionResponse>> {
    check_auth(&headers)?;
    let res = file_service::create_path(&body.path, &body.file_type)?;
    Ok(Json(res))
}

pub async fn delete(
    headers: HeaderMap,
    Json(body): Json<FileDeleteRequest>,
) -> AppResult<Json<FileActionResponse>> {
    check_auth(&headers)?;
    let res = file_service::delete_path(&body.path)?;
    Ok(Json(res))
}

pub async fn rename(
    headers: HeaderMap,
    Json(body): Json<FileRenameRequest>,
) -> AppResult<Json<FileActionResponse>> {
    check_auth(&headers)?;
    let res = file_service::rename_path(&body.path, &body.new_name)?;
    Ok(Json(res))
}

pub async fn dir_size(
    headers: HeaderMap,
    Query(query): Query<DirSizeQuery>,
) -> AppResult<Json<DirSizeResponse>> {
    check_auth(&headers)?;
    let size = file_service::dir_size(&query.path)?;
    Ok(Json(DirSizeResponse { size }))
}

pub async fn copy(
    headers: HeaderMap,
    Json(body): Json<FileCopyRequest>,
) -> AppResult<Json<FileActionResponse>> {
    check_auth(&headers)?;
    let res = file_service::copy_file(&body.src, &body.dest)?;
    Ok(Json(res))
}
