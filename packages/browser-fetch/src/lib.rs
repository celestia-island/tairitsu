//! Zero-config Chromium resolver for Tairitsu.
//!
//! Mirrors [ort]'s dependency-resolution model: a pinned Chrome for Testing
//! build is fetched into a shared cache and located transparently, so a
//! consumer never has to install Chrome by hand.
//!
//! [ort]: https://crates.io/crates/ort
//!
//! # Resolution order
//! 1. `$CHROME_PATH` — explicit override, always wins.
//! 2. The build-time baked path (`TAIRITSU_BROWSER_PATH`, set by `build.rs`
//!    when the `auto-fetch` feature downloads Chrome during the build).
//! 3. A system Chrome on `$PATH` (`chromium-browser` / `google-chrome` / …).
//! 4. Runtime fetch (`runtime-fetch` feature): download the pinned build into
//!    the cache now and use it.
//! 5. Error.
//!
//! # Binary flavor
//! Default = `chrome-headless-shell` (~90 MB; enough for headless scraping).
//! Enable the `full` cargo feature for full Chrome for Testing (~300 MB, when
//! you need full rendering). This is a single toggle, so you always get exactly
//! one flavor.
//!
//! Knobs: `TAIRITSU_CHROME_VERSION`, `TAIRITSU_CHROME_MIRROR`,
//! `TAIRITSU_CHROME_SHA256` (optional integrity check, hex),
//! `TAIRITSU_SKIP_BROWSER_FETCH` (skips both build-time and runtime download),
//! `CHROME_PATH` (explicit executable override).

use std::path::{Path, PathBuf};

/// Pinned Chrome for Testing version (Stable channel). Bump per release, like
/// ort pins an ONNX Runtime version per crate release. Override at build time
/// with `TAIRITSU_CHROME_VERSION`.
pub const CHROME_VERSION: &str = "150.0.7871.46";

const DEFAULT_MIRROR: &str = "https://storage.googleapis.com/chrome-for-testing-public";

/// Which Chrome for Testing binary to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    /// `chrome-headless-shell` — small, headless-only.
    Shell,
    /// Full `chrome` — headed-capable, full feature set.
    Full,
}

impl Flavor {
    /// Shell by default; `Full` when the `full` cargo feature is enabled.
    pub fn selected() -> Self {
        #[cfg(feature = "full")]
        {
            Flavor::Full
        }
        #[cfg(not(feature = "full"))]
        {
            Flavor::Shell
        }
    }

    /// Archive stem without extension, e.g. `chrome-headless-shell-linux64`.
    fn archive_stem(&self, plat: Platform) -> String {
        match self {
            Flavor::Shell => format!("chrome-headless-shell-{}", plat.download_id()),
            Flavor::Full => format!("chrome-{}", plat.download_id()),
        }
    }

    /// Path of the executable *relative to the version dir* after extraction.
    /// The archive extracts as `<archive_stem>/<binary>`, so the version dir is
    /// the parent of `archive_stem`.
    fn internal_relative(&self, plat: Platform) -> PathBuf {
        let stem = self.archive_stem(plat);
        match self {
            Flavor::Shell => {
                let name = if plat.is_windows() {
                    "chrome-headless-shell.exe"
                } else {
                    "chrome-headless-shell"
                };
                Path::new(&stem).join(name)
            }
            Flavor::Full => {
                let under = match plat {
                    Platform::LinuxX64 => "chrome",
                    Platform::WindowsX64 => "chrome.exe",
                    Platform::MacosArm64 | Platform::MacosX64 => {
                        "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
                    }
                };
                Path::new(&stem).join(under)
            }
        }
    }
}

/// Target platform for Chrome for Testing downloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxX64,
    MacosArm64,
    MacosX64,
    WindowsX64,
}

impl Platform {
    pub fn download_id(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "linux64",
            Platform::MacosArm64 => "mac-arm64",
            Platform::MacosX64 => "mac-x64",
            Platform::WindowsX64 => "win64",
        }
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::WindowsX64)
    }

    /// Detect the current runtime platform. Returns `None` for unsupported
    /// triples (e.g. linux aarch64, 32-bit) instead of panicking.
    pub fn detect() -> Option<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Some(Platform::LinuxX64),
            ("macos", "aarch64") => Some(Platform::MacosArm64),
            ("macos", "x86_64") => Some(Platform::MacosX64),
            ("windows", "x86_64") => Some(Platform::WindowsX64),
            _ => None,
        }
    }
}

/// The effective Chrome version (const overridden by env at build time).
pub fn version() -> &'static str {
    option_env!("TAIRITSU_CHROME_VERSION").unwrap_or(CHROME_VERSION)
}

/// Shared cache root: `<cache>/tairitsu/browsers/chromium`.
pub fn cache_root() -> PathBuf {
    cache_dir()
        .unwrap_or_else(|| std::env::temp_dir().join("tairitsu-cache"))
        .join("tairitsu")
        .join("browsers")
        .join("chromium")
}

/// Platform cache directory (mirrors the `dirs` crate, inlined to avoid the
/// dependency when no download feature is enabled).
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

/// Version-scoped dir: `<cache>/tairitsu/browsers/chromium/<flavor>/<ver>/<plat>`.
/// The archive extracts *into* this dir, producing `<archive_stem>/<binary>`.
pub fn version_dir(flavor: Flavor, ver: &str, plat: Platform) -> PathBuf {
    cache_root()
        .join(if flavor == Flavor::Shell {
            "shell"
        } else {
            "full"
        })
        .join(ver)
        .join(plat.download_id())
}

/// Where a given flavor/version/platform lands once extracted.
pub fn installed_path(flavor: Flavor, ver: &str, plat: Platform) -> PathBuf {
    version_dir(flavor, ver, plat).join(flavor.internal_relative(plat))
}

/// Download URL for the archive.
pub fn archive_url(flavor: Flavor, ver: &str, plat: Platform) -> String {
    let raw =
        std::env::var("TAIRITSU_CHROME_MIRROR").unwrap_or_else(|_| DEFAULT_MIRROR.to_string());
    // Trim a trailing slash so a user-supplied mirror doesn't yield `//`.
    let base = raw.trim_end_matches('/');
    format!(
        "{}/{}/{}/{}.zip",
        base,
        ver,
        plat.download_id(),
        flavor.archive_stem(plat)
    )
}

/// Resolve a Chrome executable, trying every source in order.
///
/// Returns the path to an existing executable. See the crate docs for the
/// resolution order.
///
/// **Blocking:** this may perform a multi-second HTTP download + zip
/// extraction (the `runtime-fetch` fallback). It must NOT be called directly
/// on a tokio/async worker thread — `reqwest::blocking` panics inside an
/// active runtime. Wrap it in `tokio::task::spawn_blocking` from async code
/// (the packager debug server already does this).
pub fn resolve() -> anyhow::Result<PathBuf> {
    // 1. Explicit override. If set, it's authoritative — a missing/typo'd path
    //    is an explicit error rather than a silent fall-through.
    if let Ok(p) = std::env::var("CHROME_PATH") {
        if !p.is_empty() {
            let path = PathBuf::from(&p);
            if path.exists() {
                return Ok(path);
            }
            anyhow::bail!("CHROME_PATH is set to {:?} but it does not exist", path);
        }
    }

    // 2. Build-time baked path (set by build.rs under `auto-fetch`).
    if let Some(p) = option_env!("TAIRITSU_BROWSER_PATH") {
        if !p.is_empty() && Path::new(p).exists() {
            return Ok(PathBuf::from(p));
        }
    }

    // 3. System Chrome on PATH.
    if let Some(p) = which_system_chrome() {
        return Ok(p);
    }

    // 4. Runtime fallback fetch.
    runtime_fallback()
}

#[cfg(feature = "runtime-fetch")]
fn runtime_fallback() -> anyhow::Result<PathBuf> {
    if std::env::var_os("TAIRITSU_SKIP_BROWSER_FETCH").is_some() {
        anyhow::bail!(
            "no chrome/chromium found and TAIRITSU_SKIP_BROWSER_FETCH is set; \
             unset it or provide CHROME_PATH"
        );
    }
    log("system chrome not found; fetching via runtime-fetch");
    ensure()
}

#[cfg(not(feature = "runtime-fetch"))]
fn runtime_fallback() -> anyhow::Result<PathBuf> {
    anyhow::bail!(
        "no chrome/chromium found. Set CHROME_PATH, install chromium on PATH, \
         or enable the `runtime-fetch` feature of tairitsu-browser-fetch."
    )
}

/// Guarantee the pinned build is in the cache, downloading it if missing.
/// Returns its path. (Requires the `runtime-fetch` feature.)
///
/// Note: this performs blocking I/O (HTTP download + extraction). When called
/// from an async runtime, wrap it in `tokio::task::spawn_blocking`.
#[cfg(feature = "runtime-fetch")]
pub fn ensure() -> anyhow::Result<PathBuf> {
    let flavor = Flavor::selected();
    let plat = Platform::detect().ok_or_else(|| {
        anyhow::anyhow!(
            "unsupported runtime platform ({}-{}); Chrome for Testing only ships for \
             linux-x86_64, macos-{{aarch64,x86_64}}, windows-x86_64",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let ver = version();
    let target = installed_path(flavor, ver, plat);
    if target.exists() {
        return Ok(target);
    }
    download_to_cache(flavor, ver, plat)?;
    // Final integrity check: the expected binary must exist after extraction.
    if !target.exists() {
        anyhow::bail!(
            "extraction completed but {} not found at {}",
            flavor_name(flavor),
            target.display()
        );
    }
    Ok(target)
}

#[cfg(feature = "runtime-fetch")]
fn download_to_cache(flavor: Flavor, ver: &str, plat: Platform) -> anyhow::Result<()> {
    let url = archive_url(flavor, ver, plat);
    // Extract to a sibling temp dir first, then swap into the version dir, so a
    // partial/interrupted extraction never poisons the cache.
    let dest = version_dir(flavor, ver, plat);
    let parent = dest.parent().unwrap_or(Path::new("."));
    // Best-effort: reap stale temp dirs left by a previous crashed run (>1h old).
    sweep_stale_temps(parent, std::time::Duration::from_secs(3600));
    // Unique per (platform, process, call): the PID + a process-local counter,
    // so two concurrent ensure() calls in the SAME process can't collide.
    let nonce = TMP_NONCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = parent.join(format!(
        ".{}-{}-{}.tmp",
        plat.download_id(),
        std::process::id(),
        nonce
    ));
    // Best-effort: reap stale temp dirs left by a crashed previous run.
    let _ = std::fs::remove_dir_all(&tmp);

    let result = download_to_cache_inner(&url, &tmp, &dest, flavor);
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&tmp);
    }
    result
}

#[cfg(feature = "runtime-fetch")]
fn download_to_cache_inner(
    url: &str,
    tmp: &Path,
    dest: &Path,
    flavor: Flavor,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(tmp)?;
    log(&format!(
        "downloading {} (this happens once, then is cached)",
        url
    ));
    let bytes = fetch_with_retry(url)?;

    extract_zip(&bytes, tmp)?;

    // Swap into place. On unix `rename` atomically replaces an existing dest;
    // on Windows rename fails if dest exists, so remove-then-rename there.
    if std::fs::rename(tmp, dest).is_err() {
        let _ = std::fs::remove_dir_all(dest);
        std::fs::rename(tmp, dest)
            .map_err(|e| anyhow::anyhow!("failed to finalize browser cache (rename): {e}"))?;
    }
    log(&format!(
        "installed {} to {}",
        flavor_name(flavor),
        dest.display()
    ));
    Ok(())
}

/// Fetch `url` with up to 3 attempts (exponential backoff), then optionally
/// verify the SHA-256 when `TAIRITSU_CHROME_SHA256` (hex) is set.
#[cfg(feature = "runtime-fetch")]
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
                // Checksum mismatch is deterministic (same URL = same bytes) —
                // fail immediately instead of wastefully re-downloading 90-300 MB.
                return verify_checksum(&bytes)
                    .map_err(|e| {
                        log(&format!("checksum failed (not retrying): {e}"));
                        e
                    })
                    .map(|()| bytes);
            }
            Err(e) => {
                log(&format!("attempt {attempt}: {e}"));
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
/// it; otherwise no-op (the zip central-directory check + post-extract
/// existence check still catch most corruption).
#[cfg(feature = "runtime-fetch")]
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
    log("checksum verified");
    Ok(())
}

#[cfg(feature = "runtime-fetch")]
fn extract_zip(bytes: &[u8], dest: &Path) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        // Capture the entry's unix mode (if any) before the entry is mutably
        // borrowed by io::copy below. Gated to unix — the value is unused off-unix.
        #[cfg(unix)]
        let unix_mode = entry.unix_mode();
        // Guard against path traversal in archive entries.
        let path = dest.join(sanitize_extract_name(&name));
        if name.ends_with('/') {
            std::fs::create_dir_all(&path)?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // macOS .app bundles ship internal symlinks (Frameworks → versioned
        // dirs); recreate them instead of flattening to text files.
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
                // Defense in depth: only relative, in-bundle targets.
                if !target.is_empty() && !target.starts_with('/') && !target.contains("..") {
                    let _ = std::os::unix::fs::symlink(target, &path);
                }
            }
        } else {
            let mut out = std::fs::File::create(&path)?;
            std::io::copy(&mut entry, &mut out)?;
            drop(out);
            // Restore unix permission bits from the archive so executables
            // (chrome, crashpad_handler, …) keep their exec bit.
            #[cfg(unix)]
            if let Some(mode) = unix_mode {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "runtime-fetch")]
fn sanitize_extract_name(name: &str) -> PathBuf {
    // Strip any leading slashes / `..` components from archive entry names.
    let mut out = PathBuf::new();
    for comp in Path::new(name).components() {
        use std::path::Component::*;
        match comp {
            Normal(c) => out.push(c),
            CurDir => {}
            Prefix(_) | RootDir | ParentDir => {}
        }
    }
    out
}

/// Locate a system Chrome. Searches `$PATH` first, then a few well-known
/// install locations (so a Chrome in `/Applications` or `Program Files` that
/// isn't on `$PATH` is still found). Best-effort, no deps.
fn which_system_chrome() -> Option<PathBuf> {
    const CANDIDATES: &[&str] = &[
        "chromium-browser",
        "chromium",
        "google-chrome",
        "google-chrome-stable",
        "chrome",
    ];
    let path_var = std::env::var_os("PATH")?;
    // On Windows, also try each candidate with a `.exe` suffix (Chrome ships as
    // chrome.exe; bare names rarely resolve on NTFS without an extension).
    let try_names: Vec<String> = if cfg!(windows) {
        let mut v: Vec<String> = CANDIDATES.iter().map(|s| format!("{s}.exe")).collect();
        v.extend(CANDIDATES.iter().map(|s| s.to_string()));
        v
    } else {
        CANDIDATES.iter().map(|s| s.to_string()).collect()
    };
    for dir in std::env::split_paths(&path_var) {
        for name in &try_names {
            let candidate = dir.join(name);
            if is_executable_file(&candidate) {
                return Some(candidate);
            }
        }
    }

    // Well-known install locations not necessarily on $PATH.
    const COMMON: &[&str] = &[
        // macOS
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        // Linux
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
        // Windows
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];
    for p in COMMON {
        let candidate = PathBuf::from(p);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file())
        .unwrap_or(false)
}

#[cfg(feature = "runtime-fetch")]
fn flavor_name(f: Flavor) -> &'static str {
    match f {
        Flavor::Shell => "chrome-headless-shell",
        Flavor::Full => "chrome",
    }
}

#[cfg(feature = "runtime-fetch")]
fn log(msg: &str) {
    eprintln!("[tairitsu-browser-fetch] {msg}");
}

/// Install the `ring` crypto provider as the process default. reqwest is built
/// with `rustls-no-provider`, so it has no TLS provider until this runs.
/// Idempotent: a second call is a no-op (install_default returns Err).
#[cfg(feature = "runtime-fetch")]
fn install_ring_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Per-process counter making each download's temp dir unique, so concurrent
/// `ensure()` calls (same PID) can't collide on the temp path.
#[cfg(feature = "runtime-fetch")]
static TMP_NONCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Best-effort sweep of stale temp dirs (from crashed runs) under `parent`,
/// older than `max_age`. Live downloads have fresh mtimes and are left alone.
#[cfg(feature = "runtime-fetch")]
fn sweep_stale_temps(parent: &Path, max_age: std::time::Duration) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    let cutoff = std::time::SystemTime::now() - max_age;
    for entry in entries.flatten() {
        let fname = entry.file_name();
        let Some(name) = fname.to_str() else {
            continue;
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_url_shape() {
        let url = archive_url(Flavor::Shell, "1.2.3", Platform::LinuxX64);
        assert_eq!(
            url,
            "https://storage.googleapis.com/chrome-for-testing-public/1.2.3/linux64/chrome-headless-shell-linux64.zip"
        );
    }

    #[test]
    fn full_url_shape() {
        let url = archive_url(Flavor::Full, "1.2.3", Platform::WindowsX64);
        assert_eq!(
            url,
            "https://storage.googleapis.com/chrome-for-testing-public/1.2.3/win64/chrome-win64.zip"
        );
    }

    #[test]
    fn installed_path_is_flavor_scoped() {
        let p = installed_path(Flavor::Shell, "1", Platform::LinuxX64);
        assert!(p.ends_with("shell/1/linux64/chrome-headless-shell-linux64/chrome-headless-shell"));
        let p = installed_path(Flavor::Full, "1", Platform::LinuxX64);
        assert!(p.ends_with("full/1/linux64/chrome-linux64/chrome"));
    }

    #[cfg(feature = "runtime-fetch")]
    #[test]
    fn sanitize_strips_traversal() {
        let p = sanitize_extract_name("../../etc/passwd");
        assert_eq!(p, PathBuf::from("etc/passwd"));
    }

    #[cfg(feature = "runtime-fetch")]
    #[test]
    fn checksum_passes_when_matching() {
        use sha2::{Digest, Sha256};
        let data = b"hello";
        let mut h = Sha256::new();
        h.update(data);
        let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
        std::env::set_var("TAIRITSU_CHROME_SHA256", &hex);
        assert!(verify_checksum(data).is_ok());
        std::env::set_var("TAIRITSU_CHROME_SHA256", "deadbeef");
        assert!(verify_checksum(data).is_err());
        std::env::remove_var("TAIRITSU_CHROME_SHA256");
        assert!(verify_checksum(data).is_ok());
    }

    #[test]
    fn resolve_errors_on_missing_chrome_path() {
        std::env::set_var("CHROME_PATH", "/nonexistent/chrome/that/does/not/exist");
        let r = resolve();
        assert!(r.is_err());
        std::env::remove_var("CHROME_PATH");
    }
}
