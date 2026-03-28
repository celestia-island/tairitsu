//! HMR WebSocket protocol definitions
//!
//! This module defines the message types and protocol used for
//! Hot Module Replacement communication between client and server.

use serde::{Deserialize, Serialize};

/// HMR message types that can be sent over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum HmrMessage {
    /// Hot reload notification for JavaScript/WASM modules
    HotReload {
        /// Unique identifier for the module
        module_id: String,
        /// New source code for the module
        code: String,
        /// Optional dependencies that also need reloading
        #[serde(skip_serializing_if = "Option::is_none")]
        dependencies: Option<Vec<String>>,
    },

    /// CSS change notification for style updates
    CssUpdate {
        /// URL or identifier for the CSS resource
        url: String,
        /// New CSS content
        css: String,
        /// Optional media query for the stylesheet
        #[serde(skip_serializing_if = "Option::is_none")]
        media: Option<String>,
    },

    /// Full page reload required
    /// Used when HMR cannot be applied safely
    FullReload {
        /// Optional reason for the reload
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// Error notification from server
    Error {
        /// Error message
        message: String,
        /// Optional stack trace
        #[serde(skip_serializing_if = "Option::is_none")]
        stack: Option<String>,
    },

    /// Heartbeat/ping to keep connection alive
    Ping,

    /// Pong response to heartbeat
    Pong,

    /// Connected confirmation with server info
    Connected {
        /// Server version
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        /// HMR protocol version
        #[serde(skip_serializing_if = "Option::is_none")]
        protocol_version: Option<String>,
    },

    /// Module state synchronization
    ModuleState {
        /// Module identifier
        module_id: String,
        /// Current state of the module
        state: ModuleState,
    },
}

/// State of a module in the HMR system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleState {
    /// Module is currently loading
    Loading,
    /// Module is loaded and active
    Loaded,
    /// Module failed to load
    Failed {
        /// Error message
        error: String,
    },
    /// Module is disposed
    Disposed,
}

impl HmrMessage {
    /// Create a new hot reload message
    pub fn hot_reload(module_id: impl Into<String>, code: impl Into<String>) -> Self {
        HmrMessage::HotReload {
            module_id: module_id.into(),
            code: code.into(),
            dependencies: None,
        }
    }

    /// Create a new hot reload message with dependencies
    pub fn hot_reload_with_deps(
        module_id: impl Into<String>,
        code: impl Into<String>,
        dependencies: Vec<String>,
    ) -> Self {
        HmrMessage::HotReload {
            module_id: module_id.into(),
            code: code.into(),
            dependencies: Some(dependencies),
        }
    }

    /// Create a new CSS update message
    pub fn css_update(url: impl Into<String>, css: impl Into<String>) -> Self {
        HmrMessage::CssUpdate {
            url: url.into(),
            css: css.into(),
            media: None,
        }
    }

    /// Create a new CSS update message with media query
    pub fn css_update_with_media(
        url: impl Into<String>,
        css: impl Into<String>,
        media: impl Into<String>,
    ) -> Self {
        HmrMessage::CssUpdate {
            url: url.into(),
            css: css.into(),
            media: Some(media.into()),
        }
    }

    /// Create a full reload message
    pub fn full_reload() -> Self {
        HmrMessage::FullReload { reason: None }
    }

    /// Create a full reload message with a reason
    pub fn full_reload_with_reason(reason: impl Into<String>) -> Self {
        HmrMessage::FullReload {
            reason: Some(reason.into()),
        }
    }

    /// Create an error message
    pub fn error(message: impl Into<String>) -> Self {
        HmrMessage::Error {
            message: message.into(),
            stack: None,
        }
    }

    /// Create an error message with stack trace
    pub fn error_with_stack(message: impl Into<String>, stack: impl Into<String>) -> Self {
        HmrMessage::Error {
            message: message.into(),
            stack: Some(stack.into()),
        }
    }

    /// Check if this message requires a page reload
    pub fn requires_reload(&self) -> bool {
        matches!(self, HmrMessage::FullReload { .. })
    }

    /// Serialize message to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize message from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = HmrMessage::hot_reload("module-1", "console.log('hello');");
        let json = msg.to_json().unwrap();
        let parsed = HmrMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_css_update_serialization() {
        let msg = HmrMessage::css_update("style.css", "body { color: red; }");
        let json = msg.to_json().unwrap();
        let parsed = HmrMessage::from_json(&json).unwrap();

        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_full_reload() {
        let msg = HmrMessage::full_reload();
        assert!(msg.requires_reload());

        let msg_with_reason = HmrMessage::full_reload_with_reason("syntax error");
        assert!(msg_with_reason.requires_reload());
    }

    #[test]
    fn test_hot_reload_does_not_require_reload() {
        let msg = HmrMessage::hot_reload("module-1", "code");
        assert!(!msg.requires_reload());
    }
}
