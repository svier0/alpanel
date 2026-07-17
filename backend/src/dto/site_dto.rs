use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateDomainInline {
    pub name: String,
    pub port: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateSiteRequest {
    pub name: Option<String>,
    pub domains: Vec<CreateDomainInline>,
    pub path: String,
    pub status: Option<String>,
    pub project_type: Option<String>,
    pub phpversion: Option<String>,
    pub ps: Option<String>,
    pub project_cmd: Option<String>,
    pub project_port: Option<i64>,
    pub run_user: Option<String>,
    pub is_onpower: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateSiteRequest {
    pub name: Option<String>,
    pub path: Option<String>,
    pub status: Option<String>,
    pub project_type: Option<String>,
    pub phpversion: Option<String>,
    pub ps: Option<String>,
    pub project_cmd: Option<String>,
    pub project_port: Option<i64>,
    pub run_user: Option<String>,
    pub is_onpower: Option<i64>,
}

#[derive(Serialize, Clone)]
pub struct DomainInline {
    pub id: i64,
    pub name: String,
    pub port: i64,
}

#[derive(Serialize)]
pub struct SiteResponse {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub status: String,
    pub project_type: String,
    pub phpversion: String,
    pub ps: String,
    pub addtime: String,
    pub project_cmd: String,
    pub project_port: i64,
    pub run_user: String,
    pub is_onpower: i64,
    pub domains: Vec<DomainInline>,
}

#[derive(Serialize)]
pub struct ProjectTypeInfo {
    pub name: String,
    pub title: String,
    pub visibled: i64,
}

pub fn project_type_list() -> Vec<ProjectTypeInfo> {
    vec![
        ProjectTypeInfo { name: "PHP".into(), title: "普通项目".into(), visibled: 1 },
        ProjectTypeInfo { name: "Other".into(), title: "其它项目".into(), visibled: 1 },
        ProjectTypeInfo { name: "Proxy".into(), title: "反向代理".into(), visibled: 0 },
    ]
}
