use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct SettingsResponse {
    pub port: u16,
    pub user: String,
    pub title: String,
    pub theme: String,
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub title: Option<String>,
    pub theme: Option<String>,
}
