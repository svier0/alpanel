use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateSiteRequest {
    pub name: String,
    pub path: String,
    pub status: Option<String>,
    pub project_type: Option<String>,
    pub ps: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSiteRequest {
    pub name: Option<String>,
    pub path: Option<String>,
    pub status: Option<String>,
    pub project_type: Option<String>,
    pub ps: Option<String>,
}

#[derive(Serialize)]
pub struct SiteResponse {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub status: String,
    pub project_type: String,
    pub ps: String,
    pub addtime: String,
}
