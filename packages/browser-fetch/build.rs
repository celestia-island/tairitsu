//! ort-style build-time Chromium resolver. When the `auto-fetch` feature is on,
//! this downloads the pinned Chrome for Testing into a shared cache (once, then
//! cached across builds) and bakes the path into the library via
//! `cargo:rustc-env=TAIRITSU_BROWSER_PATH=...`.
//!
//! KEEP IN SYNC with `src/lib.rs` (version, URL scheme, cache layout).

use std::path::{Path, PathBuf};

const CHROME_VERSION: &str = "150.0.7871.46";
const DEFAULT_MIRROR: &str = "https://storage.googleapis.com/chrome-for-testing-public";

fn main() {
    println!("cargo:rerun-if-env-changed=TAIRITSU_CHROME_VERSION");
    println!("cargo:rerun-if-env-changed=TAIRITSU_CHROME_MIRROR");
    println!("cargo:rerun-if-env-changed=TAIRITSU_SKIP_BROWSER_FETCH");
    println!("cargo:rerun-if-env-changed=CHROME_PATH");

    if std::env::var_os("CARGO_FEATURE_AUTO_FETCH").is_none() {
        return;
    }

    // Flavor is mutually exclusive (lib.rs enforces it via compile_error!);
    // default is shell.
    let full = std::env::var_os("CARGO_FEATURE_FULL").is_some();
    let shell = std::env::var_os("CARGO_FEATURE_SHELL").is_some();
    if full && shell {
        panic!("tairitsu-browser-fetch: features `shell` and `full` are mutually exclusive");
    }
    let flavor = if full { "full" } else { "shell" };

    if std::env::var_os("TAIRITSU_SKIP_BROWSER_FETCH").is_some() {
        eprintln!(
            "[tairitsu-browser-fetch] TAIRITSU_SKIP_BROWSER_FETCH set; skipping build-time fetch"
        );
        return;
    }

    let ver =
        std::env::var("TAIRITSU_CHROME_VERSION").unwrap_or_else(|_| CHROME_VERSION.to_string());
    let Some(id) = target_download_id() else {
        eprintln!(
            "[tairitsu-browser-fetch] unsupported target; skipping build-time fetch"
        );
        return;
    };

    let exec = installed_executable(flavor, &ver, id);
    let final_path = if exec.exists() {
        exec
    } else {
        match download(flavor, &ver, id) {
            Ok(p) => p,
            Err(e) => {
                // Non-fatal: fall back to runtime / system resolution.
                eprintln!(
                    "[tairitsu-browser-fetch] build-time download failed ({e}); \
                     falling back to runtime resolution"
                );
                return;
            }
        }
    };

    if let Some(s) = final_path.to_str() {
        println!("cargo:rustc-env=TAIRITSU_BROWSER_PATH={s}");
    }
    println!("cargo:rustc-env=TAIRITSU_BROWSER_VERSION={ver}");
}

/// Chrome-for-Testing download id for the *target* triple (not the host).
fn target_download_id() -> Option<&'static str> {
    let os = std::env::var("CARGO_CFG_TARGET_OS").ok()?;
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").ok()?;
    match (os.as_str(), arch.as_str()) {
        ("linux", "x86_64") => Some("linux64"),
        ("macos", "aarch64") => Some("mac-arm64"),
        ("macos", "x86_64") => Some("mac-x64"),
        ("windows", "x86_64" | "x86") => Some("win64"),
        _ => None,
    }
}

fn cache_root() -> PathBuf {
    cache_dir()
        .unwrap_or_else(|| std::env::temp_dir().join("tairitsu-cache"))
        .join("tairitsu")
        .join("browsers")
        .join("chromium")
}

/// `<cache>/.../chromium/<flavor>/<ver>/<id>` — the archive extracts *into* this.
fn version_dir(flavor: &str, ver: &str, id: &str) -> PathBuf {
    cache_root().join(flavor).join(ver).join(id)
}

/// Host cache directory (mirrors the `dirs` crate, inlined). The cache lives on
/// the build host even when cross-compiling.
fn cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Caches"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
    }
}

fn installed_executable(flavor: &str, ver: &str, id: &str) -> PathBuf {
    let (top, exec): (String, String) = match flavor {
        "shell" => {
            let name = if id == "win64" {
                "chrome-headless-shell.exe"
            } else {
                "chrome-headless-shell"
            };
            (format!("chrome-headless-shell-{id}"), name.to_string())
        }
        _ => {
            let exec = match id {
                "linux64" => "chrome",
                "win64" => "chrome.exe",
                _ => "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
            };
            (format!("chrome-{id}"), exec.to_string())
        }
    };
    cache_root()
        .join(flavor)
        .join(ver)
        .join(id)
        .join(top)
        .join(exec)
}

fn archive_url(flavor: &str, ver: &str, id: &str) -> String {
    let base =
        std::env::var("TAIRITSU_CHROME_MIRROR").unwrap_or_else(|_| DEFAULT_MIRROR.to_string());
    let stem = match flavor {
        "shell" => format!("chrome-headless-shell-{id}"),
        _ => format!("chrome-{id}"),
    };
    format!("{base}/{ver}/{id}/{stem}.zip")
}

/// Install the `ring` crypto provider as the process default. reqwest is built
/// with `rustls-no-provider`, so it has no TLS provider until this runs.
fn install_ring_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

fn download(flavor: &str, ver: &str, id: &str) -> anyhow::Result<PathBuf> {
    let url = archive_url(flavor, ver, id);
    let target = installed_executable(flavor, ver, id);
    // Extract *into* the version dir; the archive carries its own top dir
    // (`<archive_stem>/`), so the binary lands exactly at `target`.
    let dest = version_dir(flavor, ver, id);
    std::fs::create_dir_all(&dest)?;

    eprintln!("[tairitsu-browser-fetch] downloading {url} (once, then cached)");
    install_ring_provider();
    let bytes = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?
        .get(&url)
        .header("User-Agent", "tairitsu-browser-fetch")
        .send()?
        .error_for_status()?
        .bytes()?;

    extract_zip(&bytes[..], &dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&target) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&target, perms)?;
        }
    }

    eprintln!("[tairitsu-browser-fetch] installed to {}", dest.display());
    Ok(target)
}

fn extract_zip(bytes: &[u8], dest: &Path) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        // Strip traversal / absolute components from archive entry names.
        let mut safe = PathBuf::new();
        for comp in Path::new(&name).components() {
            use std::path::Component::*;
            if let Normal(c) = comp {
                safe.push(c);
            }
        }
        let path = dest.join(safe);
        if name.ends_with('/') {
            std::fs::create_dir_all(&path)?;
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&path)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}
