//! Error types for data fetching operations

use thiserror::Error;

/// Errors that can occur during fetch operations
#[derive(Error, Clone, Debug)]
pub enum FetchError {
    /// Network error (connection refused, timeout, etc.)
    #[error("Network error: {0}")]
    Network(String),

    /// HTTP error (4xx, 5xx status codes)
    #[error("HTTP error: {status} - {message}")]
    Http {
        /// HTTP status code
        status: u16,
        /// Error message
        message: String,
    },

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Request was cancelled
    #[error("Request cancelled")]
    Cancelled,

    /// Other error
    #[error("Error: {0}")]
    Other(String),
}

impl FetchError {
    /// Create a network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create an HTTP error
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::Http {
            status,
            message: message.into(),
        }
    }

    /// Create an invalid URL error
    pub fn invalid_url(msg: impl Into<String>) -> Self {
        Self::InvalidUrl(msg.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a cache error
    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    /// Check if this is a network error
    pub fn is_network(&self) -> bool {
        matches!(self, Self::Network(_))
    }

    /// Check if this is an HTTP error
    pub fn is_http(&self) -> bool {
        matches!(self, Self::Http { .. })
    }

    /// Check if this is a serialization error
    pub fn is_serialization(&self) -> bool {
        matches!(self, Self::Serialization(_))
    }

    /// Get the HTTP status code if this is an HTTP error
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Http { status, .. } => Some(*status),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = FetchError::network("connection refused");
        assert!(err.is_network());
        assert!(!err.is_http());
        assert_eq!(err.to_string(), "Network error: connection refused");

        let err = FetchError::http(404, "not found");
        assert!(err.is_http());
        assert!(!err.is_network());
        assert_eq!(err.status_code(), Some(404));
        assert_eq!(err.to_string(), "HTTP error: 404 - not found");
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            FetchError::invalid_url("bad url").to_string(),
            "Invalid URL: bad url"
        );
        assert_eq!(
            FetchError::serialization("parse error").to_string(),
            "Serialization error: parse error"
        );
        assert_eq!(FetchError::cache("miss").to_string(), "Cache error: miss");
        assert_eq!(FetchError::Cancelled.to_string(), "Request cancelled");
        assert_eq!(
            FetchError::Other("unknown".to_string()).to_string(),
            "Error: unknown"
        );
    }
}
