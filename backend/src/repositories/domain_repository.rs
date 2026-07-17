use crate::db::pool::db_conn;
use crate::errors::AppResult;
use crate::models::domain::Domain;

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn create_domain(pid: i64, name: &str, port: Option<i64>) -> AppResult<i64> {
    use crate::errors::AppError;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("域名不能为空".into()));
    }
    let conn = db_conn().ok_or_else(|| AppError::Internal("数据库不可用".into()))?;
    let addtime = now_string();
    conn.execute(
        "INSERT INTO domain (pid, name, port, addtime) VALUES (?1, ?2, ?3, ?4)",
        [&pid.to_string(), &name, &port.unwrap_or(0).to_string(), &addtime],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(conn.last_insert_rowid())
}

pub fn list_domains(pid: i64) -> Vec<Domain> {
    let conn = match db_conn() {
        Some(c) => c,
        None => return vec![],
    };
    let mut stmt = match conn.prepare(
        "SELECT id, pid, name, port, addtime FROM domain WHERE pid = ? ORDER BY id ASC",
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let mut rows = match stmt.query([pid]) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let mut out = vec![];
    while let Ok(Some(r)) = rows.next() {
        out.push(Domain {
            id: r.get(0).ok(),
            pid: r.get(1).unwrap_or(0),
            name: r.get(2).unwrap_or_default(),
            port: r.get(3).ok(),
            addtime: r.get(4).ok(),
        });
    }
    out
}
