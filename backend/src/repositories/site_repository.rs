use crate::db::pool::db_conn;
use crate::dto::site_dto::{CreateSiteRequest, SiteResponse, UpdateSiteRequest};
use crate::errors::AppResult;
use crate::models::site::Site;
use crate::repositories::domain_repository::{create_domain, list_domains};

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn create_site(req: &CreateSiteRequest) -> AppResult<i64> {
    use crate::errors::AppError;
    let path = req.path.trim().to_string();
    if path.is_empty() {
        return Err(AppError::BadRequest("根目录不能为空".into()));
    }
    let first_domain = req.domains.first().map(|d| d.name.trim().to_string());
    let name = match &req.name {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => match &first_domain {
            Some(d) => d.clone(),
            None => return Err(AppError::BadRequest("站点名称或域名不能为空".into())),
        },
    };
    let conn = db_conn().ok_or_else(|| AppError::Internal("数据库不可用".into()))?;
    let addtime = now_string();
    let status = req.status.clone().unwrap_or_else(|| "0".into());
    let project_type = req.project_type.clone().unwrap_or_else(|| "PHP".into());
    let ps = req.ps.clone().unwrap_or_default();
    conn.execute(
        "INSERT INTO sites (name, path, status, project_type, ps, addtime) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [&name, &path, &status, &project_type, &ps, &addtime],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let site_id = conn.last_insert_rowid();
    if let Some(fd) = first_domain {
        let port = req.domains.first().and_then(|d| d.port);
        let _ = create_domain(site_id, &fd, port);
    }
    Ok(site_id)
}

pub fn list_sites(project_type: Option<&str>) -> Vec<Site> {
    let conn = match db_conn() {
        Some(c) => c,
        None => return vec![],
    };
    let sql = match project_type {
        Some(_) => "SELECT id, name, path, status, project_type, ps, addtime FROM sites WHERE project_type = ? ORDER BY id DESC",
        None => "SELECT id, name, path, status, project_type, ps, addtime FROM sites ORDER BY id DESC",
    };
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let rows = match project_type {
        Some(pt) => stmt.query([pt]),
        None => stmt.query([]),
    };
    let mut rows = match rows {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let mut out = vec![];
    while let Ok(Some(r)) = rows.next() {
        out.push(Site {
            id: r.get(0).ok(),
            name: r.get(1).unwrap_or_default(),
            path: r.get(2).unwrap_or_default(),
            status: r.get(3).ok(),
            project_type: r.get(4).ok(),
            ps: r.get(5).ok(),
            addtime: r.get(6).ok(),
        });
    }
    out
}

pub fn get_site(id: i64) -> Option<Site> {
    let conn = db_conn()?;
    let mut stmt = conn
        .prepare("SELECT id, name, path, status, project_type, ps, addtime FROM sites WHERE id = ?")
        .ok()?;
    let mut rows = stmt.query([id]).ok()?;
    rows.next().ok()?.map(|r| Site {
        id: r.get(0).ok(),
        name: r.get(1).unwrap_or_default(),
        path: r.get(2).unwrap_or_default(),
        status: r.get(3).ok(),
        project_type: r.get(4).ok(),
        ps: r.get(5).ok(),
        addtime: r.get(6).ok(),
    })
}

pub fn update_site(id: i64, req: &UpdateSiteRequest) -> AppResult<()> {
    use crate::errors::AppError;
    if get_site(id).is_none() {
        return Err(AppError::NotFound("站点不存在".into()));
    }
    let conn = db_conn().ok_or_else(|| AppError::Internal("数据库不可用".into()))?;
    if let Some(name) = &req.name {
        if !name.trim().is_empty() {
            conn.execute("UPDATE sites SET name = ?1 WHERE id = ?2", [name, &id.to_string()])
                .ok();
        }
    }
    if let Some(path) = &req.path {
        if !path.trim().is_empty() {
            conn.execute("UPDATE sites SET path = ?1 WHERE id = ?2", [path, &id.to_string()])
                .ok();
        }
    }
    if let Some(status) = &req.status {
        conn.execute("UPDATE sites SET status = ?1 WHERE id = ?2", [status, &id.to_string()])
            .ok();
    }
    if let Some(project_type) = &req.project_type {
        conn.execute("UPDATE sites SET project_type = ?1 WHERE id = ?2", [project_type, &id.to_string()])
            .ok();
    }
    if let Some(ps) = &req.ps {
        conn.execute("UPDATE sites SET ps = ?1 WHERE id = ?2", [ps, &id.to_string()])
            .ok();
    }
    Ok(())
}

pub fn delete_site(id: i64) -> AppResult<()> {
    use crate::errors::AppError;
    let conn = db_conn().ok_or_else(|| AppError::Internal("数据库不可用".into()))?;
    conn.execute("DELETE FROM domain WHERE pid = ?", [id.to_string()])
        .ok();
    conn.execute("DELETE FROM sites WHERE id = ?", [id.to_string()])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn to_response(s: &Site) -> SiteResponse {
    let id = s.id.unwrap_or(0);
    let domains = list_domains(id)
        .into_iter()
        .map(|d| crate::dto::site_dto::DomainInline {
            id: d.id.unwrap_or(0),
            name: d.name,
            port: d.port.unwrap_or(0),
        })
        .collect();
    SiteResponse {
        id,
        name: s.name.clone(),
        path: s.path.clone(),
        status: s.status.clone().unwrap_or_default(),
        project_type: s.project_type.clone().unwrap_or_default(),
        ps: s.ps.clone().unwrap_or_default(),
        addtime: s.addtime.clone().unwrap_or_default(),
        domains,
    }
}
