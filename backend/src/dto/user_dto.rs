use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub login_ip: Option<String>,
    pub login_time: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}
