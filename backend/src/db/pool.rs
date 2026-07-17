use std::path::PathBuf;

use rusqlite::Connection;
use tracing::info;

fn db_path() -> PathBuf {
    let env_path = std::env::var("PANEL_ENV").unwrap_or_else(|_| ".env".to_string());
    let path = PathBuf::from(&env_path);
    let dir = path.parent().unwrap_or(std::path::Path::new("."));
    dir.join("data/db/alpanel.db")
}

pub fn init_db() {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let is_new = !path.exists();

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
            project_type TEXT,
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
        CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT
        );
        ",
    )
    .expect("Failed to create tables");

    conn.execute(
        "INSERT OR IGNORE INTO config (key, value) VALUES ('status', '1')",
        [],
    )
    .ok();
    conn.execute(
        "INSERT OR IGNORE INTO config (key, value) VALUES ('mysql_root', '')",
        [],
    )
    .ok();

    if is_new {
        info!("Database created at {:?}", path);
    } else {
        info!("alpanel.db already exists at {:?}", path);
    }
}

pub fn db_conn() -> Option<Connection> {
    Connection::open(db_path()).ok()
}

pub fn get_config(key: &str) -> Option<String> {
    let path = db_path();
    let conn = Connection::open(&path).ok()?;
    let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?").ok()?;
    let mut rows = stmt.query([key]).ok()?;
    rows.next().ok()?.map(|r| r.get::<_, String>(0).unwrap_or_default())
}

pub fn set_config(key: &str, value: &str) {
    let path = db_path();
    if let Ok(conn) = Connection::open(&path) {
        conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            [key, value],
        )
        .ok();
    }
}

