use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("linux-musl") {
        return;
    }

    let arch = target.split('-').next().unwrap_or("x86_64");
    let alpine_arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => panic!("unsupported musl target arch: {}", arch),
    };

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let lib_dir = out_dir.join("sqlite-static").join("usr").join("lib");
    let a_file = lib_dir.join("libsqlite3.a");

    if a_file.exists() {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=sqlite3");
        return;
    }

    let root = format!("https://dl-cdn.alpinelinux.org/alpine/v3.21/main/{}", alpine_arch);
    let apk_dir = out_dir.join("sqlite-static");
    std::fs::create_dir_all(&lib_dir).ok();

    let version = get_sqlite_version(&root, &out_dir);
    let apk_url = format!("{}/sqlite-static-{}.apk", root, version);
    let apk_path = out_dir.join("sqlite-static.apk");

    if !apk_path.exists() {
        let status = Command::new("curl")
            .args(["-fsSL", "--connect-timeout", "10", "--max-time", "120",
                   "-o", &apk_path.display().to_string(), &apk_url])
            .status()
            .expect("Failed to run curl");
        if !status.success() {
            panic!("Failed to download sqlite-static from {}", apk_url);
        }
    }

    let status = Command::new("tar")
        .args(["-xzf", &apk_path.display().to_string(), "-C", &apk_dir.display().to_string(),
               "usr/lib/libsqlite3.a"])
        .status()
        .expect("Failed to run tar");
    assert!(status.success(), "Failed to extract sqlite-static");

    // libsqlite3-sys outputs `cargo:rustc-link-lib=sqlite3` (dynamic preferred).
    // Override with static so the linker uses libsqlite3.a instead.
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=sqlite3");
}

fn get_sqlite_version(root: &str, out_dir: &PathBuf) -> String {
    if let Ok(v) = std::env::var("SQLITE_VERSION") {
        return v;
    }

    let index_path = out_dir.join("APKINDEX.tar.gz");
    if !index_path.exists() {
        let status = Command::new("curl")
            .args(["-fsSL", "--connect-timeout", "10", "--max-time", "30",
                   "-o", &index_path.display().to_string(),
                   &format!("{}/APKINDEX.tar.gz", root)])
            .status()
            .expect("Failed to run curl");
        assert!(status.success(), "Failed to download APKINDEX");
    }

    let text_path = out_dir.join("APKINDEX");
    if !text_path.exists() {
        let status = Command::new("tar")
            .args(["-xzf", &index_path.display().to_string(),
                   "-C", &out_dir.display().to_string()])
            .status()
            .expect("Failed to run tar");
        assert!(status.success(), "Failed to extract APKINDEX");
    }

    let content = std::fs::read_to_string(&text_path).expect("Failed to read APKINDEX");
    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with("P:") && line.trim() == "P:sqlite-static" {
            while let Some(v) = lines.next() {
                if v.starts_with("V:") {
                    return v[2..].trim().to_string();
                }
                if v.is_empty() {
                    break;
                }
            }
        }
    }
    panic!("sqlite-static not found in APKINDEX")
}
