use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    pub status: Option<String>,
    pub index: Option<String>,
    pub ps: Option<String>,
    pub addtime: Option<String>,
}
