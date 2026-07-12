use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::dto::file_dto::{
    FileActionResponse, FileItem, FileListResponse, FileReadResponse,
};
use crate::errors::{AppError, AppResult};

fn to_fwd(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn strip_drive_prefix(resolved: &str, input: &str) -> String {
    if !cfg!(windows) || !input.starts_with('/') {
        return resolved.to_string();
    }
    let fwd = resolved.replace('\\', "/");
    if fwd.len() < 2 {
        return fwd;
    }
    if let Some(slash_pos) = fwd.find('/') {
        let drive_part = &fwd[..slash_pos];
        if fwd.starts_with(drive_part) && fwd.len() >= drive_part.len() {
            let rest = &fwd[drive_part.len()..];
            if rest.is_empty() {
                return "/".to_string();
            }
            return rest.to_string();
        }
    }
    fwd
}

fn clean_path(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        let s = s.trim_start_matches("\\\\?\\");
        PathBuf::from(s)
    }
    #[cfg(not(windows))]
    {
        path
    }
}

pub fn sanitize_path_pub(path_str: &str) -> AppResult<PathBuf> {
    sanitize_path(path_str)
}

fn sanitize_path(path_str: &str) -> AppResult<PathBuf> {
    if path_str.contains("..") {
        return Err(AppError::BadRequest(
            "Path traversal is not allowed".to_string(),
        ));
    }

    let path = if cfg!(windows) && path_str.starts_with('/') {
        let cwd = std::env::current_dir().map_err(|e| {
            AppError::BadRequest(format!("Cannot get current directory: {}", e))
        })?;
        let root = cwd.ancestors().last().unwrap_or(Path::new("C:\\"));
        let trimmed = path_str.trim_start_matches('/');
        if trimmed.is_empty() {
            root.to_path_buf()
        } else {
            root.join(trimmed)
        }
    } else {
        let p = PathBuf::from(path_str);
        if !p.is_absolute() {
            return Err(AppError::BadRequest("Path must be absolute".to_string()));
        }
        p
    };

    if path.exists() {
        let p = path
            .canonicalize()
            .map_err(|e| AppError::BadRequest(format!("Invalid path: {}", e)))?;
        Ok(clean_path(p))
    } else if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(AppError::NotFound(format!(
                "Parent directory not found: {}",
                parent.display()
            )));
        }
        let canonical_parent = parent
            .canonicalize()
            .map_err(|_| AppError::BadRequest("Invalid path".to_string()))?;
        let canonical_parent = clean_path(canonical_parent);
        if let Some(file_name) = path.file_name() {
            Ok(canonical_parent.join(file_name))
        } else {
            Ok(canonical_parent)
        }
    } else {
        Err(AppError::BadRequest("Invalid path".to_string()))
    }
}

fn map_io_error(e: std::io::Error, path: &Path) -> AppError {
    match e.kind() {
        std::io::ErrorKind::NotFound => {
            AppError::NotFound(format!("Path not found: {}", path.display()))
        }
        std::io::ErrorKind::PermissionDenied => {
            AppError::BadRequest(format!("Permission denied: {}", path.display()))
        }
        _ => AppError::BadRequest(format!("IO error for {}: {}", path.display(), e)),
    }
}

#[cfg(unix)]
fn format_permissions(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    format!("{:o}", mode & 0o7777)
}

#[cfg(unix)]
fn get_owner(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt;
    let uid = metadata.uid();
    // Try to resolve from /etc/passwd
    if let Ok(content) = std::fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                if let Ok(file_uid) = parts[2].parse::<u32>() {
                    if file_uid == uid {
                        return parts[0].to_string();
                    }
                }
            }
        }
    }
    uid.to_string()
}

#[cfg(not(unix))]
fn format_permissions(metadata: &std::fs::Metadata) -> String {
    "0644".to_string()
}

#[cfg(not(unix))]
fn get_owner(_metadata: &std::fs::Metadata) -> String {
    String::new()
}

pub fn list_dir(path_str: &str, show_hidden: bool) -> AppResult<FileListResponse> {
    let path = sanitize_path(path_str)?;

    if !path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Not a directory: {}",
            path.display()
        )));
    }

    let mut items = Vec::new();
    let mut entries = std::fs::read_dir(&path).map_err(|e| map_io_error(e, &path))?;

    while let Some(entry) = entries.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();

        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let is_link = file_type.is_symlink();

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_dir = metadata.is_dir();
        let size = metadata.len();
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mode = format_permissions(&metadata);
        let owner = get_owner(&metadata);

        items.push(FileItem {
            name,
            path: strip_drive_prefix(&to_fwd(&entry.path()), path_str),
            size,
            is_dir,
            is_link,
            mode,
            owner,
            modified,
        });
    }

    items.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.cmp(&b.name)
        }
    });

    let parent = path.parent().map(|p| strip_drive_prefix(&to_fwd(p), path_str));

    Ok(FileListResponse {
        total: items.len(),
        path: strip_drive_prefix(&to_fwd(&path), path_str),
        parent,
        items,
    })
}

pub fn read_file(path_str: &str) -> AppResult<FileReadResponse> {
    let path = sanitize_path(path_str)?;

    if !path.is_file() {
        return Err(AppError::BadRequest(format!(
            "Not a file: {}",
            path.display()
        )));
    }

    let metadata = std::fs::metadata(&path).map_err(|e| map_io_error(e, &path))?;
    let content = std::fs::read_to_string(&path).map_err(|e| map_io_error(e, &path))?;

    Ok(FileReadResponse {
        path: strip_drive_prefix(&to_fwd(&path), path_str),
        content,
        size: metadata.len(),
    })
}

pub fn write_file(path_str: &str, content: &str) -> AppResult<FileActionResponse> {
    let path = sanitize_path(path_str)?;

    std::fs::write(&path, content).map_err(|e| map_io_error(e, &path))?;

    Ok(FileActionResponse {
        success: true,
        message: "File saved".to_string(),
    })
}

pub fn create_path(path_str: &str, file_type: &str) -> AppResult<FileActionResponse> {
    let path = sanitize_path(path_str)?;

    if path.exists() {
        return Err(AppError::BadRequest(format!(
            "Already exists: {}",
            path.display()
        )));
    }

    match file_type {
        "dir" => {
            std::fs::create_dir(&path).map_err(|e| map_io_error(e, &path))?;
            Ok(FileActionResponse {
                success: true,
                message: "Directory created".to_string(),
            })
        }
        "file" => {
            std::fs::File::create(&path).map_err(|e| map_io_error(e, &path))?;
            Ok(FileActionResponse {
                success: true,
                message: "File created".to_string(),
            })
        }
        _ => Err(AppError::BadRequest(format!(
            "Invalid type: {}, must be 'file' or 'dir'",
            file_type
        ))),
    }
}

pub fn delete_path(path_str: &str) -> AppResult<FileActionResponse> {
    let path = sanitize_path(path_str)?;

    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "Path not found: {}",
            path.display()
        )));
    }

    if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| map_io_error(e, &path))?;
    } else {
        std::fs::remove_file(&path).map_err(|e| map_io_error(e, &path))?;
    }

    Ok(FileActionResponse {
        success: true,
        message: "Deleted".to_string(),
    })
}

pub fn rename_path(path_str: &str, new_name: &str) -> AppResult<FileActionResponse> {
    let old_path = sanitize_path(path_str)?;

    if !old_path.exists() {
        return Err(AppError::NotFound(format!(
            "Path not found: {}",
            old_path.display()
        )));
    }

    let parent = old_path
        .parent()
        .ok_or_else(|| AppError::BadRequest("Cannot rename root".to_string()))?;

    let new_path = parent.join(new_name);

    if new_path.exists() {
        return Err(AppError::BadRequest(format!(
            "Target already exists: {}",
            new_path.display()
        )));
    }

    std::fs::rename(&old_path, &new_path).map_err(|e| map_io_error(e, &old_path))?;

    Ok(FileActionResponse {
        success: true,
        message: "Renamed".to_string(),
    })
}

pub fn dir_size(path_str: &str) -> AppResult<u64> {
    let path = sanitize_path(path_str)?;

    if !path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Not a directory: {}",
            path.display()
        )));
    }

    fn walk(p: &std::path::Path) -> u64 {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(p) {
            for entry in entries.flatten() {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                if meta.is_dir() {
                    total += walk(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
        total
    }

    Ok(walk(&path))
}

pub fn copy_file(src_str: &str, dest_str: &str) -> AppResult<FileActionResponse> {
    let src = sanitize_path(src_str)?;
    let dest = sanitize_path(dest_str)?;

    if !src.exists() {
        return Err(AppError::NotFound(format!(
            "Source not found: {}",
            src.display()
        )));
    }

    if src.is_dir() {
        fn copy_dir_all(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
            std::fs::create_dir_all(to)?;
            for entry in std::fs::read_dir(from)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                let target = to.join(entry.file_name());
                if ty.is_dir() {
                    copy_dir_all(&entry.path(), &target)?;
                } else {
                    std::fs::copy(entry.path(), &target)?;
                }
            }
            Ok(())
        }
        copy_dir_all(&src, &dest).map_err(|e| {
            AppError::BadRequest(format!("Copy failed: {}", e))
        })?;
    } else {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::BadRequest(format!("Cannot create parent: {}", e))
            })?;
        }
        std::fs::copy(&src, &dest).map_err(|e| {
            AppError::BadRequest(format!("Copy failed: {}", e))
        })?;
    }

    Ok(FileActionResponse {
        success: true,
        message: "Copied".to_string(),
    })
}

pub fn download_file(url: &str, dest_dir: &str) -> AppResult<FileActionResponse> {
    let dir = sanitize_path(dest_dir)?;

    if !dir.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Not a directory: {}",
            dir.display()
        )));
    }

    #[cfg(unix)]
    {
        let output = std::process::Command::new("wget")
            .arg("-q")
            .arg("-P")
            .arg(&dir)
            .arg("--content-disposition")
            .arg(url)
            .output()
            .map_err(|e| AppError::BadRequest(format!("wget not found or failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::BadRequest(format!(
                "Download failed: {}",
                stderr.trim()
            )));
        }
    }

    #[cfg(windows)]
    {
        let output = std::process::Command::new("curl")
            .arg("-L")
            .arg("-s")
            .arg("-J")
            .arg("-O")
            .arg("--output-dir")
            .arg(&dir)
            .arg(url)
            .output()
            .map_err(|e| AppError::BadRequest(format!("curl not found or failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::BadRequest(format!(
                "Download failed: {}",
                stderr.trim()
            )));
        }
    }

    Ok(FileActionResponse {
        success: true,
        message: "Download started".to_string(),
    })
}
