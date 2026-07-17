use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    pub status: Option<String>,
    pub project_type: Option<String>,
    pub phpversion: Option<String>,
    pub ps: Option<String>,
    pub addtime: Option<String>,
    pub project_cmd: Option<String>,
    pub project_port: Option<i64>,
    pub run_user: Option<String>,
    pub is_onpower: Option<i64>,
}
