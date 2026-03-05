use thiserror::Error;

#[derive(Error, Debug)]
pub enum TairitsuPackagerError {
    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TairitsuPackagerError>;
