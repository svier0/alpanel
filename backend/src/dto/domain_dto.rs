use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateDomainRequest {
    pub pid: i64,
    pub name: String,
    pub port: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateDomainRequest {
    pub name: Option<String>,
    pub port: Option<i64>,
}

#[derive(Serialize, Clone)]
pub struct DomainResponse {
    pub id: i64,
    pub pid: i64,
    pub name: String,
    pub port: i64,
    pub addtime: String,
}
