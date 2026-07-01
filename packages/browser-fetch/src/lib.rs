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
//! Pick **one** via cargo features (mutually exclusive):
//! - `shell` (default) — `chrome-headless-shell`, ~90 MB. Enough for headless
//!   scraping (seia's use case).
//! - `full` — full Chrome for Testing, ~300 MB. When you need full rendering.
//!
//! Knobs: `TAIRITSU_CHROME_VERSION`, `TAIRITSU_CHROME_MIRROR`,
//! `TAIRITSU_SKIP_BROWSER_FETCH`.

// ── mutually-exclusive flavor ───────────────────────────────────────────────
#[cfg(all(feature = "shell", feature = "full"))]
compile_error!(
    "tairitsu-browser-fetch: features `shell` and `full` are mutually exclusive; enable only one"
);
#[cfg(not(any(feature = "shell", feature = "full")))]
compile_error!(
    "tairitsu-browser-fetch: enable exactly one of `shell` or `full`"
);

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
    /// Selected via cargo features (`shell` default, `full` alternative).
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

    /// Executable path inside the *full-chrome* archive (unused for shell;
    /// kept for reference / parity with browser-test).
    #[allow(dead_code)]
    fn chrome_exec_relative(&self) -> PathBuf {
        match self {
            Platform::LinuxX64 => PathBuf::from("chrome-linux64/chrome"),
            Platform::MacosArm64 | Platform::MacosX64 => PathBuf::from(
                "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
            ),
            Platform::WindowsX64 => PathBuf::from("chrome-win64/chrome.exe"),
        }
    }

    pub fn detect() -> Self {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Platform::LinuxX64,
            ("macos", "aarch64") => Platform::MacosArm64,
            ("macos", "x86_64") => Platform::MacosX64,
            ("windows", "x86_64" | "x86") => Platform::WindowsX64,
            other => panic!("tairitsu-browser-fetch: unsupported platform {other:?}"),
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
}

/// Version-scoped dir: `<cache>/tairitsu/browsers/chromium/<flavor>/<ver>/<plat>`.
/// The archive extracts *into* this dir, producing `<archive_stem>/<binary>`.
pub fn version_dir(flavor: Flavor, ver: &str, plat: Platform) -> PathBuf {
    cache_root()
        .join(if flavor == Flavor::Shell { "shell" } else { "full" })
        .join(ver)
        .join(plat.download_id())
}

/// Where a given flavor/version/platform lands once extracted.
pub fn installed_path(flavor: Flavor, ver: &str, plat: Platform) -> PathBuf {
    version_dir(flavor, ver, plat).join(flavor.internal_relative(plat))
}

/// Download URL for the archive.
pub fn archive_url(flavor: Flavor, ver: &str, plat: Platform) -> String {
    let base = std::env::var("TAIRITSU_CHROME_MIRROR").unwrap_or_else(|_| DEFAULT_MIRROR.to_string());
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
pub fn resolve() -> anyhow::Result<PathBuf> {
    // 1. Explicit override.
    if let Ok(p) = std::env::var("CHROME_PATH") {
        if !p.is_empty() {
            return Ok(PathBuf::from(p));
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
    #[cfg(feature = "runtime-fetch")]
    {
        log("system chrome not found; fetching via runtime-fetch");
        return ensure();
    }

    #[cfg(not(feature = "runtime-fetch"))]
    {
        anyhow::bail!(
            "no chrome/chromium found. Set CHROME_PATH, install chromium on PATH, \
             or enable the `runtime-fetch` feature of tairitsu-browser-fetch."
        )
    }
}

/// Guarantee the pinned build is in the cache, downloading it if missing.
/// Returns its path. (Requires the `runtime-fetch` feature.)
#[cfg(feature = "runtime-fetch")]
pub fn ensure() -> anyhow::Result<PathBuf> {
    let flavor = Flavor::selected();
    let plat = Platform::detect();
    let ver = version();
    let target = installed_path(flavor, ver, plat);
    if target.exists() {
        return Ok(target);
    }
    download_to_cache(flavor, ver, plat)?;
    make_executable(&target)?;
    Ok(target)
}

#[cfg(feature = "runtime-fetch")]
fn download_to_cache(flavor: Flavor, ver: &str, plat: Platform) -> anyhow::Result<()> {
    let url = archive_url(flavor, ver, plat);
    // Extract *into* the version dir; the archive carries its own top dir
    // (`<archive_stem>/`), so the binary lands exactly at `installed_path`.
    let dest = version_dir(flavor, ver, plat);
    std::fs::create_dir_all(&dest)?;

    log(&format!("downloading {} (this happens once, then is cached)", url));
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
    log(&format!("installed {} to {}", flavor_name(flavor), dest.display()));
    Ok(())
}

#[cfg(feature = "runtime-fetch")]
fn extract_zip(bytes: &[u8], dest: &Path) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        // Guard against path traversal in archive entries.
        let path = dest.join(sanitize_extract_name(&name));
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

#[cfg(all(unix, feature = "runtime-fetch"))]
fn make_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}
#[cfg(all(not(unix), feature = "runtime-fetch"))]
fn make_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

/// Locate a system Chrome on `$PATH` (best-effort, no deps).
fn which_system_chrome() -> Option<PathBuf> {
    const CANDIDATES: &[&str] = &[
        "chromium-browser",
        "chromium",
        "google-chrome",
        "google-chrome-stable",
        "chrome",
    ];
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in CANDIDATES {
            let candidate = dir.join(name);
            if let Ok(meta) = std::fs::metadata(&candidate) {
                if meta.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
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
}
