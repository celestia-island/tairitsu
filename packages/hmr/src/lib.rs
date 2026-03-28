//! Hot Module Replacement (HMR) for Tairitsu
//!
//! This crate provides HMR support for development mode, allowing
//! modules to be updated without a full page refresh.

#![warn(missing_docs)]

mod protocol;
mod registry;

pub use protocol::{HmrMessage, ModuleState};
pub use registry::{ModuleInfo, ModuleRegistry};

use std::sync::{Arc, RwLock};

/// Default HMR server port
pub const DEFAULT_HMR_PORT: u16 = 24678;

/// Default HMR WebSocket path
pub const DEFAULT_HMR_PATH: &str = "/__hmr";

/// HMR protocol version
pub const HMR_PROTOCOL_VERSION: &str = "1";

/// Client for HMR WebSocket connection
///
/// The HmrClient manages the WebSocket connection to the HMR server
/// and handles incoming hot reload messages.
#[derive(Debug, Clone)]
pub struct HmrClient {
    /// WebSocket server URL
    url: String,
    /// Module registry for tracking loaded modules
    module_registry: Arc<RwLock<ModuleRegistry>>,
    /// Connection state
    connected: Arc<RwLock<bool>>,
}

impl HmrClient {
    /// Create a new HMR client
    ///
    /// # Arguments
    /// * `url` - WebSocket server URL (e.g., "ws://localhost:24678")
    pub fn new(url: impl Into<String>) -> Self {
        HmrClient {
            url: url.into(),
            module_registry: Arc::new(RwLock::new(ModuleRegistry::new())),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Get the WebSocket URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the module registry
    pub fn registry(&self) -> &Arc<RwLock<ModuleRegistry>> {
        &self.module_registry
    }

    /// Check if connected to the HMR server
    pub fn is_connected(&self) -> bool {
        *self.connected.read().unwrap()
    }

    /// Register a module with the client
    ///
    /// # Arguments
    /// * `path` - Module path or URL
    ///
    /// # Returns
    /// Unique module ID
    pub fn register_module(&self, path: impl Into<String>) -> String {
        let registry = self.module_registry.read().unwrap();
        registry.register(path)
    }

    /// Handle an incoming HMR message
    ///
    /// # Arguments
    /// * `message` - The HMR message to handle
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn handle_message(&self, message: &HmrMessage) -> Result<(), String> {
        match message {
            HmrMessage::HotReload {
                module_id,
                code: _,
                dependencies,
            } => {
                let registry = self.module_registry.read().unwrap();

                if registry.get(module_id).is_some() {
                    // Module exists, trigger reload
                    if let Some(deps) = dependencies {
                        for dep in deps {
                            // Handle dependency reloads
                            let _ = registry.get(dep);
                        }
                    }
                    Ok(())
                } else {
                    Err(format!("Module not found: {}", module_id))
                }
            }

            HmrMessage::CssUpdate { url, css: _, media: _ } => {
                // CSS updates are handled by the browser
                log::debug!("CSS update received for: {}", url);
                Ok(())
            }

            HmrMessage::FullReload { reason } => {
                log::warn!(
                    "Full page reload required: {}",
                    reason.as_deref().unwrap_or("unknown reason")
                );
                Err("Full reload required".to_string())
            }

            HmrMessage::Error { message, .. } => {
                log::error!("HMR error: {}", message);
                Err(format!("HMR error: {}", message))
            }

            HmrMessage::Ping => {
                // Respond with pong
                Ok(())
            }

            HmrMessage::Pong => {
                // Heartbeat received
                Ok(())
            }

            HmrMessage::Connected { .. } => {
                *self.connected.write().unwrap() = true;
                log::info!("Connected to HMR server");
                Ok(())
            }

            HmrMessage::ModuleState { module_id, state } => {
                let registry = self.module_registry.read().unwrap();
                let _ = registry.update_state(module_id, state.clone());
                Ok(())
            }
        }
    }

    /// Disconnect from the HMR server
    pub fn disconnect(&self) {
        *self.connected.write().unwrap() = false;
    }
}

impl Default for HmrClient {
    fn default() -> Self {
        Self::new(format!("ws://localhost:{}", DEFAULT_HMR_PORT))
    }
}

/// Generate the HMR client script to inject into pages
///
/// This generates the JavaScript code that runs in the browser
/// to handle HMR WebSocket connection and module updates.
///
/// # Arguments
/// * `server_url` - WebSocket server URL (e.g., "ws://localhost:24678")
///
/// # Returns
/// JavaScript code as a string
pub fn hmr_script(server_url: &str) -> String {
    format!(
        r#"(function() {{
  const HMR_PROTOCOL_VERSION = "{}";
  const RECONNECT_DELAY = 2000;
  let ws = null;
  let reconnectTimer = null;
  const moduleRegistry = new Map();

  function connect() {{
    ws = new WebSocket("{}");

    ws.onopen = function() {{
      console.log("[HMR] Connected to server");
      clearTimeout(reconnectTimer);
    }};

    ws.onmessage = function(event) {{
      try {{
        const message = JSON.parse(event.data);
        handleMessage(message);
      }} catch (e) {{
        console.error("[HMR] Failed to parse message:", e);
      }}
    }};

    ws.onclose = function() {{
      console.log("[HMR] Disconnected from server");
      reconnectTimer = setTimeout(connect, RECONNECT_DELAY);
    }};

    ws.onerror = function(error) {{
      console.error("[HMR] WebSocket error:", error);
    }};
  }}

  function handleMessage(message) {{
    switch (message.type) {{
      case "HotReload":
        handleHotReload(message.data);
        break;
      case "CssUpdate":
        handleCssUpdate(message.data);
        break;
      case "FullReload":
        handleFullReload(message.data);
        break;
      case "Error":
        console.error("[HMR] Server error:", message.data.message);
        break;
      case "Ping":
        sendMessage({{ type: "Pong" }});
        break;
      case "Connected":
        console.log("[HMR] Server connected:", message.data);
        break;
      default:
        console.warn("[HMR] Unknown message type:", message.type);
    }}
  }}

  function handleHotReload(data) {{
    const {{ moduleId, code, dependencies }} = data;
    console.log("[HMR] Hot reload:", moduleId);

    // Update module registry
    moduleRegistry.set(moduleId, {{ code, dependencies }});

    // Dispatch custom event for application to handle
    window.dispatchEvent(new CustomEvent("hmr:reload", {{
      detail: {{ moduleId, code, dependencies }}
    }}));
  }}

  function handleCssUpdate(data) {{
    const {{ url, css }} = data;
    console.log("[HMR] CSS update:", url);

    // Find and update stylesheet
    for (const link of document.querySelectorAll("link[rel=stylesheet]")) {{
      if (link.href.includes(url)) {{
        // Force reload by updating timestamp
        const newUrl = url + (url.includes("?") ? "&" : "?") + "t=" + Date.now();
        link.href = newUrl;
        return;
      }}
    }}

    // Create new stylesheet if not found
    const style = document.createElement("style");
    style.textContent = css;
    document.head.appendChild(style);
  }}

  function handleFullReload(data) {{
    const {{ reason }} = data;
    console.log("[HMR] Full reload required:", reason || "unknown reason");
    location.reload();
  }}

  function sendMessage(message) {{
    if (ws && ws.readyState === WebSocket.OPEN) {{
      ws.send(JSON.stringify(message));
    }}
  }}

  // Register module for HMR tracking
  window.__HMR_REGISTER__ = function(moduleId, code) {{
    moduleRegistry.set(moduleId, {{ code }});
  }};

  // Start connection
  connect();

  // Cleanup on page unload
  window.addEventListener("beforeunload", function() {{
    if (ws) ws.close();
    clearTimeout(reconnectTimer);
  }});
}})();
"#,
        HMR_PROTOCOL_VERSION,
        server_url
    )
}

/// Generate HMR script for development server
///
/// This is a convenience function that uses the default HMR port.
pub fn hmr_script_default() -> String {
    hmr_script(&format!("ws://localhost:{}", DEFAULT_HMR_PORT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmr_client_creation() {
        let client = HmrClient::new("ws://localhost:24678");
        assert_eq!(client.url(), "ws://localhost:24678");
        assert!(!client.is_connected());
    }

    #[test]
    fn test_hmr_client_default() {
        let client = HmrClient::default();
        assert_eq!(client.url(), format!("ws://localhost:{}", DEFAULT_HMR_PORT));
    }

    #[test]
    fn test_hmr_script_generation() {
        let script = hmr_script("ws://localhost:24678");
        assert!(script.contains("WebSocket"));
        assert!(script.contains("ws://localhost:24678"));
        assert!(script.contains("HMR_PROTOCOL_VERSION"));
    }

    #[test]
    fn test_hmr_script_default() {
        let script = hmr_script_default();
        assert!(script.contains(&format!("ws://localhost:{}", DEFAULT_HMR_PORT)));
    }

    #[test]
    fn test_module_registration() {
        let client = HmrClient::default();
        let id = client.register_module("/test/module.js");

        let registry = client.registry().read().unwrap();
        let module = registry.get(&id).unwrap();
        assert_eq!(module.path, "/test/module.js");
    }

    #[test]
    fn test_handle_hot_reload_message() {
        let client = HmrClient::default();
        let id = client.register_module("/test/module.js");

        let message = HmrMessage::hot_reload(&id, "updated code");
        let result = client.handle_message(&message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_full_reload() {
        let client = HmrClient::default();
        let message = HmrMessage::full_reload_with_reason("test reload");

        let result = client.handle_message(&message);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Full reload required");
    }

    #[test]
    fn test_handle_css_update() {
        let client = HmrClient::default();
        let message = HmrMessage::css_update("style.css", "body { color: red; }");

        let result = client.handle_message(&message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_error_message() {
        let client = HmrClient::default();
        let message = HmrMessage::error("test error");

        let result = client.handle_message(&message);
        assert!(result.is_err());
    }

    #[test]
    fn test_connected_message() {
        let client = HmrClient::default();
        let message = HmrMessage::Connected {
            version: Some("1.0.0".to_string()),
            protocol_version: Some(HMR_PROTOCOL_VERSION.to_string()),
        };

        client.handle_message(&message).unwrap();
        assert!(client.is_connected());
    }

    #[test]
    fn test_disconnect() {
        let client = HmrClient::default();
        let message = HmrMessage::Connected {
            version: None,
            protocol_version: None,
        };

        client.handle_message(&message).unwrap();
        assert!(client.is_connected());

        client.disconnect();
        assert!(!client.is_connected());
    }
}
