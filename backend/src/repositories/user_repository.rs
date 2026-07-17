use md5;

use crate::db::pool::db_conn;
use crate::models::user::User;

fn md5_hex(input: &str) -> String {
    let digest = md5::compute(input.as_bytes());
    format!("{:x}", digest)
}

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    md5_hex(&format!("{}{}", md5_hex(password), salt))
}

pub fn gen_salt() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let rand = std::process::id() as u128 ^ nanos;
    format!("{:032x}", rand)
}

pub fn init_user(username: &str, password: &str) {
    if get_user_by_username(username).is_some() {
        return;
    }
    let salt = gen_salt();
    let hashed = hash_password(password, &salt);
    let conn = match db_conn() {
        Some(c) => c,
        None => return,
    };
    conn.execute(
        "INSERT INTO users (username, password, salt) VALUES (?1, ?2, ?3)",
        [username, &hashed, &salt],
    )
    .ok();
}

pub fn get_user_by_username(username: &str) -> Option<User> {
    let conn = db_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, username, password, login_ip, login_time, phone, email, salt \
             FROM users WHERE username = ?",
        )
        .ok()?;
    let mut rows = stmt.query([username]).ok()?;
    rows.next()
        .ok()?
        .map(|r| User {
            id: r.get(0).ok(),
            username: r.get(1).unwrap_or_default(),
            password: r.get(2).unwrap_or_default(),
            login_ip: r.get(3).ok(),
            login_time: r.get(4).ok(),
            phone: r.get(5).ok(),
            email: r.get(6).ok(),
            salt: r.get(7).unwrap_or_default(),
        })
}

pub fn get_user(id: i64) -> Option<User> {
    let conn = db_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, username, password, login_ip, login_time, phone, email, salt \
             FROM users WHERE id = ?",
        )
        .ok()?;
    let mut rows = stmt.query([id]).ok()?;
    rows.next()
        .ok()?
        .map(|r| User {
            id: r.get(0).ok(),
            username: r.get(1).unwrap_or_default(),
            password: r.get(2).unwrap_or_default(),
            login_ip: r.get(3).ok(),
            login_time: r.get(4).ok(),
            phone: r.get(5).ok(),
            email: r.get(6).ok(),
            salt: r.get(7).unwrap_or_default(),
        })
}

pub fn verify_password(user: &User, password: &str) -> bool {
    hash_password(password, &user.salt) == user.password
}

pub fn update_login(id: i64, ip: Option<String>) {
    let conn = match db_conn() {
        Some(c) => c,
        None => return,
    };
    let now = now_string();
    conn.execute(
        "UPDATE users SET login_ip = ?1, login_time = ?2 WHERE id = ?3",
        [ip.unwrap_or_default(), now, id.to_string()],
    )
    .ok();
}

pub fn update_user(
    id: i64,
    username: Option<String>,
    password: Option<String>,
    phone: Option<String>,
    email: Option<String>,
) {
    let conn = match db_conn() {
        Some(c) => c,
        None => return,
    };

    if let Some(u) = username {
        if !u.is_empty() {
            conn.execute("UPDATE users SET username = ?1 WHERE id = ?2", [u, id.to_string()])
                .ok();
        }
    }
    if let Some(p) = password {
        if !p.is_empty() {
            let salt = gen_salt();
            let hashed = hash_password(&p, &salt);
            conn.execute(
                "UPDATE users SET password = ?1, salt = ?2 WHERE id = ?3",
                [hashed, salt, id.to_string()],
            )
            .ok();
        }
    }
    if let Some(p) = phone {
        conn.execute("UPDATE users SET phone = ?1 WHERE id = ?2", [p, id.to_string()])
            .ok();
    }
    if let Some(e) = email {
        conn.execute("UPDATE users SET email = ?1 WHERE id = ?2", [e, id.to_string()])
            .ok();
    }
}
