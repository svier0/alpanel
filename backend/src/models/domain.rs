use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: Option<i64>,
    pub pid: i64,
    pub name: String,
    pub port: Option<i64>,
    pub addtime: Option<String>,
}
