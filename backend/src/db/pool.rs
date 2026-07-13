use std::path::PathBuf;

use rusqlite::Connection;
use tracing::info;

fn db_path() -> PathBuf {
    let env_path = std::env::var("PANEL_ENV").unwrap_or_else(|_| ".env".to_string());
    let path = PathBuf::from(&env_path);
    let dir = path.parent().unwrap_or(std::path::Path::new("."));
    dir.join("alpanel.db")
}

pub fn init_db() {
    let path = db_path();
    if path.exists() {
        info!("alpanel.db already exists at {:?}", path);
        return;
    }

    let conn = Connection::open(&path).expect("Failed to create database");
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT,
            password TEXT,
            login_ip TEXT,
            login_time TEXT,
            phone TEXT,
            email TEXT,
            salt TEXT
        );
        CREATE TABLE IF NOT EXISTS sites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            path TEXT,
            status TEXT,
            \"index\" TEXT,
            ps TEXT,
            addtime TEXT
        );
        CREATE TABLE IF NOT EXISTS domain (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pid INTEGER,
            name TEXT,
            port INTEGER,
            addtime TEXT
        );
        ",
    )
    .expect("Failed to create tables");

    info!("Database created at {:?}", path);
}
