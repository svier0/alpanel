use axum::{Json, http::HeaderMap};

use crate::errors::AppResult;
use crate::middleware::auth::check_auth;

pub async fn list_users(
    headers: HeaderMap,
) -> AppResult<Json<Vec<String>>> {
    check_auth(&headers)?;
    let mut users: Vec<String> = vec!["www".to_string()];
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() < 3 {
                continue;
            }
            if let Ok(uid) = parts[2].parse::<u32>() {
                if uid >= 1000 && uid < 65534 {
                    users.push(parts[0].to_string());
                }
            }
        }
    }
    Ok(Json(users))
}
