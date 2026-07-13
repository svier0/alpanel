use axum::extract::Query;
use axum::body::Body;
use axum::http::{HeaderMap, Response, StatusCode};
use axum::{Json};

use crate::dto::file_dto::{
    DirSizeQuery, DirSizeResponse, FileActionResponse, FileCopyRequest, FileCreateRequest,
    FileDeleteRequest, FileDownloadRequest, FileListQuery, FileListResponse, FileReadQuery,
    FileReadResponse, FileRenameRequest, FileWriteRequest,
};
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::check_auth;
use crate::services::file_service;

pub async fn list(
    headers: HeaderMap,
    Query(query): Query<FileListQuery>,
) -> AppResult<Json<FileListResponse>> {
    check_auth(&headers)?;
    let res = file_service::list_dir(&query.path)?;
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
    let path = query.path.clone();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio::task::spawn_blocking(move || file_service::dir_size(&path)),
    )
    .await;
    match result {
        Ok(Ok(Ok(size))) => Ok(Json(DirSizeResponse { size })),
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(_)) => Err(AppError::BadRequest("计算目录大小失败".to_string())),
        Err(_) => Err(AppError::BadRequest("计算超时（30秒），目录过大".to_string())),
    }
}

pub async fn copy(
    headers: HeaderMap,
    Json(body): Json<FileCopyRequest>,
) -> AppResult<Json<FileActionResponse>> {
    check_auth(&headers)?;
    let res = file_service::copy_file(&body.src, &body.dest)?;
    Ok(Json(res))
}

pub async fn download(
    headers: HeaderMap,
    Json(body): Json<FileDownloadRequest>,
) -> AppResult<Json<FileActionResponse>> {
    check_auth(&headers)?;
    let res = file_service::download_file(&body.url, &body.path)?;
    Ok(Json(res))
}

pub async fn stream_download(
    headers: HeaderMap,
    Query(query): Query<FileReadQuery>,
) -> Result<Response<Body>, AppError> {
    check_auth(&headers)?;
    let path = file_service::sanitize_path_pub(&query.path)?;

    if !path.is_file() {
        return Err(AppError::BadRequest(format!(
            "Not a file: {}",
            path.display()
        )));
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());

    let data = std::fs::read(&path).map_err(|e| AppError::BadRequest(format!("Read failed: {}", e)))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", name)
            .parse()
            .unwrap(),
    );
    headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
    headers.insert(
        "Content-Length",
        data.len().to_string().parse().unwrap(),
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(data))
        .unwrap())
}
