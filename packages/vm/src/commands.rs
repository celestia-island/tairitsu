//! Command definitions and traits for type-safe host-guest communication
//!
//! This module provides enums and traits for defining commands that can be
//! passed between host and guest with compile-time type safety.

use serde::{Deserialize, Serialize};

/// Trait for commands that can be executed by the host
pub trait HostCommand: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// The response type for this command
    type Response: Serialize + for<'de> Deserialize<'de> + Send + Sync;

    /// Execute the command
    fn execute(&self) -> Result<Self::Response, String>;
}

/// Trait for commands that can be executed by the guest
pub trait GuestCommand: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// The response type for this command
    type Response: Serialize + for<'de> Deserialize<'de> + Send + Sync;

    /// Execute the command
    fn execute(&self) -> Result<Self::Response, String>;
}

/// Host commands - commands that the guest can send to the host
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum HostCommands {
    /// Get information about the host
    GetInfo,
    /// Echo a message back
    Echo(String),
    /// Custom command with arbitrary data
    Custom { name: String, data: String },
}

/// Guest commands - commands that the host can send to the guest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GuestCommands {
    /// Greet with a message
    Greet(String),
    /// Perform a computation
    Compute(String),
    /// Call back to the host
    CallHost(String),
    /// Custom command with arbitrary data
    Custom { name: String, data: String },
}

/// Host command responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HostResponse {
    /// Host information
    Info {
        name: String,
        version: String,
        status: String,
    },
    /// Simple text response
    Text(String),
}

/// Guest command responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GuestResponse {
    /// Simple text response
    Text(String),
}

/// Log levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }
}

/// Serialize a command to a string
pub fn serialize_command<T: Serialize>(cmd: &T) -> Result<String, String> {
    serde_json::to_string(cmd).map_err(|e| format!("Failed to serialize command: {}", e))
}

/// Deserialize a command from a string
pub fn deserialize_command<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, String> {
    serde_json::from_str(s).map_err(|e| format!("Failed to deserialize command: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_command_serialization() {
        let cmd = HostCommands::GetInfo;
        let serialized = serialize_command(&cmd).unwrap();
        let deserialized: HostCommands = deserialize_command(&serialized).unwrap();
        match deserialized {
            HostCommands::GetInfo => {}
            _ => panic!("Wrong command deserialized"),
        }
    }

    #[test]
    fn test_guest_command_serialization() {
        let cmd = GuestCommands::Greet("Hello".to_string());
        let serialized = serialize_command(&cmd).unwrap();
        let deserialized: GuestCommands = deserialize_command(&serialized).unwrap();
        match deserialized {
            GuestCommands::Greet(msg) => assert_eq!(msg, "Hello"),
            _ => panic!("Wrong command deserialized"),
        }
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }
}
