use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct FileListQuery {
    pub path: String,
}

#[derive(Serialize, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_link: bool,
    pub mode: String,
    pub owner: String,
    pub modified: u64,
    pub ps: String,
}

#[derive(Serialize)]
pub struct FileListResponse {
    pub path: String,
    pub parent: Option<String>,
    pub items: Vec<FileItem>,
    pub total: usize,
}

#[derive(Deserialize)]
pub struct FileReadQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct FileReadResponse {
    pub path: String,
    pub content: String,
    pub size: u64,
}

#[derive(Deserialize)]
pub struct FileWriteRequest {
    pub path: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct FileCreateRequest {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
}

#[derive(Deserialize)]
pub struct FileDeleteRequest {
    pub path: String,
}

#[derive(Deserialize)]
pub struct FileRenameRequest {
    pub path: String,
    pub new_name: String,
}

#[derive(Serialize)]
pub struct FileActionResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct DirSizeQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct DirSizeResponse {
    pub size: u64,
}

#[derive(Deserialize)]
pub struct FileCopyRequest {
    pub src: String,
    pub dest: String,
}

#[derive(Deserialize)]
pub struct FileDownloadRequest {
    pub url: String,
    pub path: String,
}

#[derive(Deserialize)]
pub struct FilePsRequest {
    pub path: String,
    pub ps: String,
}
