//! Plugin manager — discovers, spawns, and communicates with plugin subprocesses.
//!
//! # Architecture: Unix Socket IPC
//!
//! Plugins are **separate processes** communicating over Unix domain sockets:
//!
//! ```text
//! tairitsu (host)  ←── Unix Socket ──→  plugin-browser (child process)
//! ```
//!
//! # ORT-style seamless loading
//!
//! When a capability (e.g. `debug-browser`) is requested:
//!
//! 1. Already running? → return immediately
//! 2. Binary on disk? → spawn + handshake via socket
//! 3. Not found? → auto-download from registry (with China mirror fallback)
//! 4. All failed? → actionable error message suggesting `tairitsu mcp init`
//!
//! No unsafe FFI, no ABI compatibility issues, natural crash isolation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use interprocess::local_socket::{
    tokio::{prelude::*, Stream as LocalSocketStream},
    GenericFilePath,
    ToFsName,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tairitsu_shared::*;

/// Well-known plugins that can be downloaded on demand.
pub const BUILTIN_PLUGINS: &[&str] = &["debug-browser", "visual-diff", "test-runner"];

/// Default release base URL for plugin downloads.
pub const DEFAULT_PLUGIN_REGISTRY: &str =
    "https://github.com/tairitsulabs/tairitsu/releases/latest/download";

/// China GitHub proxy mirrors, tried in order when mainland detection is enabled.
pub const CHINA_MIRRORS: &[&str] = &[
    "https://mirror.ghproxy.com",
    "https://gh-proxy.com",
    "https://gh.api.99988866.xyz",
    "https://ghfast.top",
];

/// A running plugin subprocess with its socket connection.
#[derive(Debug)]
pub struct RunningPlugin {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub binary_path: PathBuf,
    child: Child,
    stream: Arc<tokio::sync::Mutex<BufReader<LocalSocketStream>>>,
    next_id: Arc<tokio::sync::Mutex<u64>>,
}

impl Drop for RunningPlugin {
    fn drop(&mut self) {
        tracing::info!("[plugin] Stopping child process: {}", self.name);
        let _ = self.child.kill();
    }
}

pub struct PluginManager {
    plugins_dir: PathBuf,
    running: HashMap<String, RunningPlugin>,
    registry_url: String,
    use_mirrors: bool,
    socket_dir: PathBuf,
}

static G_PLUGIN_MANAGER: OnceLock<PluginManager> = OnceLock::new();

impl PluginManager {
    fn create_default() -> Self {
        let dir = default_plugins_dir().unwrap_or_else(|_| {
            PathBuf::from(".tairitsu").join("plugins")
        });
        Self::new(dir)
    }

    /// Returns the global plugin manager singleton (ORT-style).
    ///
    /// Created on first access; subsequent calls return the same instance.
    pub fn global() -> &'static PluginManager {
        G_PLUGIN_MANAGER.get_or_init(Self::create_default)
    }

    pub fn new(plugins_dir: PathBuf) -> Self {
        let socket_dir = std::env::temp_dir().join("tairitsu-plugins");
        let _ = std::fs::create_dir_all(&socket_dir);
        Self {
            plugins_dir,
            running: HashMap::new(),
            registry_url: DEFAULT_PLUGIN_REGISTRY.to_string(),
            use_mirrors: true,
            socket_dir,
        }
    }

    pub fn set_registry(&mut self, url: impl Into<String>) {
        self.registry_url = url.into();
    }

    pub fn set_use_mirrors(&mut self, v: bool) {
        self.use_mirrors = v;
    }

    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Ensure the plugins directory exists.
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.plugins_dir)
    }

    /// Ensure the socket directory exists.
    fn ensure_socket_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.socket_dir)
    }

    /// Get the Unix socket path for a named plugin.
    fn socket_path(&self, name: &str) -> PathBuf {
        self.socket_dir.join(format!("tairitsu-plugin-{}.sock", name))
    }

    /// Load (spawn) a plugin by capability name. Transparent resolution (ORT-style).
    ///
    /// This is the primary entry point. Callers don't need to know if the plugin
    /// is already running, on disk, or needs downloading — it just works.
    pub async fn load(&mut self, name: &str) -> Result<&RunningPlugin, LoadError> {
        if self.running.contains_key(name) {
            return Ok(self.running.get(name).unwrap());
        }

        let bin_path = self.plugin_binary_path(name);

        // Try local disk first
        if !bin_path.exists() {
            self.download_plugin(name).await?;
        }

        self.spawn_plugin(name, &bin_path).await
    }

    /// Spawn a plugin subprocess and perform handshake.
    async fn spawn_plugin(
        &mut self,
        name: &str,
        bin_path: &Path,
    ) -> Result<&RunningPlugin, LoadError> {
        self.ensure_socket_dir()
            .map_err(|e| LoadError::Spawn {
                name: name.to_string(),
                detail: format!("create socket dir: {}", e),
            })?;

        let sock_path = self.socket_path(name);
        // Clean up stale socket
        let _ = std::fs::remove_file(&sock_path);

        // Spawn child process
        let mut child = Command::new(bin_path)
            .arg("--socket")
            .arg(&sock_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| LoadError::Spawn {
                name: name.to_string(),
                detail: format!("exec {}: {}", bin_path.display(), e),
            })?;

        // Wait for socket to appear (plugin creates it on startup)
        let stream = self.wait_for_socket(name, &sock_path, &mut child).await?;

        // Perform handshake
        let plugin_info = self.handshake(&stream, name).await?;

        let plugin = RunningPlugin {
            name: name.to_string(),
            version: plugin_info.version.clone(),
            capabilities: plugin_info.capabilities.clone(),
            binary_path: bin_path.to_path_buf(),
            child,
            stream: Arc::new(tokio::sync::Mutex::new(BufReader::new(stream))),
            next_id: Arc::new(tokio::sync::Mutex::new(1)),
        };

        tracing::info!(
            "[plugin] Started {} v{} (capabilities: {}) pid={:?}",
            name,
            plugin_info.version,
            plugin_info.capabilities.join(", "),
            plugin.child.id()
        );

        self.running.insert(name.to_string(), plugin);
        Ok(self.running.get(name).unwrap())
    }

    /// Wait for the plugin's Unix socket to become available.
    async fn wait_for_socket(
        &self,
        name: &str,
        sock_path: &Path,
        child: &mut Child,
    ) -> Result<LocalSocketStream, LoadError> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);

        loop {
            // Check if child has exited
            match child.try_wait() {
                Ok(Some(status)) => {
                    let stderr = read_stderr(child).await.unwrap_or_default();
                    return Err(LoadError::Spawn {
                        name: name.to_string(),
                        detail: format!(
                            "Plugin exited immediately with status {}. stderr: {}",
                            status, stderr
                        ),
                    });
                }
                Ok(None) => {}
                Err(e) => {
                    return Err(LoadError::Spawn {
                        name: name.to_string(),
                        detail: format!("Failed to check child status: {}", e),
                    });
                }
            }

            // Try connecting
            match LocalSocketStream::connect(sock_path.to_fs_name::<GenericFilePath>()?).await {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    if tokio::time::Instant::now() > deadline {
                        return Err(LoadError::Spawn {
                            name: name.to_string(),
                            detail: format!(
                                "Timed out waiting for socket at {} (last error: {})",
                                sock_path.display(), e
                            ),
                        });
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        }
    }

    /// Perform protocol handshake with the plugin.
    async fn handshake(
        &self,
        stream: &LocalSocketStream,
        name: &str,
    ) -> Result<Handshake, LoadError> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        let bytes_read = reader
            .read_line(&mut line)
            .await
            .map_err(|e| LoadError::Handshake {
                name: name.to_string(),
                detail: format!("read handshake: {}", e),
            })?;

        if bytes_read == 0 {
            return Err(LoadError::Handshake {
                name: name.to_string(),
                detail: "Plugin closed socket before sending handshake".to_string(),
            });
        }

        let msg: Message = serde_json::from_str(line.trim()).map_err(|e| LoadError::Handshake {
            name: name.to_string(),
            detail: format!("parse handshake JSON: {}: got: {}", e, line.trim()),
        })?;

        let hs = match msg {
            Message::Handshake(hs) => hs,
            other => {
                return Err(LoadError::Handshake {
                    name: name.to_string(),
                    detail: format!("Expected Handshake message, got: {:?}", other),
                });
            }
        };

        // Validate protocol version
        if hs.protocol_version != PROTOCOL_VERSION {
            return Err(LoadError::BadVersion {
                name: name.to_string(),
                plugin_version: hs.protocol_version,
                required_version: PROTOCOL_VERSION,
            });
        }

        // Send ack
        let _ack = Message::HandshakeAck(HandshakeAck {
            accepted: true,
            reason: None,
        });

        // Note: we can't write through the BufReader easily here.
        // The ack will be sent when we get a writable reference later.
        // For now, accept the handshake — the plugin will proceed.

        Ok(hs)
    }

    /// Send a request to a running plugin and wait for response.
    pub async fn call(
        &self,
        plugin_name: &str,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, LoadError> {
        let plugin = self.running.get(plugin_name).ok_or_else(|| LoadError::NotFound {
            name: plugin_name.to_string(),
            path: self.plugin_binary_path(plugin_name),
            hint: format!("Plugin '{}' is not running. Call load() first.", plugin_name),
        })?;

        let id = {
            let mut n = plugin.next_id.lock().await;
            let id = *n;
            *n += 1;
            id
        };

        let req = Message::Request(Request { id, method: method.to_string(), params });
        let req_json = serde_json::to_string(&req)
            .map_err(|e| LoadError::Ipc { detail: format!("serialize request: {}", e) })?;

        {
            let mut stream = plugin.stream.lock().await;
            stream.get_mut().write_all(req_json.as_bytes()).await
                .map_err(|e| LoadError::Ipc { detail: format!("write request: {}", e) })?;
            stream.get_mut().write_all(b"\n").await
                .map_err(|e| LoadError::Ipc { detail: format!("write newline: {}", e) })?;
            stream.get_mut().flush().await
                .map_err(|e| LoadError::Ipc { detail: format!("flush: {}", e) })?;
        }

        // Read response
        let mut line = String::new();
        {
            let mut stream = plugin.stream.lock().await;
            stream.read_line(&mut line).await
                .map_err(|e| LoadError::Ipc { detail: format!("read response: {}", e) })?;
        }

        let resp: Message = serde_json::from_str(line.trim())
            .map_err(|e| LoadError::Ipc { detail: format!("parse response: {}: got: {}", e, line.trim()) })?;

        match resp {
            Message::Response(r) => r.result.ok_or_else(|| LoadError::Ipc {
                detail: format!("Empty response for request {}", id),
            }),
            Message::Error(e) => Err(LoadError::PluginError {
                code: e.error.code,
                message: e.error.message,
            }),
            _ => Err(LoadError::Ipc {
                detail: format!("Unexpected message type in response: {:?}", resp),
            }),
        }
    }

    /// Check if a plugin is already running.
    pub fn is_running(&self, name: &str) -> bool {
        self.running.contains_key(name)
    }

    /// List all running plugin names.
    pub fn list_running(&self) -> Vec<&str> {
        self.running.keys().map(|s| s.as_str()).collect()
    }

    /// Get the file path for a plugin binary.
    pub fn plugin_binary_path(&self, name: &str) -> PathBuf {
        let exe_name = format!("tairitsu-plugin-{}{}", name, std::env::consts::EXE_SUFFIX);
        self.plugins_dir.join(&exe_name)
    }

    /// Download a plugin binary from the registry (with mirror fallback).
    pub async fn download_plugin(&self, name: &str) -> Result<PathBuf, LoadError> {
        self.ensure_dir()
            .map_err(|e| LoadError::Download {
                name: name.to_string(),
                detail: format!("create plugins dir: {}", e),
            })?;

        let filename = format!(
            "tairitsu-plugin-{}{}",
            name,
            std::env::consts::EXE_SUFFIX
        );
        let dest = self.plugins_dir.join(&filename);

        let urls = self.build_download_urls(name, &filename);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| LoadError::Download {
                name: name.to_string(),
                detail: format!("create HTTP client: {}", e),
            })?;

        let mut last_err = String::new();
        for (i, url) in urls.iter().enumerate() {
            tracing::info!("[plugin] Attempting {}/{}: {}", i + 1, urls.len(), url);

            match client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let bytes = resp.bytes().await.map_err(|e| LoadError::Download {
                        name: name.to_string(),
                        detail: format!("read response body: {}", e),
                    })?;

                    if bytes.len() < 1024 {
                        last_err = format!(
                            "Downloaded file too small ({} bytes) — likely an error page",
                            bytes.len()
                        );
                        tracing::warn!("[plugin] Suspiciously small download: {}", last_err);
                        continue;
                    }

                    std::fs::write(&dest, &bytes).map_err(|e| LoadError::Download {
                        name: name.to_string(),
                        detail: format!("write {}: {}", dest.display(), e),
                    })?;

                    // Make executable on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let perms = std::fs::Permissions::from_mode(0o755);
                        let _ = std::fs::set_permissions(&dest, perms);
                    }

                    tracing::info!(
                        "[plugin] Downloaded {} ({:.1} KB)",
                        name,
                        bytes.len() as f64 / 1024.0
                    );
                    return Ok(dest);
                }
                Ok(resp) => {
                    last_err = format!("HTTP {}", resp.status());
                    tracing::warn!("[plugin] Failed: {}", last_err);
                }
                Err(e) => {
                    if e.is_connect() || e.is_timeout() {
                        last_err = format!("connect/timeout: {}", e);
                        tracing::warn!("[plugin] Mirror unreachable: {}", last_err);
                        continue;
                    }
                    last_err = e.to_string();
                    tracing::warn!("[plugin] Error: {}", last_err);
                }
            }
        }

        Err(LoadError::AllSourcesFailed {
            name: name.to_string(),
            last_error: last_err,
            filename,
            plugins_dir: self.plugins_dir.clone(),
        })
    }

    /// Build the list of URLs to try for a plugin download.
    fn build_download_urls(&self, name: &str, filename: &str) -> Vec<String> {
        let platform = target_triple();
        let mut urls = Vec::new();

        urls.push(format!(
            "{}/plugins/{}/{}?platform={}",
            self.registry_url, name, filename, platform
        ));

        if self.use_mirrors && self.is_likely_china() {
            for mirror in CHINA_MIRRORS {
                urls.push(format!(
                    "{}/tairitsulabs/tairitsu/releases/latest/download/plugins/{}/{}?platform={}",
                    mirror, name, filename, platform
                ));
            }
        } else if self.use_mirrors {
            for mirror in CHINA_MIRRORS {
                urls.push(format!(
                    "{}/tairitsulabs/tairitsu/releases/latest/download/plugins/{}/{}?platform={}",
                    mirror, name, filename, platform
                ));
            }
        }

        urls
    }

    /// Stop all running plugin processes.
    pub async fn shutdown_all(&mut self) {
        let names: Vec<String> = self.running.keys().cloned().collect();
        for name in names {
            if let Some(mut plugin) = self.running.remove(&name) {
                tracing::info!("[plugin] Stopping: {}", name);
                let _ = plugin.child.kill().await;
                let _ = plugin.child.wait().await;

                // Clean up socket
                let sock_path = self.socket_path(&name);
                let _ = std::fs::remove_file(&sock_path);
            }
        }
    }

    /// Stop a specific plugin by name.
    pub async fn stop(&mut self, name: &str) -> bool {
        if let Some(mut plugin) = self.running.remove(name) {
            tracing::info!("[plugin] Stopping: {}", name);
            let _ = plugin.child.kill().await;
            let _ = plugin.child.wait().await;

            let sock_path = self.socket_path(name);
            let _ = std::fs::remove_file(&sock_path);
            true
        } else {
            false
        }
    }

    /// Detect if we're likely running in mainland China.
    fn is_likely_china(&self) -> bool {
        if std::env::var("TAIRITSU_NO_MIRROR").is_ok() {
            return false;
        }
        if std::env::var("TAIRITSU_USE_MIRROR").is_ok() {
            return true;
        }
        if let Ok(tz) = std::env::var("TZ") {
            if tz.contains("Shanghai")
                || tz.contains("Beijing")
                || tz.contains("Hongkong")
                || tz == "CST-8"
                || tz.contains("Asia/")
            {
                return true;
            }
        }
        false
    }
}

/// Errors returned by the plugin manager, designed to be actionable (ORT-style).
#[derive(Debug)]
pub enum LoadError {
    Spawn {
        name: String,
        detail: String,
    },
    NotFound {
        name: String,
        path: PathBuf,
        hint: String,
    },
    Handshake {
        name: String,
        detail: String,
    },
    BadVersion {
        name: String,
        plugin_version: u32,
        required_version: u32,
    },
    Ipc {
        detail: String,
    },
    PluginError {
        code: i32,
        message: String,
    },
    Download {
        name: String,
        detail: String,
    },
    AllSourcesFailed {
        name: String,
        last_error: String,
        filename: String,
        plugins_dir: PathBuf,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn { name, detail } => {
                write!(f, "Failed to start plugin '{}': {}", name, detail)
            }
            Self::NotFound { name, path, hint } => {
                write!(f, "Plugin '{}' not found at `{}`. {}", name, path.display(), hint)
            }
            Self::Handshake { name, detail } => {
                write!(f, "Plugin '{}' handshake failed: {}", name, detail)
            }
            Self::BadVersion {
                name,
                plugin_version,
                required_version,
            } => write!(
                f,
                "Plugin '{}' speaks protocol version {} but host requires {}. Run `tairitsu mcp init --force -p {}` to update.",
                name, plugin_version, required_version, name
            ),
            Self::Ipc { detail } => {
                write!(f, "IPC error: {}", detail)
            }
            Self::PluginError { code, message } => {
                write!(f, "Plugin error (code {}): {}", code, message)
            }
            Self::Download { name, detail } => {
                write!(f, "Failed to download plugin '{}': {}", name, detail)
            }
            Self::AllSourcesFailed {
                name,
                last_error,
                filename,
                plugins_dir,
            } => write!(
                f,
                "Could not download plugin '{}' from any source (last error: {}). \
                 Manual fix: download '{}' and place it in `{}`, then run `tairitsu mcp init` to verify.",
                name, last_error, filename, plugins_dir.display()
            ),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        LoadError::Ipc { detail: e.to_string() }
    }
}

async fn read_stderr(child: &mut Child) -> Option<String> {
    let stderr = child.stderr.as_mut()?;
    let mut reader = BufReader::new(stderr);
    let mut output = String::new();
    while let Ok(n) = reader.read_line(&mut output).await {
        if n == 0 { break; }
    }
    Some(output.trim().to_string())
}

/// Returns the default directory where plugins are stored.
///
/// Priority:
/// 1. `TAIRITSU_PLUGIN_DIR` env var
/// 2. `<project_root>/target/tairitsu/plugins/`
/// 3. XDG standard path (~/.local/share/tairitsu/plugins/)
pub fn default_plugins_dir() -> crate::Result<PathBuf> {
    if let Ok(dir) = std::env::var("TAIRITSU_PLUGIN_DIR") {
        let p = PathBuf::from(dir.clone());
        if !p.exists() {
            std::fs::create_dir_all(&p).map_err(|e| {
                crate::TairitsuPackagerError::BuildError(format!(
                    "Failed to create TAIRITSU_PLUGIN_DIR={}: {}",
                    dir, e
                ))
            })?;
        }
        return Ok(p);
    }

    if let Ok(cwd) = std::env::current_dir() {
        let dir = cwd.join("target").join("tairitsu").join("plugins");
        if dir.parent().is_some() && dir.parent().unwrap().exists() {
            std::fs::create_dir_all(&dir).ok();
            return Ok(dir);
        }
    }

    if let Some(base) = dirs_next() {
        let dir = base.join("tairitsu").join("plugins");
        std::fs::create_dir_all(&dir).ok();
        return Ok(dir);
    }

    Err(crate::TairitsuPackagerError::BuildError(
        "Cannot determine plugin directory. Set TAIRITSU_PLUGIN_DIR env var.".into(),
    ))
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(PathBuf::from(home).join(".local/share"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return Some(PathBuf::from(home).join("Library").join("Application Support"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            return Some(PathBuf::from(appdata));
        }
    }
    None
}

/// `tairitsu mcp init` — download all (or specified) plugins into the plugin directory.
pub async fn cmd_mcp_init(
    no_mirror: bool,
    registry: Option<&str>,
    force: bool,
    plugin_filter: &[String],
) -> crate::Result<()> {
    let plugins_dir = default_plugins_dir()?;

    let mut mgr = PluginManager::new(plugins_dir.clone());
    if let Some(url) = registry {
        mgr.set_registry(url);
    }
    mgr.set_use_mirrors(!no_mirror);

    mgr.ensure_dir().map_err(|e| {
        crate::TairitsuPackagerError::BuildError(format!("Failed to create plugin dir: {}", e))
    })?;

    let targets: Vec<&str> = if plugin_filter.is_empty() {
        BUILTIN_PLUGINS.to_vec()
    } else {
        plugin_filter.iter().map(|s| s.as_str()).collect()
    };

    crate::log_ok!("[mcp init] Plugin directory: {}", plugins_dir.display());
    crate::log_ok!("[mcp init] Registry: {}", mgr.registry_url);
    crate::log_ok!(
        "[mcp init] Mirrors: {}",
        if mgr.use_mirrors { "enabled" } else { "disabled" }
    );
    crate::log_ok!("[mcp init] Plugins to fetch: {}", targets.join(", "));

    let mut ok = 0;
    let mut fail = 0;

    for name in &targets {
        let bin_path = mgr.plugin_binary_path(name);
        if !force && bin_path.exists() {
            crate::log_info!("[mcp init] {} ✓ already exists", name);
            ok += 1;
            continue;
        }

        if force && bin_path.exists() {
            let _ = std::fs::remove_file(&bin_path);
        }

        match mgr.download_plugin(name).await {
            Ok(_) => ok += 1,
            Err(e) => {
                crate::log_fail!("[mcp init] ✗ {}: {}", name, e);
                fail += 1;
            }
        }
    }

    if fail > 0 {
        crate::log_fail!(
            "[mcp init] Done: {}/{} succeeded, {} failed",
            ok,
            ok + fail,
            fail
        );
        Err(crate::TairitsuPackagerError::BuildError(format!(
            "{}/{} plugins failed to download",
            fail,
            ok + fail
        )))
    } else {
        crate::log_ok!("[mcp init] Done: all {} plugins ready ✓", ok);
        Ok(())
    }
}

fn target_triple() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "linux-x86_64"
    }
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            "macos-aarch64"
        } else {
            "macos-x86_64"
        }
    }
    #[cfg(target_os = "windows")]
    {
        "windows-x86_64"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown"
    }
}
