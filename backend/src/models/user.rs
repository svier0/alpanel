use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
    pub login_ip: Option<String>,
    pub login_time: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub salt: String,
}
