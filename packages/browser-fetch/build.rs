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
    println!("cargo:rerun-if-env-changed=TAIRITSU_CHROME_SHA256");
    println!("cargo:rerun-if-env-changed=TAIRITSU_SKIP_BROWSER_FETCH");
    println!("cargo:rerun-if-env-changed=CHROME_PATH");
    // Cache-location vars (cache_dir() reads these); a change must re-bake.
    println!("cargo:rerun-if-env-changed=HOME");
    println!("cargo:rerun-if-env-changed=XDG_CACHE_HOME");
    println!("cargo:rerun-if-env-changed=LOCALAPPDATA");

    if std::env::var_os("CARGO_FEATURE_AUTO_FETCH").is_none() {
        return;
    }

    // Flavor: shell is the implicit default; `full` opts into full Chrome.
    let flavor = if std::env::var_os("CARGO_FEATURE_FULL").is_some() {
        "full"
    } else {
        "shell"
    };

    if std::env::var_os("TAIRITSU_SKIP_BROWSER_FETCH").is_some() {
        eprintln!(
            "[tairitsu-browser-fetch] TAIRITSU_SKIP_BROWSER_FETCH set; skipping build-time fetch"
        );
        return;
    }

    let ver =
        std::env::var("TAIRITSU_CHROME_VERSION").unwrap_or_else(|_| CHROME_VERSION.to_string());
    let Some(id) = target_download_id() else {
        eprintln!("[tairitsu-browser-fetch] unsupported target; skipping build-time fetch");
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

    // Propagate the resolved version unconditionally so lib.rs `version()`
    // (option_env!) and the build-time download stay in lock-step even if the
    // cache path itself isn't valid UTF-8.
    println!("cargo:rustc-env=TAIRITSU_CHROME_VERSION={ver}");
    if let Some(s) = final_path.to_str() {
        println!("cargo:rustc-env=TAIRITSU_BROWSER_PATH={s}");
    } else {
        eprintln!(
            "[tairitsu-browser-fetch] cache path is not valid UTF-8; \
             TAIRITSU_BROWSER_PATH not emitted"
        );
    }
}

/// Chrome-for-Testing download id for the *target* triple (not the host).
fn target_download_id() -> Option<&'static str> {
    let os = std::env::var("CARGO_CFG_TARGET_OS").ok()?;
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").ok()?;
    match (os.as_str(), arch.as_str()) {
        ("linux", "x86_64") => Some("linux64"),
        ("macos", "aarch64") => Some("mac-arm64"),
        ("macos", "x86_64") => Some("mac-x64"),
        // Chrome for Testing only ships a win64 build; 32-bit Windows has no match.
        ("windows", "x86_64") => Some("win64"),
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
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        None
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
    let raw =
        std::env::var("TAIRITSU_CHROME_MIRROR").unwrap_or_else(|_| DEFAULT_MIRROR.to_string());
    let base = raw.trim_end_matches('/'); // avoid `//` if the mirror ends with `/`
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
    let dest = version_dir(flavor, ver, id);
    // Extract into a sibling temp dir, then swap into place, so a
    // partial/interrupted extraction never poisons the cache. The nonce makes
    // the temp path unique per call within a process.
    let parent = dest.parent().unwrap_or(Path::new("."));
    // Best-effort: reap stale temp dirs left by a previous crashed run (>1h old).
    sweep_stale_temps(parent, std::time::Duration::from_secs(3600));
    let nonce = TMP_NONCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = parent.join(format!(".{id}-{}-{nonce}.tmp", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);

    let result = download_inner(&url, &tmp, &dest);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&tmp);
        return result.map(|()| target);
    }
    // Post-extract integrity: the expected binary must actually exist, else the
    // archive was truncated/malformed and we must not bake a dangling path.
    if !target.exists() {
        let _ = std::fs::remove_dir_all(&dest);
        anyhow::bail!(
            "extraction completed but the expected binary {} is missing",
            target.display()
        );
    }
    Ok(target)
}

fn download_inner(url: &str, tmp: &Path, dest: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(tmp)?;
    eprintln!("[tairitsu-browser-fetch] downloading {url} (once, then cached)");
    let bytes = fetch_with_retry(url)?;

    extract_zip(&bytes, tmp)?;

    // Swap into place. On unix `rename` atomically replaces an existing dest;
    // on Windows rename fails if dest exists, so remove-then-rename there.
    if std::fs::rename(tmp, dest).is_err() {
        let _ = std::fs::remove_dir_all(dest);
        std::fs::rename(tmp, dest)
            .map_err(|e| anyhow::anyhow!("failed to finalize browser cache (rename): {e}"))?;
    }
    eprintln!("[tairitsu-browser-fetch] installed to {}", dest.display());
    Ok(())
}

/// Fetch `url` with up to 3 attempts (exponential backoff), then optionally
/// verify the SHA-256 when `TAIRITSU_CHROME_SHA256` (hex) is set.
fn fetch_with_retry(url: &str) -> anyhow::Result<Vec<u8>> {
    install_ring_provider();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=3 {
        let outcome = client
            .get(url)
            .header("User-Agent", "tairitsu-browser-fetch")
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.bytes());
        match outcome {
            Ok(b) => {
                let bytes = b.to_vec();
                // Checksum mismatch is deterministic — fail fast, don't retry.
                return verify_checksum(&bytes)
                    .map_err(|e| {
                        eprintln!("[tairitsu-browser-fetch] checksum failed (not retrying): {e}");
                        e
                    })
                    .map(|()| bytes);
            }
            Err(e) => {
                eprintln!("[tairitsu-browser-fetch] attempt {attempt}: {e}");
                last_err = Some(e.into());
            }
        }
        if attempt < 3 {
            std::thread::sleep(std::time::Duration::from_secs(1u64 << attempt));
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("download failed after 3 attempts")))
}

/// If `TAIRITSU_CHROME_SHA256` (lowercase hex) is set, verify the body against
/// it; otherwise no-op.
fn verify_checksum(bytes: &[u8]) -> anyhow::Result<()> {
    let Some(expected) = std::env::var("TAIRITSU_CHROME_SHA256")
        .ok()
        .filter(|s| !s.is_empty())
    else {
        return Ok(());
    };
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let actual: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    if actual != expected.trim().to_lowercase() {
        anyhow::bail!("checksum mismatch: expected {expected}, got {actual}");
    }
    eprintln!("[tairitsu-browser-fetch] checksum verified");
    Ok(())
}

/// Best-effort sweep of stale temp dirs (from crashed runs) under `parent`,
/// older than `max_age`. Ignores errors. Live downloads have fresh mtimes.
#[allow(dead_code)]
fn sweep_stale_temps(parent: &Path, max_age: std::time::Duration) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    let cutoff = std::time::SystemTime::now() - max_age;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with('.') || !name.ends_with(".tmp") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if meta.is_dir() {
                if let Ok(mtime) = meta.modified() {
                    if mtime < cutoff {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        }
    }
}

/// Per-call nonce making each download's temp dir unique.
static TMP_NONCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn extract_zip(bytes: &[u8], dest: &Path) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        // Capture unix mode before the mutable borrow in io::copy; unused off-unix.
        #[cfg(unix)]
        let unix_mode = entry.unix_mode();
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
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Recreate symlinks (macOS .app bundles ship internal symlinks).
        #[cfg(unix)]
        let is_symlink = unix_mode.is_some_and(|m| (m & 0o170000) == 0o120000);
        #[cfg(not(unix))]
        let is_symlink = false;
        if is_symlink {
            #[cfg(unix)]
            {
                use std::io::Read;
                let mut target = String::new();
                let _ = entry.read_to_string(&mut target);
                let target = target.trim();
                if !target.is_empty() && !target.starts_with('/') && !target.contains("..") {
                    let _ = std::os::unix::fs::symlink(target, &path);
                }
            }
        } else {
            let mut out = std::fs::File::create(&path)?;
            std::io::copy(&mut entry, &mut out)?;
            drop(out);
            // Restore unix permission bits so executables keep their exec bit.
            #[cfg(unix)]
            if let Some(mode) = unix_mode {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode));
            }
        }
    }
    Ok(())
}
