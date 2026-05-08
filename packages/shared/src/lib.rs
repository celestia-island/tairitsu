//! Shared types for tairitsu host-plugin IPC over Unix sockets.
//!
//! # Protocol
//!
//! Communication is line-delimited JSON (newline as frame delimiter):
//!
//! ```text
//! Host → Plugin:  { "id": 1, "method": "browser.navigate", "params": {...} }
//! Plugin → Host:  { "id": 1, "result": {...} }
//!                  { "id": 1, "error": { "code": -1, "message": "..." } }
//! ```
//!
//! Plugins can also push events to the host:
//!
//! ```text
//! Plugin → Host:  { "event": "log", "params": { "level": "info", "msg": "..." } }
//! ```

use serde::{Deserialize, Serialize};

/// Well-known capability IDs.
pub mod caps {
    pub const DEBUG_BROWSER: &str = "debug-browser";
    pub const VISUAL_DIFF: &str = "visual-diff";
    pub const TEST_RUNNER: &str = "test-runner";
}

/// Protocol version — bumped on breaking changes.
pub const PROTOCOL_VERSION: u32 = 1;

/// A request from host to plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Unique request ID (for request-response matching).
    pub id: u64,
    /// Method name, e.g. `"browser.navigate"`, `"screenshot.capture"`.
    pub method: String,
    /// Method-specific parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// A successful response from plugin to host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Matches the request ID.
    pub id: u64,
    /// Result value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// An error response from plugin to host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Matches the request ID.
    pub id: u64,
    /// Error details.
    pub error: ErrorBody,
}

/// Error body inside an ErrorResponse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Numeric error code (negative = plugin error, positive = protocol error).
    pub code: i32,
    /// Human-readable message.
    pub message: String,
}

/// An event pushed from plugin to host (unsolicited).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event type, e.g. `"log"`, `"ready"`, `"shutdown"`.
    pub event: String,
    /// Event payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Handshake message sent by plugin immediately after connecting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    /// Protocol version the plugin speaks.
    pub protocol_version: u32,
    /// Plugin name.
    pub name: String,
    /// Semantic version string.
    pub version: String,
    /// Capabilities this plugin provides.
    pub capabilities: Vec<String>,
    /// One-line description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Handshake response sent by host after validating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeAck {
    /// Whether the handshake was accepted.
    pub accepted: bool,
    /// Reason if rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Any message on the wire (union type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    Request(Request),
    Response(Response),
    Error(ErrorResponse),
    Event(Event),
    Handshake(Handshake),
    HandshakeAck(HandshakeAck),
}
